use crate::cell_collector::get_live_cell_by_typescript;
use crate::indexer::{Cell, IndexerRpcClient};
use crate::settings::{OutpointConf, Settings};
use crate::tx_helper::TxHelper;
use crate::util::{
    check_capacity, get_live_cell, get_live_cell_with_cache, get_max_mature_number, is_mature,
    parse_privkey_path,
};
use ckb_sdk::{
    constants::{MIN_SECP_CELL_CAPACITY, ONE_CKB},
    Address, AddressPayload, GenesisInfo, HttpRpcClient, SECP256K1,
};
use ckb_types::prelude::{Pack, Unpack};
use ckb_types::{
    bytes::Bytes,
    core::{BlockView, Capacity, DepType, TransactionView},
    packed::{self, Byte32, CellDep, CellOutput, OutPoint, Script, WitnessArgs},
    prelude::{Builder, Entity},
    H256,
};
use int_enum::IntEnum;
use molecule::prelude::Byte;
use secp256k1::SecretKey;
use std::collections::HashMap;
use tockb_types::config::{CKB_UNITS, COLLATERAL_PERCENT, UDT_LEN, XT_CELL_CAPACITY};
use tockb_types::generated::tockb_cell_data::ToCKBCellData;
use tockb_types::tockb_cell_data::ToCKBTypeArgs;
use tockb_types::{basic, ToCKBCellDataView, ToCKBStatus, XChainKind};

pub struct Generator {
    pub rpc_client: HttpRpcClient,
    pub indexer_client: IndexerRpcClient,
    genesis_info: GenesisInfo,
    settings: Settings,
}

impl Generator {
    pub fn new(rpc_url: String, indexer_url: String, settings: Settings) -> Result<Self, String> {
        let mut rpc_client = HttpRpcClient::new(rpc_url);
        let indexer_client = IndexerRpcClient::new(indexer_url);
        let genesis_block: BlockView = rpc_client
            .get_block_by_number(0)?
            .expect("Can not get genesis block?")
            .into();
        let genesis_info = GenesisInfo::from_block(&genesis_block)?;
        Ok(Self {
            rpc_client,
            indexer_client,
            genesis_info,
            settings,
        })
    }

    fn get_price_oracle(&mut self) -> Result<(CellDep, u128), String> {
        let outpoint = OutPoint::new_builder()
            .tx_hash(
                Byte32::from_slice(
                    &hex::decode(&self.settings.price_oracle.outpoint.tx_hash)
                        .map_err(|e| format!("invalid price oracle config. err: {}", e))?,
                )
                .map_err(|e| format!("invalid price oracle config. err: {}", e))?,
            )
            .index(self.settings.price_oracle.outpoint.index.pack())
            .build();
        let cell_dep = CellDep::new_builder()
            .out_point(outpoint.clone())
            .dep_type(DepType::Code.into())
            .build();
        let cell = get_live_cell(&mut self.rpc_client, outpoint, true)?;

        let mut buf = [0u8; UDT_LEN];
        buf.copy_from_slice(cell.1.as_ref());
        let price = u128::from_le_bytes(buf);
        Ok((cell_dep, price))
    }

    fn add_cell_deps(
        &mut self,
        helper: &mut TxHelper,
        outpoints: Vec<OutpointConf>,
    ) -> Result<(), String> {
        let mut builder = helper.transaction.as_advanced_builder();
        for conf in outpoints {
            let outpoint = OutPoint::new_builder()
                .tx_hash(
                    Byte32::from_slice(
                        &hex::decode(conf.tx_hash)
                            .map_err(|e| format!("invalid OutpointConf config. err: {}", e))?,
                    )
                    .map_err(|e| format!("invalid OutpointConf config. err: {}", e))?,
                )
                .index(conf.index.pack())
                .build();
            builder = builder.cell_dep(
                CellDep::new_builder()
                    .out_point(outpoint)
                    .dep_type(DepType::Code.into())
                    .build(),
            );
        }
        helper.transaction = builder.build();
        Ok(())
    }

    pub fn deposit_request(
        &mut self,
        from_lockscript: Script,
        tx_fee: u64,
        user_lockscript: Script,
        pledge: u64,
        kind: u8,
        lot_size: u8,
    ) -> Result<TransactionView, String> {
        let to_capacity = pledge * CKB_UNITS;
        let mut helper = TxHelper::default();

        let outpoints = vec![
            self.settings.lockscript.outpoint.clone(),
            self.settings.typescript.outpoint.clone(),
        ];
        self.add_cell_deps(&mut helper, outpoints)?;

        let tockb_data = ToCKBCellData::new_builder()
            .status(Byte::new(ToCKBStatus::Initial.int_value()))
            .lot_size(Byte::new(lot_size))
            .user_lockscript(basic::Script::from_slice(user_lockscript.as_slice()).unwrap())
            .build()
            .as_bytes();
        check_capacity(to_capacity, tockb_data.len())?;
        let lockscript_code_hash = hex::decode(&self.settings.lockscript.code_hash)
            .expect("wrong lockscript code hash config");
        let typescript_code_hash = hex::decode(&self.settings.typescript.code_hash)
            .expect("wrong typescript code hash config");
        let mut typescript_args = [0u8; 37];
        let typescript = Script::new_builder()
            .code_hash(Byte32::from_slice(&typescript_code_hash).unwrap())
            .hash_type(DepType::Code.into())
            .args(Bytes::from(typescript_args.to_vec()).pack())
            .build();
        let typescript_hash = typescript.calc_script_hash();
        let lockscript = Script::new_builder()
            .code_hash(Byte32::from_slice(&lockscript_code_hash).unwrap())
            .hash_type(DepType::Code.into())
            // TODO: should change args to `code_hash + hash_type + kind`
            .args(typescript_hash.as_bytes().pack())
            .build();
        let to_output = CellOutput::new_builder()
            .capacity(Capacity::shannons(to_capacity).pack())
            .type_(Some(typescript.clone()).pack())
            .lock(lockscript)
            .build();
        helper.add_output(to_output.clone(), tockb_data);
        // get tx with empty typescript_args
        let mut tx = helper.supply_capacity(
            &mut self.rpc_client,
            &mut self.indexer_client,
            from_lockscript,
            &self.genesis_info,
            tx_fee,
        )?;
        // fill typescript args with first outpoint
        let first_outpoint = tx
            .inputs()
            .get(0)
            .expect("should have input")
            .previous_output()
            .as_bytes();
        let new_typescript_args = ToCKBTypeArgs::new_builder()
            .xchain_kind(Byte::new(kind))
            .cell_id(basic::OutPoint::from_slice(first_outpoint.as_ref()).unwrap())
            .build()
            .as_bytes();
        assert!(
            new_typescript_args.len() == 37,
            "typescript_args len should be 37"
        );
        let new_typescript = typescript
            .as_builder()
            .args(new_typescript_args.pack())
            .build();
        let new_output = to_output
            .as_builder()
            .type_(Some(new_typescript).pack())
            .build();
        let mut new_outputs = tx.outputs().into_iter().collect::<Vec<_>>();
        new_outputs[0] = new_output;
        let tx = tx.as_advanced_builder().set_outputs(new_outputs).build();
        Ok(tx)
    }

    fn get_ckb_cell(
        &mut self,
        helper: &mut TxHelper,
        cell_typescript: Script,
        add_to_input: bool,
    ) -> Result<(CellOutput, Bytes), String> {
        let genesis_info = self.genesis_info.clone();
        let cell = get_live_cell_by_typescript(&mut self.indexer_client, cell_typescript)?
            .ok_or(format!("cell not found"))?;
        let ckb_cell = CellOutput::from(cell.output);
        let ckb_cell_data = packed::Bytes::from(cell.output_data).raw_data();
        if add_to_input {
            let mut get_live_cell_fn = |out_point: OutPoint, with_data: bool| {
                get_live_cell(&mut self.rpc_client, out_point, with_data).map(|(output, _)| output)
            };

            helper.add_input(
                cell.out_point.into(),
                None,
                &mut get_live_cell_fn,
                &genesis_info,
                true,
            )?;
        }
        Ok((ckb_cell, ckb_cell_data))
    }

    pub fn bonding(
        &mut self,
        from_lockscript: Script,
        tx_fee: u64,
        cell_typescript: Script,
        signer_lockscript: Script,
        lock_address: String,
    ) -> Result<TransactionView, String> {
        let mut helper = TxHelper::default();
        let (ckb_cell, ckb_cell_data) = self.get_ckb_cell(&mut helper, cell_typescript, true)?;
        let input_capacity: u64 = ckb_cell.capacity().unpack();

        let type_script = ckb_cell
            .type_()
            .to_opt()
            .expect("should return ckb type script");
        let kind: u8 = type_script.args().raw_data()[0];
        let data_view: ToCKBCellDataView =
            ToCKBCellDataView::new(ckb_cell_data.as_ref(), XChainKind::from_int(kind).unwrap())
                .map_err(|err| format!("Parse to ToCKBCellDataView error: {}", err as i8))?;

        let sudt_amount: u128 = data_view
            .get_lot_xt_amount()
            .map_err(|err| format!("get_lot_xt_amount error: {}", err as i8))?;
        let (_price_oracle_dep, price) = self.get_price_oracle()?;
        let to_capacity = (input_capacity as u128
            + 2 * XT_CELL_CAPACITY as u128
            + sudt_amount * (COLLATERAL_PERCENT as u128) / (100 * price) * CKB_UNITS as u128)
            as u64;

        let outpoints = vec![
            self.settings.price_oracle.outpoint.clone(),
            self.settings.typescript.outpoint.clone(),
            self.settings.lockscript.outpoint.clone(),
        ];
        self.add_cell_deps(&mut helper, outpoints)?;

        let mut to_data_view = data_view.clone();
        to_data_view.status = ToCKBStatus::Bonded;
        to_data_view.x_lock_address = Bytes::from(lock_address);
        let tockb_data = to_data_view
            .as_molecule_data()
            .map_err(|e| format!("serde tockb_data err: {}", e))?;

        check_capacity(to_capacity, tockb_data.len())?;

        let to_output = CellOutput::new_builder()
            .capacity(to_capacity.pack())
            .type_(ckb_cell.type_())
            .lock(ckb_cell.lock())
            .build();
        helper.add_output(to_output, tockb_data);
        let tx = helper.supply_capacity(
            &mut self.rpc_client,
            &mut self.indexer_client,
            from_lockscript,
            &self.genesis_info,
            tx_fee,
        )?;
        Ok(tx)
    }
}

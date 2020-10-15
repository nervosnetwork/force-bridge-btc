use crate::cell_collector::{collect_sudt_amount, get_live_cell_by_typescript};
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
use std::str::FromStr;

use tockb_types::config::{
    CKB_UNITS, COLLATERAL_PERCENT, PLEDGE, SIGNER_FEE_RATE, UDT_LEN, XT_CELL_CAPACITY,
};
use tockb_types::generated::mint_xt_witness::{BTCSPVProof, MintXTWitness};
use tockb_types::generated::tockb_cell_data::ToCKBCellData;
use tockb_types::tockb_cell_data::ToCKBTypeArgs;
use tockb_types::{
    basic, BtcExtraView, ToCKBCellDataView, ToCKBStatus, ToCKBTypeArgsView, XChainKind, XExtraView,
};

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
            .args(typescript.as_slice()[0..54].pack())
            .build();
        let to_output = CellOutput::new_builder()
            .capacity(Capacity::shannons(to_capacity).pack())
            .type_(Some(typescript.clone()).pack())
            .lock(lockscript.clone())
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

        let new_lockscript = lockscript
            .as_builder()
            .args(new_typescript.as_slice()[0..54].pack())
            .build();
        let new_output = to_output
            .as_builder()
            .type_(Some(new_typescript).pack())
            .lock(new_lockscript)
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

        let typescript_args = ToCKBTypeArgsView::from_slice(type_script.args().raw_data().as_ref())
            .map_err(|err| format!("Parse to ToCKBTypeArgsView error: {}", err as i8))?;

        let data_view: ToCKBCellDataView =
            ToCKBCellDataView::new(ckb_cell_data.as_ref(), typescript_args.xchain_kind)
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
        to_data_view.signer_lockscript = signer_lockscript.as_bytes();
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

    pub fn mint_xt(
        &mut self,
        from_lockscript: Script,
        tx_fee: u64,
        cell_typescript: Script,
        spv_proof: Vec<u8>,
    ) -> Result<TransactionView, String> {
        let mut helper = TxHelper::default();
        let (from_cell, ckb_cell_data) = self.get_ckb_cell(&mut helper, cell_typescript, true)?;
        let from_ckb_cell_data = ToCKBCellData::from_slice(ckb_cell_data.as_ref()).unwrap();

        // add cellDeps
        let outpoints = vec![
            self.settings.btc_difficulty_cell.outpoint.clone(),
            self.settings.lockscript.outpoint.clone(),
            self.settings.typescript.outpoint.clone(),
            self.settings.sudt.outpoint.clone(),
        ];
        self.add_cell_deps(&mut helper, outpoints)?;

        let (tockb_typescript, _) = match from_cell.type_().to_opt() {
            Some(script) => (script.clone(), script.args().raw_data().as_ref()[0]),
            None => return Err("typescript of tockb cell is none".to_owned()),
        };
        let tockb_lockscript = from_cell.lock();

        let typescript_args =
            ToCKBTypeArgsView::from_slice(tockb_typescript.args().raw_data().as_ref())
                .map_err(|err| format!("Parse to ToCKBTypeArgsView error: {}", err as i8))?;

        let data_view = ToCKBCellDataView::new(ckb_cell_data.as_ref(), typescript_args.xchain_kind)
            .map_err(|err| format!("Parse to ToCKBCellDataView error: {}", err as i8))?;
        let lot_amount = data_view
            .get_lot_xt_amount()
            .map_err(|_| "get lot_amount from tockb cell data error".to_owned())?;
        let from_capacity: u64 = from_cell.capacity().unpack();
        // gen output of tockb cell
        {
            let to_capacity = from_capacity - PLEDGE - XT_CELL_CAPACITY;

            // get tx_id and funding_output_index from spv_proof
            let btc_spv_proof = BTCSPVProof::from_slice(spv_proof.as_slice())
                .map_err(|err| format!("btc_spv_proof invalid: {}", err))?;
            let tx_id = btc_spv_proof.tx_id().raw_data();
            let funding_output_index: u32 = btc_spv_proof.funding_output_index().into();

            let mut output_data_view = data_view.clone();
            output_data_view.status = ToCKBStatus::Warranty;
            output_data_view.x_extra = XExtraView::Btc(BtcExtraView {
                lock_tx_hash: tx_id.into(),
                lock_vout_index: funding_output_index,
            });
            let tockb_data = output_data_view
                .as_molecule_data()
                .expect("output_data_view.as_molecule_data error");
            check_capacity(to_capacity, tockb_data.len())?;

            let to_output = CellOutput::new_builder()
                .capacity(Capacity::shannons(to_capacity).pack())
                .type_(Some(tockb_typescript).pack())
                .lock(tockb_lockscript.clone())
                .build();
            helper.add_output(to_output, tockb_data);
        }
        // 2 xt cells
        {
            // mint xt cell to user, amount = lot_size * (1 - signer fee rate)
            let user_lockscript = Script::from_slice(
                from_ckb_cell_data.user_lockscript().as_slice(),
            )
            .map_err(|e| format!("parse user_lockscript from tockb_cell_data error: {}", e))?;

            let sudt_typescript_code_hash = hex::decode(&self.settings.sudt.code_hash)
                .expect("wrong sudt_script code hash config");
            let sudt_typescript = Script::new_builder()
                .code_hash(Byte32::from_slice(&sudt_typescript_code_hash).unwrap())
                .hash_type(DepType::Code.into())
                .args(tockb_lockscript.calc_script_hash().as_bytes().pack())
                .build();

            let sudt_user_output = CellOutput::new_builder()
                .capacity(Capacity::shannons(PLEDGE).pack())
                .type_(Some(sudt_typescript.clone()).pack())
                .lock(user_lockscript)
                .build();

            let (to_user, to_signer) = {
                let signer_fee = lot_amount * SIGNER_FEE_RATE.0 / SIGNER_FEE_RATE.1;
                (lot_amount - signer_fee, signer_fee)
            };
            let to_user_amount_data: Bytes = to_user.to_le_bytes().to_vec().into();
            helper.add_output(sudt_user_output, to_user_amount_data);

            // xt cell of signer fee
            let signer_lockscript = Script::from_slice(
                from_ckb_cell_data.signer_lockscript().as_slice(),
            )
            .map_err(|e| format!("parse signer_lockscript from tockb_cell_data error: {}", e))?;

            let sudt_signer_output = CellOutput::new_builder()
                .capacity(Capacity::shannons(XT_CELL_CAPACITY).pack())
                .type_(Some(sudt_typescript).pack())
                .lock(signer_lockscript)
                .build();

            let to_signer_amount_data = to_signer.to_le_bytes().to_vec().into();
            helper.add_output(sudt_signer_output, to_signer_amount_data);
        }

        // add witness
        {
            let witness_data = MintXTWitness::new_builder()
                .spv_proof(spv_proof.into())
                .cell_dep_index_list(vec![0].into())
                .build();
            let witness = WitnessArgs::new_builder()
                .input_type(Some(witness_data.as_bytes()).pack())
                .build();

            helper.transaction = helper
                .transaction
                .as_advanced_builder()
                .set_witnesses(vec![witness.as_bytes().pack()])
                .build();
        }

        // build tx
        let tx = helper.supply_capacity(
            &mut self.rpc_client,
            &mut self.indexer_client,
            from_lockscript,
            &self.genesis_info,
            tx_fee,
        )?;
        Ok(tx)
    }

    pub fn pre_term_redeem(
        &mut self,
        from_lockscript: Script,
        tx_fee: u64,
        cell_typescript: Script,
        x_unlock_address: String,
        redeemer_lockscript: Script,
    ) -> Result<TransactionView, String> {
        let mut helper = TxHelper::default();
        let (from_cell, ckb_cell_data) = self.get_ckb_cell(&mut helper, cell_typescript, true)?;
        let from_ckb_cell_data = ToCKBCellData::from_slice(ckb_cell_data.as_ref()).unwrap();

        // add cellDeps
        {
            let outpoints = vec![
                self.settings.lockscript.outpoint.clone(),
                self.settings.typescript.outpoint.clone(),
                self.settings.sudt.outpoint.clone(),
            ];
            self.add_cell_deps(&mut helper, outpoints)?;
        }

        // get input tockb cell and basic info
        let (tockb_typescript, kind) = match from_cell.type_().to_opt() {
            Some(script) => (script.clone(), script.args().raw_data().as_ref()[0]),
            None => return Err("typescript of tockb cell is none".to_owned()),
        };
        let tockb_lockscript = from_cell.lock();

        let typescript_args =
            ToCKBTypeArgsView::from_slice(tockb_typescript.args().raw_data().as_ref())
                .map_err(|err| format!("Parse to ToCKBTypeArgsView error: {}", err as i8))?;

        let data_view = ToCKBCellDataView::new(ckb_cell_data.as_ref(), typescript_args.xchain_kind)
            .map_err(|err| format!("Parse to ToCKBCellDataView error: {}", err as i8))?;
        let lot_amount = data_view
            .get_lot_xt_amount()
            .map_err(|_| "get lot_amount from tockb cell data error".to_owned())?;
        let from_capacity: u64 = from_cell.capacity().unpack();

        let sudt_typescript_code_hash =
            hex::decode(&self.settings.sudt.code_hash).expect("wrong sudt_script code hash config");
        let sudt_typescript = Script::new_builder()
            .code_hash(Byte32::from_slice(&sudt_typescript_code_hash).unwrap())
            .hash_type(DepType::Code.into())
            .args(tockb_lockscript.calc_script_hash().as_bytes().pack())
            .build();

        let (redeemer_is_depositor, user_lockscript) = {
            (
                data_view.user_lockscript == redeemer_lockscript.as_bytes(),
                data_view.user_lockscript.clone(),
            )
        };

        // gen output of tockb cell
        {
            let to_capacity = from_capacity;
            let mut output_data_view = data_view.clone();
            output_data_view.status = ToCKBStatus::Redeeming;
            output_data_view.x_unlock_address = x_unlock_address.as_bytes().to_vec().into();
            output_data_view.redeemer_lockscript = redeemer_lockscript.as_bytes();

            let tockb_data = output_data_view
                .as_molecule_data()
                .expect("output_data_view.as_molecule_data error");
            check_capacity(to_capacity, tockb_data.len())?;

            let to_output = CellOutput::new_builder()
                .capacity(Capacity::shannons(to_capacity).pack())
                .type_(Some(tockb_typescript).pack())
                .lock(tockb_lockscript.clone())
                .build();
            helper.add_output(to_output, tockb_data);
        }

        // collect xt cell inputs to burn lot_amount xt
        {
            let signer_fee = lot_amount * SIGNER_FEE_RATE.0 / SIGNER_FEE_RATE.1;
            let mut need_sudt_amount = lot_amount;
            if !redeemer_is_depositor {
                need_sudt_amount += signer_fee;
            }

            helper.supply_sudt(
                &mut self.rpc_client,
                &mut self.indexer_client,
                from_lockscript.clone(),
                &self.genesis_info,
                need_sudt_amount,
                sudt_typescript.clone(),
            )?;

            if !redeemer_is_depositor {
                let to_depositor_xt_cell = CellOutput::new_builder()
                    .capacity(Capacity::shannons(XT_CELL_CAPACITY).pack())
                    .type_(Some(sudt_typescript).pack())
                    .lock(
                        Script::from_slice(user_lockscript.as_ref())
                            .expect("user_lockscript decode from input_data error"),
                    )
                    .build();

                let data = signer_fee.to_le_bytes().to_vec().into();
                helper.add_output(to_depositor_xt_cell, data)
            }
        }

        // build tx
        let tx = helper.supply_capacity(
            &mut self.rpc_client,
            &mut self.indexer_client,
            from_lockscript,
            &self.genesis_info,
            tx_fee,
        )?;
        Ok(tx)
    }

    pub fn withdraw_collateral(
        &mut self,
        from_lockscript: Script,
        tx_fee: u64,
        cell_typescript: Script,
        spv_proof: Vec<u8>,
    ) -> Result<TransactionView, String> {
        let mut helper = TxHelper::default();

        let (ckb_cell, _) = self.get_ckb_cell(&mut helper, cell_typescript, true)?;
        let to_capacity: u64 = ckb_cell.capacity().unpack();

        let outpoints = vec![
            self.settings.btc_difficulty_cell.outpoint.clone(),
            self.settings.lockscript.outpoint.clone(),
            self.settings.typescript.outpoint.clone(),
        ];
        self.add_cell_deps(&mut helper, outpoints)?;

        {
            let witness_data = MintXTWitness::new_builder()
                .spv_proof(spv_proof.into())
                .cell_dep_index_list(vec![0].into())
                .build();
            let witness = WitnessArgs::new_builder()
                .input_type(Some(witness_data.as_bytes()).pack())
                .build();

            helper.transaction = helper
                .transaction
                .as_advanced_builder()
                .set_witnesses(vec![witness.as_bytes().pack()])
                .build();
        }

        let to_output = CellOutput::new_builder()
            .capacity(Capacity::shannons(to_capacity).pack())
            .lock(from_lockscript.clone())
            .build();
        helper.add_output(to_output, Bytes::new());

        let tx = helper.supply_capacity(
            &mut self.rpc_client,
            &mut self.indexer_client,
            from_lockscript,
            &self.genesis_info,
            tx_fee,
        )?;
        Ok(tx)
    }

    pub fn transfer_sudt(
        &mut self,
        from_lockscript: Script,
        kind: u8,
        to_lockscript: Script,
        sudt_amount: u128,
        ckb_amount: u64,
        tx_fee: u64,
    ) -> Result<TransactionView, String> {
        //let ckb_amount: u64 = CapacityParser.parse(&ckb_amount)?.into();
        let mut helper = TxHelper::default();

        // add cellDeps
        let outpoints = vec![self.settings.sudt.outpoint.clone()];
        self.add_cell_deps(&mut helper, outpoints)?;

        let lockscript_code_hash = hex::decode(self.settings.lockscript.code_hash.clone())
            .expect("wrong lockscript code hash config");
        let typescript_code_hash = hex::decode(self.settings.typescript.code_hash.clone())
            .expect("wrong typescript code hash config");

        let typescript_args = ToCKBTypeArgs::new_builder()
            .xchain_kind(Byte::new(kind))
            .build();

        let typescript = Script::new_builder()
            .code_hash(Byte32::from_slice(&typescript_code_hash).unwrap())
            .hash_type(DepType::Code.into())
            .args(typescript_args.as_bytes().pack())
            .build();
        let lockscript = Script::new_builder()
            .code_hash(Byte32::from_slice(&lockscript_code_hash).unwrap())
            .hash_type(DepType::Code.into())
            .args(typescript.as_slice()[0..54].pack())
            .build();

        {
            let sudt_typescript_code_hash = hex::decode(self.settings.sudt.code_hash.clone())
                .expect("wrong sudt_script code hash config");
            let sudt_typescript = Script::new_builder()
                .code_hash(Byte32::from_slice(&sudt_typescript_code_hash).unwrap())
                .hash_type(DepType::Code.into())
                .args(lockscript.calc_script_hash().as_bytes().pack())
                .build();

            let sudt_output = CellOutput::new_builder()
                .capacity(Capacity::shannons(ckb_amount).pack())
                .type_(Some(sudt_typescript.clone()).pack())
                .lock(to_lockscript)
                .build();

            helper.add_output(sudt_output, sudt_amount.to_le_bytes().to_vec().into());

            helper.supply_sudt(
                &mut self.rpc_client,
                &mut self.indexer_client,
                from_lockscript.clone(),
                &self.genesis_info,
                sudt_amount,
                sudt_typescript.clone(),
            )?;
        }

        // add signature to pay tx fee
        let tx = helper.supply_capacity(
            &mut self.rpc_client,
            &mut self.indexer_client,
            from_lockscript,
            &self.genesis_info,
            tx_fee,
        )?;
        Ok(tx)
    }

    pub fn get_sudt_balance(&mut self, address: String, kind: u8) -> Result<u128, String> {
        let addr_lockscript: Script = Address::from_str(&address)?.payload().into();

        let lockscript_code_hash = hex::decode(&self.settings.lockscript.code_hash)
            .expect("wrong lockscript code hash config");
        let typescript_code_hash = hex::decode(&self.settings.typescript.code_hash)
            .expect("wrong typescript code hash config");

        let typescript_args = ToCKBTypeArgs::new_builder()
            .xchain_kind(Byte::new(kind))
            .build();

        let typescript = Script::new_builder()
            .code_hash(Byte32::from_slice(&typescript_code_hash).unwrap())
            .hash_type(DepType::Code.into())
            .args(typescript_args.as_bytes().pack())
            .build();
        let lockscript = Script::new_builder()
            .code_hash(Byte32::from_slice(&lockscript_code_hash).unwrap())
            .hash_type(DepType::Code.into())
            .args(typescript.as_slice()[0..54].pack())
            .build();

        let sudt_typescript_code_hash =
            hex::decode(&self.settings.sudt.code_hash).expect("wrong sudt_script code hash config");
        let sudt_typescript = Script::new_builder()
            .code_hash(Byte32::from_slice(&sudt_typescript_code_hash).unwrap())
            .hash_type(DepType::Code.into())
            .args(lockscript.calc_script_hash().as_bytes().pack())
            .build();

        collect_sudt_amount(&mut self.indexer_client, addr_lockscript, sudt_typescript)
    }
}

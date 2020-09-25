use crate::indexer::{Cell, IndexerRpcClient};
use crate::settings::Settings;
use crate::tx_helper::TxHelper;
use crate::util::{
    check_capacity, get_live_cell, get_live_cell_with_cache, get_max_mature_number, is_mature,
    parse_privkey_path,
};
use ckb_sdk::{
    constants::{MIN_SECP_CELL_CAPACITY, ONE_CKB},
    Address, AddressPayload, GenesisInfo, HttpRpcClient, SECP256K1,
};
use ckb_types::prelude::Pack;
use ckb_types::{
    bytes::Bytes,
    core::{BlockView, Capacity, DepType, TransactionView},
    packed::{Byte32, CellDep, CellOutput, OutPoint, Script, WitnessArgs},
    prelude::{Builder, Entity},
    H256,
};
use int_enum::IntEnum;
use molecule::prelude::Byte;
use secp256k1::SecretKey;
use std::collections::HashMap;
use tockb_types::config::CKB_UNITS;
use tockb_types::generated::tockb_cell_data::ToCKBCellData;
use tockb_types::{basic, ToCKBStatus};

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
        dbg!(&genesis_info);
        Ok(Self {
            rpc_client,
            indexer_client,
            genesis_info,
            settings,
        })
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

        let lockscript_out_point = OutPoint::new_builder()
            .tx_hash(
                Byte32::from_slice(
                    &hex::decode(&self.settings.lockscript.outpoint.tx_hash)
                        .map_err(|e| format!("invalid lockscript config. err: {}", e))?,
                )
                .map_err(|e| format!("invalid lockscript config. err: {}", e))?,
            )
            .index(self.settings.lockscript.outpoint.index.pack())
            .build();
        let typescript_out_point = OutPoint::new_builder()
            .tx_hash(
                Byte32::from_slice(
                    &hex::decode(&self.settings.typescript.outpoint.tx_hash)
                        .map_err(|e| format!("invalid typescript config. err: {}", e))?,
                )
                .map_err(|e| format!("invalid typescript config. err: {}", e))?,
            )
            .index(self.settings.typescript.outpoint.index.pack())
            .build();
        let typescript_cell_dep = CellDep::new_builder()
            .out_point(typescript_out_point)
            .dep_type(DepType::Code.into())
            .build();
        let lockscript_cell_dep = CellDep::new_builder()
            .out_point(lockscript_out_point)
            .dep_type(DepType::Code.into())
            .build();
        helper.transaction = helper
            .transaction
            .as_advanced_builder()
            .cell_dep(typescript_cell_dep)
            .cell_dep(lockscript_cell_dep)
            .build();
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
        let typescript = Script::new_builder()
            .code_hash(Byte32::from_slice(&typescript_code_hash).unwrap())
            .hash_type(DepType::Code.into())
            .args(vec![kind].pack())
            .build();
        let typescript_hash = typescript.calc_script_hash();
        let lockscript = Script::new_builder()
            .code_hash(Byte32::from_slice(&lockscript_code_hash).unwrap())
            .hash_type(DepType::Code.into())
            .args(typescript_hash.as_bytes().pack())
            .build();
        let to_output = CellOutput::new_builder()
            .capacity(Capacity::shannons(to_capacity).pack())
            .type_(Some(typescript).pack())
            .lock(lockscript)
            .build();
        helper.add_output(to_output, tockb_data);
        helper.supply_capacity(
            &mut self.rpc_client,
            &mut self.indexer_client,
            from_lockscript,
            &self.genesis_info,
            tx_fee,
        )
    }
}

use ckb_sdk::{HttpRpcClient, TxHelper};
use ckb_cli::utils::other:: {
    check_capacity, get_address, get_arg_value, get_live_cell_with_cache,
    get_max_mature_number, get_network_type, get_privkey_signer, get_to_data, is_mature,
    read_password, sync_to_tip,
};
use molecule::prelude::{Entity, Byte};
use thiserror::Error;
use tockb_types::{
    config,
    cell,
    generated::{basic, btc_difficulty, toCKB_cell_data::ToCKBCellData},
};

#[derive(Error, Debug)]
pub enum ToCKBError {
    #[error("data store disconnected")]
    Disconnect(#[from] std::io::Error),
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("unknown data store error")]
    Unknown,
}

pub struct DepositRequestArgs {
    pub kind: u8,
    pub pledge: u64,
    pub lot_size: u8,
    pub user_lockscript: basic::Script,
    pub tx_fee: u64,
}

pub struct ToCkbSdk {
    rpc_client: HttpRpcClient,
}

impl ToCkbSdk {
    pub fn deposit_request(&self, args: DepositRequestArgs) -> Result<(), ToCKBError> {
        let toCKB_data = ToCKBCellData::new_builder()
            .status(Byte::new(cell::ToCKBStatus::Initial))
            .lot_size(Byte::new(args.lot_size))
            .user_lockscript(args.user_lockscript)
            .build();

        let mut helper = TxHelper::default();

        todo!()
    }
}

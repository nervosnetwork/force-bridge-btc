use crate::switch::ToCKBCellDataTuple;
use crate::utils::types::{Error, ToCKBCellDataView, ToCKBStatus, XChainKind};
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::{bytes::Bytes, prelude::*};
use ckb_std::high_level::{load_cell_capacity, load_witness_args};
use core::result::Result;

pub const COLLATERAL_PERCENT: u64 = 150;

pub fn verify_data(
    input_toCKB_data: &ToCKBCellDataView,
    out_toCKB_data: &ToCKBCellDataView,
) -> Result<(), Error> {
    if input_toCKB_data.kind != out_toCKB_data.kind {
        return Err(Error::LotSizeInvalid);
    }

    let witness_args = load_witness_args(0, Source::Input)?.input_type();
    let price_bytes: Bytes = witness_args.to_opt().unwrap().unpack();
    let price = price_bytes[0];

    let input_capacity = load_cell_capacity(0, Source::GroupInput)?;
    let output_capacity = load_cell_capacity(0, Source::GroupOutput)?;
    let diff_capacity = output_capacity - input_capacity;
    match input_toCKB_data.kind {
        XChainKind::Btc => {
            let btc_lot_size = input_toCKB_data.get_btc_lot_size()?;
            if out_toCKB_data.get_btc_lot_size()? != btc_lot_size {
                return Err(Error::BTCLotSizeMismatch);
            }
        }
        XChainKind::Eth => {
            let eth_lot_size = input_toCKB_data.get_eth_lot_size()?;
            if out_toCKB_data.get_eth_lot_size()? != eth_lot_size {
                return Err(Error::ETHLotSizeMismatch);
            }
        }
    }

    if input_toCKB_data.user_lockscript_hash.as_ref()
        != out_toCKB_data.user_lockscript_hash.as_ref()
    {
        return Err(Error::UserLockScriptHashMismatch);
    }

    if out_toCKB_data.status != ToCKBStatus::Bonded {
        return Err(Error::StatusInvalid);
    }

    Ok(())
}

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    let input_toCKB_data = toCKB_data_tuple
        .0
        .as_ref()
        .expect("inputs contain toCKB cell");
    let output_toCKB_data = toCKB_data_tuple
        .1
        .as_ref()
        .expect("outputs contain toCKB cell");
    verify_data(input_toCKB_data, output_toCKB_data)
}

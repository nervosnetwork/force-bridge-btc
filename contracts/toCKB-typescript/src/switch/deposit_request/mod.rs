use crate::switch::ToCKBCellDataTuple;
use crate::utils::{
    config::PLEDGE,
    types::{Error, ToCKBCellDataView, XChainKind},
};
use ckb_std::{ckb_constants::Source, high_level::load_cell_capacity};
use core::result::Result;

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    let toCKB_data = toCKB_data_tuple
        .1
        .as_ref()
        .expect("outputs contain toCKB cell");
    verify_capacity()?;
    verify_lot_size(toCKB_data)
}

fn verify_capacity() -> Result<(), Error> {
    let capacity = load_cell_capacity(0, Source::GroupOutput)?;
    if capacity != PLEDGE {
        return Err(Error::PledgeInvalid);
    }
    Ok(())
}

fn verify_lot_size(toCKB_data: &ToCKBCellDataView) -> Result<(), Error> {
    if let XChainKind::Btc = toCKB_data.kind {
        if toCKB_data.get_btc_lot_size().is_err() {
            return Err(Error::LotSizeInvalid);
        }
    } else {
        if toCKB_data.get_eth_lot_size().is_err() {
            return Err(Error::LotSizeInvalid);
        }
    }
    Ok(())
}

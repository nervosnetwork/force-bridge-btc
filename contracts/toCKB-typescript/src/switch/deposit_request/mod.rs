use crate::switch::ToCKBCellDataTuple;
use crate::utils::{
    config::PLEDGE,
    tools::*,
    types::{Error, ToCKBCellDataView},
};
use ckb_std::{
    ckb_constants::Source,
    debug,
    high_level::{load_cell_capacity, load_input_out_point},
};
use core::result::Result;
use molecule::prelude::Entity;

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    debug!("begin verify deposit request");
    let toCKB_data = toCKB_data_tuple
        .1
        .as_ref()
        .expect("outputs contain toCKB cell");
    verify_capacity()?;
    debug!("verify capacity success");
    verify_lot_size(toCKB_data)?;
    debug!("verify lot size success");
    verify_cell_id()
}

fn verify_capacity() -> Result<(), Error> {
    let capacity = load_cell_capacity(0, Source::GroupOutput)?;
    if capacity != PLEDGE {
        return Err(Error::PledgeInvalid);
    }
    Ok(())
}

fn verify_lot_size(toCKB_data: &ToCKBCellDataView) -> Result<(), Error> {
    let xchain_kind = toCKB_data.get_xchain_kind();
    match xchain_kind {
        XChainKind::Btc if toCKB_data.get_btc_lot_size().is_ok() => Ok(()),
        XChainKind::Eth if toCKB_data.get_eth_lot_size().is_ok() => Ok(()),
        _ => Err(Error::LotSizeInvalid),
    }
}

fn verify_cell_id() -> Result<(), Error> {
    let expect_cell_id = load_input_out_point(0, Source::Input)?;
    if get_cell_id()?.as_slice() != expect_cell_id.as_slice() {
        return Err(Error::CellIDInvalid);
    }
    Ok(())
}

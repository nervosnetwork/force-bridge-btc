use crate::switch::ToCKBCellDataTuple;
use crate::utils::config::SINCE_N2;
use crate::utils::types::{Error, ToCKBCellDataView};
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::prelude::*;
use ckb_std::high_level::{load_cell, load_cell_capacity, load_input_since, QueryIter};
use core::result::Result;

pub fn verify_since() -> Result<(), Error> {
    let since = load_input_since(0, Source::GroupInput)?;
    if since != SINCE_N2 {
        return Err(Error::InvariantDataMutated);
    }
    Ok(())
}

pub fn verify_capacity(input_toCKB_data: &ToCKBCellDataView) -> Result<(), Error> {
    let pledge_collateral = load_cell_capacity(0, Source::GroupInput)?;
    let outputs = QueryIter::new(load_cell, Source::GroupOutput);
    let mut sum = 0;
    for output in outputs {
        if output.lock().as_bytes().as_ref() == input_toCKB_data.signer_lockscript.as_ref() {
            sum += output.capacity().unpack()
        }
    }

    if sum != pledge_collateral {
        return Err(Error::CapacityMismatch);
    }
    Ok(())
}

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    let input_toCKB_data = toCKB_data_tuple
        .0
        .as_ref()
        .expect("inputs contain toCKB cell");
    verify_since()?;
    verify_capacity(input_toCKB_data)
}

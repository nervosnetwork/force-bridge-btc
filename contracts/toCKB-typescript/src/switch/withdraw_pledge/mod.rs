use crate::switch::ToCKBCellDataTuple;
use crate::utils::config::{PLEDGE, SINCE_WITHDRAW_PLEDGE};
use crate::utils::types::{Error, ToCKBCellDataView};
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::prelude::*;
use ckb_std::high_level::{load_cell, load_input_since, QueryIter};
use core::result::Result;

fn verify_since() -> Result<(), Error> {
    let since = load_input_since(0, Source::GroupInput)?;
    if since != SINCE_WITHDRAW_PLEDGE {
        return Err(Error::InputSinceInvalid);
    }
    Ok(())
}

fn verify_capacity(input_toCKB_data: &ToCKBCellDataView) -> Result<(), Error> {
    let outputs = QueryIter::new(load_cell, Source::Output);
    let mut sum = 0;
    for output in outputs {
        if output.lock().as_bytes().as_ref() == input_toCKB_data.user_lockscript.as_ref() {
            sum += output.capacity().unpack()
        }
    }

    if sum != PLEDGE {
        return Err(Error::CapacityInvalid);
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

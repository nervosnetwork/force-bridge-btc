use crate::switch::ToCKBCellDataTuple;
use crate::utils::{
    config::SINCE_SIGNER_TIMEOUT,
    types::{Error, ToCKBCellDataView},
};
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell_capacity, load_input_since},
};
use core::result::Result;

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    let input_data = toCKB_data_tuple
        .0
        .as_ref()
        .expect("inputs should contain toCKB cell");
    let output_data = toCKB_data_tuple
        .1
        .as_ref()
        .expect("outputs should contain toCKB cell");

    verify_since()?;
    verify_capacity()?;
    verify_data(input_data, output_data)?;
    Ok(())
}

fn verify_since() -> Result<(), Error> {
    let since = load_input_since(0, Source::GroupInput).map_err(|_| Error::InputSinceInvalid)?;
    if since != SINCE_SIGNER_TIMEOUT {
        // check if it is timestamp value
        return Err(Error::InputSinceInvalid);
    }
    Ok(())
}

fn verify_capacity() -> Result<(), Error> {
    let cap_input = load_cell_capacity(0, Source::GroupInput).expect("get input capacity");
    let cap_output = load_cell_capacity(0, Source::GroupOutput).expect("get output capacity");
    if cap_input != cap_output {
        return Err(Error::CapacityInvalid);
    }
    Ok(())
}

fn verify_data(
    input_data: &ToCKBCellDataView,
    output_data: &ToCKBCellDataView,
) -> Result<(), Error> {
    if input_data.get_raw_lot_size() != output_data.get_raw_lot_size()
        || input_data.user_lockscript.as_ref() != output_data.user_lockscript.as_ref()
        || input_data.x_lock_address.as_ref() != output_data.x_lock_address.as_ref()
        || input_data.signer_lockscript.as_ref() != output_data.signer_lockscript.as_ref()
        || input_data.x_unlock_address.as_ref() != output_data.x_unlock_address.as_ref()
        || input_data.redeemer_lockscript.as_ref() != output_data.redeemer_lockscript.as_ref()
    {
        return Err(Error::InvariantDataMutated);
    }

    Ok(())
}

use crate::switch::ToCKBCellDataTuple;
use crate::utils::verifier::{verify_capacity, verify_since_by_value};
use crate::utils::{
    config::SINCE_SIGNER_TIMEOUT,
    types::{Error, ToCKBCellDataView},
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

    verify_since_by_value(SINCE_SIGNER_TIMEOUT)?;
    verify_capacity()?;
    verify_data(input_data, output_data)?;
    Ok(())
}

fn verify_data(
    input_data: &ToCKBCellDataView,
    output_data: &ToCKBCellDataView,
) -> Result<(), Error> {
    if input_data.get_raw_lot_size() != output_data.get_raw_lot_size()
        || input_data.user_lockscript != output_data.user_lockscript
        || input_data.x_lock_address != output_data.x_lock_address
        || input_data.signer_lockscript != output_data.signer_lockscript
        || input_data.x_unlock_address != output_data.x_unlock_address
        || input_data.redeemer_lockscript != output_data.redeemer_lockscript
        || input_data.x_extra != output_data.x_extra
    {
        return Err(Error::InvariantDataMutated);
    }

    Ok(())
}

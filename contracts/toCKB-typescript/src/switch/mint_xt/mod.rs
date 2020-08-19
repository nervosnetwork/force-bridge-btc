use crate::switch::ToCKBCellDataTuple;
use crate::utils::types::Error;
use core::result::Result;

fn verify_data(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    let input_data = toCKB_data_tuple.0.as_ref().expect("should not happen");
    let output_data = toCKB_data_tuple.1.as_ref().expect("should not happen");
    if input_data.kind != output_data.kind
        || input_data.signer_lockscript_hash.as_ref() != output_data.signer_lockscript_hash.as_ref()
        || input_data.user_lockscript_hash.as_ref() != output_data.user_lockscript_hash.as_ref()
        || input_data.x_lock_address.as_ref() != output_data.x_lock_address.as_ref()
    {
        return Err(Error::InvalidDataChange);
    }
    Ok(())
}

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    verify_data(toCKB_data_tuple)?;
    Ok(())
}

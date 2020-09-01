use crate::switch::ToCKBCellDataTuple;
use crate::utils::types::Error;
use core::result::Result;

pub fn verify(_toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    Ok(())
}

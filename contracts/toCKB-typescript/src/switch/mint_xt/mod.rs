use core::result::Result;
use crate::utils::types::Error;
use crate::switch::ToCKBCellDataTuple;

pub fn verify(_toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    Ok(())
}

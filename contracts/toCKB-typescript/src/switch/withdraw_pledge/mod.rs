use crate::switch::ToCKBCellDataTuple;
use crate::utils::common::{verify_capacity_with_value, verify_since_by_value};
use crate::utils::config::{PLEDGE, SINCE_WITHDRAW_PLEDGE};
use crate::utils::types::Error;
use core::result::Result;

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    let input_toCKB_data = toCKB_data_tuple
        .0
        .as_ref()
        .expect("inputs contain toCKB cell");
    verify_since_by_value(SINCE_WITHDRAW_PLEDGE)?;
    verify_capacity_with_value(input_toCKB_data, PLEDGE)
}

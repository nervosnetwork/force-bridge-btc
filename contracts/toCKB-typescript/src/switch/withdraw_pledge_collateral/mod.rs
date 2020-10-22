use crate::switch::ToCKBCellDataTuple;
use crate::utils::common::{verify_capacity_with_value, verify_since_by_value};
use crate::utils::config::SINCE_WITHDRAW_PLEDGE_COLLATERAL;
use crate::utils::types::Error;
use ckb_std::ckb_constants::Source;
use ckb_std::high_level::load_cell_capacity;
use core::result::Result;

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    let input_toCKB_data = toCKB_data_tuple
        .0
        .as_ref()
        .expect("inputs contain toCKB cell");
    verify_since_by_value(SINCE_WITHDRAW_PLEDGE_COLLATERAL)?;
    let pledge_collateral = load_cell_capacity(0, Source::GroupInput)?;
    verify_capacity_with_value(input_toCKB_data, pledge_collateral)
}

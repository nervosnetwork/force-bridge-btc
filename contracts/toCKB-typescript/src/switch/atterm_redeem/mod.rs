use crate::switch::ToCKBCellDataTuple;
use crate::utils::config::SINCE_AT_TERM_REDEEM;
use crate::utils::transaction::{get_sum_sudt_amount, is_XT_typescript};
use crate::utils::types::Error;
use crate::utils::verifier::{verify_capacity, verify_data, verify_since_by_value};
use ckb_std::ckb_constants::Source;
use ckb_std::error::SysError;
use ckb_std::high_level::{load_cell_data, load_cell_lock_hash, load_cell_type};
use core::result::Result;

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    let input_toCKB_data = toCKB_data_tuple
        .0
        .as_ref()
        .expect("inputs contain toCKB cell");
    let output_toCKB_data = toCKB_data_tuple
        .1
        .as_ref()
        .expect("outputs contain toCKB cell");
    verify_capacity()?;
    let lot_size = verify_data(input_toCKB_data, output_toCKB_data)?;
    verify_burn(lot_size)?;
    verify_since_by_value(SINCE_AT_TERM_REDEEM)
}

fn verify_burn(lot_size: u128) -> Result<(), Error> {
    let lock_hash = load_cell_lock_hash(0, Source::GroupInput)?;
    let mut input_sudt_sum: u128 = get_sum_sudt_amount(0, Source::Input, lock_hash.as_ref())?;
    let mut output_sudt_num = get_sum_sudt_amount(0, Source::Output, lock_hash.as_ref())?;
    if input_sudt_sum - output_sudt_num != lot_size {
        return Err(Error::XTBurnInvalid);
    }
    Ok(())
}

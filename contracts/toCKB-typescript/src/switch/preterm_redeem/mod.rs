use crate::switch::ToCKBCellDataTuple;
use crate::utils::config::SIGNER_FEE_RATE;
use crate::utils::transaction::is_XT_typescript;
use crate::utils::types::{Error, ToCKBCellDataView};
use crate::utils::verifier::{verify_capacity, verify_data};
use ckb_std::ckb_constants::Source;
use ckb_std::debug;
use ckb_std::error::SysError;
use ckb_std::high_level::{load_cell_data, load_cell_lock, load_cell_lock_hash, load_cell_type};
use core::result::Result;
use molecule::prelude::*;

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
    verify_burn(lot_size, output_toCKB_data)
}

fn verify_burn(lot_size: u128, out_toCKB_data: &ToCKBCellDataView) -> Result<(), Error> {
    let mut deposit_requestor = false;
    let mut input_sudt_sum: u128 = 0;
    let mut output_sudt_sum: u128 = 0;
    let mut output_sudt_xt_receipt_sum: u128 = 0;
    let mut input_index = 0;

    let lock_hash = load_cell_lock_hash(0, Source::GroupInput)?;

    loop {
        let cell_type = load_cell_type(input_index, Source::Input);
        debug!("input cell type {:?}", cell_type);
        match cell_type {
            Err(SysError::IndexOutOfBound) => break,
            Err(_err) => panic!("iter input return an error"),
            Ok(cell_type) => {
                if !is_XT_typescript(&cell_type, lock_hash.as_ref()) {
                    input_index += 1;
                    continue;
                }

                let lock = load_cell_lock(input_index, Source::Input)?;

                if lock.as_slice() == out_toCKB_data.user_lockscript.as_ref() {
                    deposit_requestor = true;
                }

                let data = load_cell_data(input_index, Source::Input)?;
                let mut buf = [0u8; 16];
                if data.len() == 16 {
                    buf.copy_from_slice(&data);
                    input_sudt_sum += u128::from_le_bytes(buf)
                }
                input_index += 1;
            }
        }
    }

    let mut output_index = 0;
    loop {
        let cell_type = load_cell_type(output_index, Source::Output);
        debug!("output cell type {:?}", cell_type);
        match cell_type {
            Err(SysError::IndexOutOfBound) => break,
            Err(_err) => panic!("iter output return an error"),
            Ok(cell_type) => {
                if !is_XT_typescript(&cell_type, lock_hash.as_ref()) {
                    output_index += 1;
                    continue;
                }

                let lock = load_cell_lock(output_index, Source::Output)?;

                let data = load_cell_data(output_index, Source::Output)?;
                let mut buf = [0u8; 16];
                if data.len() == 16 {
                    buf.copy_from_slice(&data);
                    let output_sudt = u128::from_le_bytes(buf);
                    if lock.as_slice() == out_toCKB_data.user_lockscript.as_ref() {
                        output_sudt_xt_receipt_sum += output_sudt;
                    }
                    output_sudt_sum += output_sudt
                }
                output_index += 1;
            }
        }
    }

    if deposit_requestor && input_sudt_sum - output_sudt_sum != lot_size {
        return Err(Error::XTBurnInvalid);
    }
    if !deposit_requestor {
        let signer_fee: u128 = lot_size * SIGNER_FEE_RATE.0 / SIGNER_FEE_RATE.1;
        debug!("input_sudt_sum {:?}, output_sudt_sum {:?}, output_sudt_xt_receipt_sum {:?}, signer_fee {:?}, lot_size {:?}", input_sudt_sum, output_sudt_sum, output_sudt_xt_receipt_sum, signer_fee, lot_size);
        if (input_sudt_sum - output_sudt_sum != lot_size)
            || (output_sudt_xt_receipt_sum != signer_fee)
        {
            return Err(Error::XTBurnInvalid);
        }
    }
    Ok(())
}

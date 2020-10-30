use crate::switch::ToCKBCellDataTuple;
use crate::utils::config::{CKB_UNITS, PRE_UNDERCOLLATERAL_RATE, XT_CELL_CAPACITY};
use crate::utils::transaction::{get_price, get_sum_sudt_amount, is_XT_typescript, XChainKind};
use crate::utils::types::{Error, ToCKBCellDataView};
use ckb_std::ckb_constants::Source;
use ckb_std::debug;
use ckb_std::error::SysError;
use ckb_std::high_level::{
    load_cell_capacity, load_cell_data, load_cell_lock, load_cell_lock_hash, load_cell_type,
};
use core::result::Result;
use molecule::prelude::*;

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    let input_toCKB_data = toCKB_data_tuple
        .0
        .as_ref()
        .expect("inputs contain toCKB cell");
    let lot_size = match input_toCKB_data.get_xchain_kind() {
        XChainKind::Btc => input_toCKB_data.get_btc_lot_size()?.get_sudt_amount(),
        XChainKind::Eth => input_toCKB_data.get_eth_lot_size()?.get_sudt_amount(),
    };
    verify_collateral_rate(lot_size)?;
    verify_burn(lot_size, input_toCKB_data)
}

fn verify_burn(lot_size: u128, data: &ToCKBCellDataView) -> Result<(), Error> {
    let lock_hash = load_cell_lock_hash(0, Source::GroupInput)?;

    let mut is_signer = false;
    let mut input_sudt_sum: u128 = 0;
    let mut input_index = 0;
    loop {
        let cell_type = load_cell_type(input_index, Source::Input);
        match cell_type {
            Err(SysError::IndexOutOfBound) => break,
            Err(_err) => panic!("iter input return an error"),
            Ok(cell_type) => {
                let lock = load_cell_lock(input_index, Source::Input)?;
                if lock.as_bytes() == data.signer_lockscript {
                    is_signer = true;
                }
                if !is_XT_typescript(&cell_type, lock_hash.as_ref()) {
                    input_index += 1;
                    continue;
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

    if !is_signer {
        return Err(Error::InputSignerInvalid);
    }

    let output_sudt_sum = get_sum_sudt_amount(0, Source::Output, lock_hash.as_ref())?;

    if input_sudt_sum - output_sudt_sum != lot_size {
        return Err(Error::XTBurnInvalid);
    }

    Ok(())
}

fn verify_collateral_rate(lot_size: u128) -> Result<(), Error> {
    let price = get_price()?;
    let input_capacity = load_cell_capacity(0, Source::GroupInput)?;

    debug!(
        "input_capacity {}, price {}, lot_size {} ",
        input_capacity, price, lot_size
    );
    if (100 * (input_capacity - XT_CELL_CAPACITY) as u128 * price) / (CKB_UNITS as u128)
        >= PRE_UNDERCOLLATERAL_RATE as u128 * lot_size
    {
        return Err(Error::UndercollateralInvalid);
    }

    Ok(())
}

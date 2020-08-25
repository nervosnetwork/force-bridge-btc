use crate::switch::ToCKBCellDataTuple;
use crate::utils::config::COLLATERAL_PERCENT;
use crate::utils::types::{Error, ToCKBCellDataView, XChainKind};
use bech32::{self, ToBase32};
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::{bytes::Bytes, prelude::*};
use ckb_std::high_level::{load_cell_capacity, load_witness_args};
use core::result::Result;

pub fn verify_data(
    input_toCKB_data: &ToCKBCellDataView,
    out_toCKB_data: &ToCKBCellDataView,
) -> Result<(), Error> {
    if input_toCKB_data.kind != out_toCKB_data.kind {
        return Err(Error::InvariantDataMutated);
    }

    match input_toCKB_data.kind {
        XChainKind::Btc => {
            if out_toCKB_data.get_btc_lot_size()? != input_toCKB_data.get_btc_lot_size()? {
                return Err(Error::InvariantDataMutated);
            }
            if core::str::from_utf8(out_toCKB_data.x_lock_address.as_ref()).is_err() {
                return Err(Error::XChainAddressInvalid);
            }
            if bech32::decode(core::str::from_utf8(out_toCKB_data.x_lock_address.as_ref()).unwrap())
                .is_err()
            {
                return Err(Error::XChainAddressInvalid);
            }
        }
        XChainKind::Eth => {
            if out_toCKB_data.get_eth_lot_size()? != input_toCKB_data.get_eth_lot_size()? {
                return Err(Error::InvariantDataMutated);
            }
            if out_toCKB_data.x_lock_address.as_ref().len() != 20 {
                return Err(Error::XChainAddressInvalid);
            }
        }
    }

    if input_toCKB_data.user_lockscript_hash.as_ref()
        != out_toCKB_data.user_lockscript_hash.as_ref()
    {
        return Err(Error::InvariantDataMutated);
    }

    Ok(())
}

pub fn verify_collateral(lot_amount: u128) -> Result<(), Error> {
    let witness_args = load_witness_args(0, Source::Input)?.input_type();
    if witness_args.is_none() {
        return Err(Error::InvalidWitness);
    }
    let price_bytes: Bytes = witness_args.to_opt().unwrap().unpack();
    let price: u8 = price_bytes[0].into();

    let input_capacity = load_cell_capacity(0, Source::GroupInput)?;
    let output_capacity = load_cell_capacity(0, Source::GroupOutput)?;
    let diff_capacity = output_capacity - input_capacity;
    if lot_amount * COLLATERAL_PERCENT as u128 * price as u128 != (diff_capacity * 100) as u128 {
        return Err(Error::CollateralInvalid);
    }
    Ok(())
}

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    let input_toCKB_data = toCKB_data_tuple
        .0
        .as_ref()
        .expect("inputs contain toCKB cell");
    let output_toCKB_data = toCKB_data_tuple
        .1
        .as_ref()
        .expect("outputs contain toCKB cell");
    verify_data(input_toCKB_data, output_toCKB_data)

    // todo: verify lot amount
    // let lot_amount = input_toCKB_data.get_lot_amount()?
    // verify_collateral(lot_amount)
}

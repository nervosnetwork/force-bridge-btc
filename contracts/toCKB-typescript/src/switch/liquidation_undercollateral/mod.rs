use crate::switch::ToCKBCellDataTuple;
use crate::utils::config::{CKB_UNITS, LIQUIDATION_COLLATERAL_PERCENT, XT_CELL_CAPACITY};
use crate::utils::{
    tools::{get_xchain_kind, XChainKind},
    types::{Error, ToCKBCellDataView},
};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, prelude::*},
    high_level::{load_cell_capacity, load_witness_args},
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

    let asset_collateral = verify_capacity()? - XT_CELL_CAPACITY;
    verify_data(input_data, output_data)?;
    verify_undercollateral(asset_collateral as u128, input_data)?;

    Ok(())
}

fn verify_capacity() -> Result<u64, Error> {
    let cap_input = load_cell_capacity(0, Source::GroupInput).expect("get input capacity");
    let cap_output = load_cell_capacity(0, Source::GroupOutput).expect("get output capacity");
    if cap_input != cap_output {
        return Err(Error::CapacityInvalid);
    }
    Ok(cap_input)
}

fn verify_data(
    input_data: &ToCKBCellDataView,
    output_data: &ToCKBCellDataView,
) -> Result<(), Error> {
    if input_data.get_raw_lot_size() != output_data.get_raw_lot_size()
        || input_data.user_lockscript != output_data.user_lockscript
        || input_data.x_lock_address != output_data.x_lock_address
        || input_data.signer_lockscript != output_data.signer_lockscript
        || input_data.x_extra != output_data.x_extra
    {
        return Err(Error::InvariantDataMutated);
    }
    Ok(())
}

fn verify_undercollateral(
    asset_collateral: u128,
    input_data: &ToCKBCellDataView,
) -> Result<(), Error> {
    // get lot amount
    let lot_amount: u128 = input_data.get_lot_xt_amount()?;

    // get X/CKB price from witness
    let witness_args = load_witness_args(0, Source::GroupInput)?.input_type();
    if witness_args.is_none() {
        return Err(Error::WitnessInvalid);
    }
    let witness_bytes: Bytes = witness_args.to_opt().unwrap().unpack();
    let mut data = [0u8; 16];
    data.copy_from_slice(witness_bytes.as_ref());
    let price: u128 = u128::from_le_bytes(data);
    if asset_collateral * price * 100
        >= lot_amount * (LIQUIDATION_COLLATERAL_PERCENT as u128) * (CKB_UNITS as u128)
    {
        return Err(Error::UndercollateralInvalid);
    }

    Ok(())
}

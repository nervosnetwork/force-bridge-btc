use crate::switch::ToCKBCellDataTuple;
use crate::utils::config::LIQUIDATION_COLLATERAL_PERCENT;
use crate::utils::{
    tools::{get_xchain_kind, XChainKind},
    types::{Error, ToCKBCellDataView},
};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, prelude::*},
    debug,
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

    let singer_collateral = verify_capacity()?;
    verify_data(input_data, output_data)?;
    verify_undercollateral(singer_collateral as u128, input_data)?;

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
        || input_data.user_lockscript.as_ref() != output_data.user_lockscript.as_ref()
        || input_data.x_lock_address.as_ref() != output_data.x_lock_address.as_ref()
        || input_data.signer_lockscript.as_ref() != output_data.signer_lockscript.as_ref()
    {
        return Err(Error::InvariantDataMutated);
    }
    Ok(())
}

fn verify_undercollateral(
    singer_collateral: u128,
    input_data: &ToCKBCellDataView,
) -> Result<(), Error> {
    // get lot amount
    let lot_amount: u128 = match get_xchain_kind()? {
        XChainKind::Btc => {
            let btc_lot_size = input_data.get_btc_lot_size()?;
            btc_lot_size.get_sudt_amount()
        }
        XChainKind::Eth => {
            let eth_lot_size = input_data.get_eth_lot_size()?;
            eth_lot_size.get_sudt_amount()
        }
    };

    // get X/CKB price from witness
    let witness_args = load_witness_args(0, Source::GroupInput)?.input_type();
    if witness_args.is_none() {
        return Err(Error::WitnessInvalid);
    }
    let witness_bytes: Bytes = witness_args.to_opt().unwrap().unpack();
    let mut data = [0u8; 16];
    data.copy_from_slice(witness_bytes.as_ref());
    let price: u128 = u128::from_le_bytes(data);

    debug!("price: {}", price);

    // LIQUIDATION_COLLATERAL_PERCENT means min liquidation threshold of collateral/lot_amount
    if singer_collateral * price * 100 >= lot_amount * (LIQUIDATION_COLLATERAL_PERCENT as u128) {
        return Err(Error::UndercollateralInvalid);
    }

    Ok(())
}

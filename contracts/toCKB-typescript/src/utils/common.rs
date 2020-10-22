use crate::utils::tools::{verify_btc_address, XChainKind};
use crate::utils::{
    config::{
        LOCK_TYPE_FLAG, METRIC_TYPE_FLAG_MASK, REMAIN_FLAGS_BITS, SINCE_TYPE_TIMESTAMP, VALUE_MASK,
    },
    tools::get_sum_sudt_amount,
    types::{Error, ToCKBCellDataView},
};
use alloc::vec::Vec;
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::prelude::*;
use ckb_std::high_level::{load_cell, load_cell_capacity, load_input_since, QueryIter};

pub fn verify_since() -> Result<u64, Error> {
    let since = load_input_since(0, Source::GroupInput).map_err(|_| Error::InputSinceInvalid)?;

    if since & REMAIN_FLAGS_BITS != 0 // check flags is valid
        || since & LOCK_TYPE_FLAG == 0 // check if it is relative_flag
        || since & METRIC_TYPE_FLAG_MASK != SINCE_TYPE_TIMESTAMP
    // check if it is timestamp value
    {
        return Err(Error::InputSinceInvalid);
    }

    let auction_time = since & VALUE_MASK;
    Ok(auction_time)
}

pub fn verify_since_by_value(value: u64) -> Result<(), Error> {
    let since = load_input_since(0, Source::GroupInput)?;
    if since != value {
        return Err(Error::InputSinceInvalid);
    }
    Ok(())
}

pub fn verify_inputs(
    toCKB_lock_hash: &[u8],
    lot_amount: u128,
    signer_fee: u128,
) -> Result<u128, Error> {
    // inputs[0]: toCKB cell
    // inputs[1:]: XT cell the bidder provides
    // check XT cell on inputs
    let inputs_amount = get_sum_sudt_amount(1, Source::Input, toCKB_lock_hash)?;

    if inputs_amount < lot_amount + signer_fee {
        return Err(Error::FundingNotEnough);
    }
    Ok(inputs_amount)
}

pub fn verify_capacity() -> Result<(), Error> {
    let cap_input = load_cell_capacity(0, Source::GroupInput).expect("get input capacity");
    let cap_output = load_cell_capacity(0, Source::GroupOutput).expect("get output capacity");
    if cap_input != cap_output {
        return Err(Error::CapacityInvalid);
    }
    Ok(())
}

pub fn verify_capacity_with_value(input_data: &ToCKBCellDataView, value: u64) -> Result<(), Error> {
    let sum = QueryIter::new(load_cell, Source::Output)
        .filter(|cell| cell.lock().as_bytes() == input_data.signer_lockscript)
        .map(|cell| cell.capacity().unpack())
        .collect::<Vec<u64>>()
        .into_iter()
        .sum::<u64>();
    if sum < value {
        return Err(Error::CapacityInvalid);
    }
    Ok(())
}

pub fn verify_data(
    input_toCKB_data: &ToCKBCellDataView,
    out_toCKB_data: &ToCKBCellDataView,
) -> Result<u128, Error> {
    let lot_size = match input_toCKB_data.get_xchain_kind() {
        XChainKind::Btc => {
            if out_toCKB_data.get_btc_lot_size()? != input_toCKB_data.get_btc_lot_size()? {
                return Err(Error::InvariantDataMutated);
            }
            verify_btc_address(out_toCKB_data.x_unlock_address.as_ref())?;
            out_toCKB_data.get_btc_lot_size()?.get_sudt_amount()
        }
        XChainKind::Eth => {
            if out_toCKB_data.get_eth_lot_size()? != input_toCKB_data.get_eth_lot_size()? {
                return Err(Error::InvariantDataMutated);
            }
            if out_toCKB_data.x_unlock_address.as_ref().len() != 20 {
                return Err(Error::XChainAddressInvalid);
            }
            out_toCKB_data.get_eth_lot_size()?.get_sudt_amount()
        }
    };
    if input_toCKB_data.user_lockscript != out_toCKB_data.user_lockscript
        || input_toCKB_data.x_lock_address != out_toCKB_data.x_lock_address
        || input_toCKB_data.signer_lockscript != out_toCKB_data.signer_lockscript
        || input_toCKB_data.x_extra != out_toCKB_data.x_extra
    {
        return Err(Error::InvariantDataMutated);
    }
    Ok(lot_size)
}

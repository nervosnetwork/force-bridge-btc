use crate::switch::ToCKBCellDataTuple;
use crate::utils::{
    config::{
        AUCTION_INIT_PERCENT, AUCTION_MAX_TIME, LOCK_TYPE_FLAG, METRIC_TYPE_FLAG_MASK,
        REMAIN_FLAGS_BITS, SINCE_TYPE_TIMESTAMP, VALUE_MASK, XT_CELL_CAPACITY,
    },
    tools::get_sum_sudt_amount,
    types::{Error, ToCKBCellDataView},
};
use ckb_std::{
    ckb_constants::Source,
    debug,
    high_level::{load_cell_capacity, load_cell_lock, load_cell_lock_hash, load_input_since},
};
use core::result::Result;
use molecule::prelude::Entity;

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    debug!("begin verify Auction: SignerTimeout");
    let input_data = toCKB_data_tuple
        .0
        .as_ref()
        .expect("inputs should contain toCKB cell");
    let toCKB_lock_hash = load_cell_lock_hash(0, Source::GroupInput)?;

    let lot_amount = input_data.get_lot_xt_amount()?;

    debug!("begin verify since");
    let auction_time = verify_since()?;
    debug!("begin verify input");
    let inputs_xt_amount = verify_inputs(toCKB_lock_hash.as_ref(), lot_amount)?;
    debug!("begin verify output");
    verify_outputs(
        input_data,
        inputs_xt_amount,
        auction_time,
        toCKB_lock_hash.as_ref(),
        lot_amount,
    )?;

    Ok(())
}

fn verify_since() -> Result<u64, Error> {
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

fn verify_inputs(toCKB_lock_hash: &[u8], lot_amount: u128) -> Result<u128, Error> {
    // inputs[0]: toCKB cell
    // inputs[1:]: XT cell the bidder provides
    // check XT cell on inputs
    let inputs_amount = get_sum_sudt_amount(1, Source::Input, toCKB_lock_hash)?;
    if inputs_amount < lot_amount {
        return Err(Error::FundingNotEnough);
    }
    Ok(inputs_amount)
}

fn verify_outputs(
    input_data: &ToCKBCellDataView,
    inputs_xt_amount: u128,
    auction_time: u64,
    toCKB_lock_hash: &[u8],
    lot_amount: u128,
) -> Result<(), Error> {
    // check bidder cell
    debug!("begin check bidder cell");
    debug!(
        "auction_time: {}, AUCTION_MAX_TIME: {}",
        auction_time, AUCTION_MAX_TIME
    );

    let mut output_index = 0;
    // - 1. check bidder lock
    if load_cell_lock_hash(output_index, Source::Output)? != load_cell_lock_hash(1, Source::Input)?
    {
        return Err(Error::InvalidAuctionBidderCell);
    }
    debug!("1. check bidder lock success! ");

    // expect paying ckb to bidder,trigger and signer
    // cap of toCKB_cell ==  XT_CELL_CAPACITY + to_bidder + to_trigger + to_signer
    let asset_collateral = load_cell_capacity(0, Source::GroupInput)? - XT_CELL_CAPACITY;
    let mut to_bidder = asset_collateral;
    let init_collateral = asset_collateral * AUCTION_INIT_PERCENT as u64 / 100;
    if auction_time < AUCTION_MAX_TIME {
        to_bidder =
            init_collateral + (asset_collateral - init_collateral) / AUCTION_MAX_TIME * auction_time
    }

    let to_trigger = (asset_collateral - to_bidder) / 2;

    // the XT_CELL_CAPACITY is provided by signer when bonding
    let to_signer = asset_collateral - to_bidder - to_trigger + XT_CELL_CAPACITY;

    // - 2. check the repayment to bidder
    // expect bidder_cell_cap == repayment_to_bidder + (cap_sum of inputs_xt_cell)
    if load_cell_capacity(output_index, Source::Output)? <= to_bidder {
        return Err(Error::InvalidAuctionBidderCell);
    }
    debug!("2. check bidder cell capacity success! ");

    debug!("collateral: {}, ", asset_collateral);
    debug!(
        "to_bidder: {}, to_trigger: {}, to_signer:{}",
        to_bidder, to_trigger, to_signer
    );

    // check trigger cell
    if to_trigger > 0 {
        debug!("begin check trigger cell, output_index={}", output_index);
        output_index += 1;
        if to_trigger != load_cell_capacity(output_index, Source::Output)?
            || input_data.liquidation_trigger_lockscript.as_ref()
                != load_cell_lock(output_index, Source::Output)?.as_slice()
        {
            return Err(Error::InvalidTriggerOrSignerCell);
        }
    }

    // check signer cell
    if to_signer > 0 {
        debug!("begin check signer cell, output_index={}", output_index);
        output_index += 1;
        if to_signer != load_cell_capacity(output_index, Source::Output)?
            || input_data.signer_lockscript.as_ref()
                != load_cell_lock(output_index, Source::Output)?.as_slice()
        {
            return Err(Error::InvalidTriggerOrSignerCell);
        }
    }

    // - check XT change, make sure inputs_sudt_amount == outputs_sudt_amount
    let outputs_xt_amount = get_sum_sudt_amount(output_index + 1, Source::Output, toCKB_lock_hash)?;
    if inputs_xt_amount - outputs_xt_amount != lot_amount {
        return Err(Error::XTAmountInvalid);
    }
    debug!("make sure inputs_sudt_amount == outputs_sudt_amount success!");

    Ok(())
}

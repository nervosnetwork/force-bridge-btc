use crate::switch::ToCKBCellDataTuple;
use crate::utils::{
    config::{AUCTION_INIT_PERCENT, AUCTION_MAX_TIME, SIGNER_FEE_RATE, UDT_LEN, XT_CELL_CAPACITY},
    transaction::{get_sum_sudt_amount, is_XT_typescript},
    types::{Error, ToCKBCellDataView},
    verifier::{verify_auction_inputs, verify_since},
};
use ckb_std::high_level::load_cell_capacity;
use ckb_std::{
    ckb_constants::Source,
    debug,
    high_level::{load_cell_data, load_cell_lock, load_cell_lock_hash, load_cell_type},
};
use core::result::Result;
use molecule::prelude::Entity;

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    debug!("begin verify Auction: FaultyWhenWarranty");
    let input_data = toCKB_data_tuple
        .0
        .as_ref()
        .expect("inputs should contain toCKB cell");
    let toCKB_lock_hash = load_cell_lock_hash(0, Source::GroupInput)?;

    let lot_amount = input_data.get_lot_xt_amount()?;
    let signer_fee = lot_amount * SIGNER_FEE_RATE.0 / SIGNER_FEE_RATE.1;

    debug!("begin verify since");
    let auction_time = verify_since()?;
    debug!("begin verify input");
    let inputs_xt_amount = verify_auction_inputs(toCKB_lock_hash.as_ref(), lot_amount, signer_fee)?;
    debug!("begin verify output");
    verify_outputs(
        input_data,
        inputs_xt_amount,
        auction_time,
        toCKB_lock_hash.as_ref(),
        lot_amount,
        signer_fee,
    )?;

    Ok(())
}

fn verify_outputs(
    input_data: &ToCKBCellDataView,
    inputs_xt_amount: u128,
    auction_time: u64,
    toCKB_lock_hash: &[u8],
    lot_amount: u128,
    signer_fee: u128,
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
    let to_trigger = asset_collateral - to_bidder;

    // - 2. check the repayment to bidder
    // expect bidder_cell_cap == repayment_to_bidder + (cap_sum of inputs_xt_cell)
    if load_cell_capacity(output_index, Source::Output)? <= to_bidder {
        return Err(Error::InvalidAuctionBidderCell);
    }
    debug!("2. check bidder repayment success! ");

    debug!("collateral: {}, ", asset_collateral);
    debug!("to_bidder: {}, to_trigger: {}", to_bidder, to_trigger);

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

    // check XT cell
    output_index += 1;
    debug!("begin check XT cell, output_index={}", output_index);
    // - 1. check if lock is deposit Requester's lockscript
    let script = load_cell_lock(output_index, Source::Output)?;
    if script.as_slice() != input_data.user_lockscript.as_ref() {
        return Err(Error::InvalidAuctionXTCell);
    }
    debug!("1. check XT lock is redeemer's lock success!");

    // - 2. check if typescript is sudt typescript
    let script = load_cell_type(output_index, Source::Output)?;

    if !is_XT_typescript(&script, toCKB_lock_hash) {
        return Err(Error::InvalidAuctionXTCell);
    }
    debug!("2. check XT type is sudt typescript success!");

    // - 3. check XT amount == signer fee
    let cell_data = load_cell_data(output_index, Source::Output)?;
    let mut data = [0u8; UDT_LEN];
    data.copy_from_slice(&cell_data);
    let to_user_amount = u128::from_le_bytes(data);
    debug!(
        "to_redeemer_amount: {}, signer_fee: {}",
        to_user_amount, signer_fee
    );
    if to_user_amount != signer_fee {
        return Err(Error::InvalidAuctionXTCell);
    }
    debug!("3. check XT amount is signer_fee success!");

    // - 4. check XT cell capacity
    let redeemer_XT_cell_cap = load_cell_capacity(output_index, Source::Output)?;
    if redeemer_XT_cell_cap != XT_CELL_CAPACITY {
        return Err(Error::InvalidAuctionXTCell);
    }
    debug!("4. check XT cell capacity success!");

    // check XT change, make sure inputs_sudt_amount == outputs_sudt_amount
    let outputs_xt_amount = get_sum_sudt_amount(output_index + 1, Source::Output, toCKB_lock_hash)?;
    if inputs_xt_amount - outputs_xt_amount != lot_amount + signer_fee {
        return Err(Error::XTAmountInvalid);
    }

    Ok(())
}

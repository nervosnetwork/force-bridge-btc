use crate::switch::ToCKBCellDataTuple;
use crate::utils::{
    config::{
        AUCTION_INIT_PERCENT, AUCTION_MAX_TIME, LOCK_TYPE_FLAG, METRIC_TYPE_FLAG_MASK,
        REMAIN_FLAGS_BITS, SINCE_TYPE_TIMESTAMP, UDT_LEN, VALUE_MASK, XT_CELL_CAPACITY,
    },
    tools::{get_xchain_kind, is_XT_typescript, XChainKind},
    types::{Error, ToCKBCellDataView},
};
use ckb_std::{
    ckb_constants::Source,
    debug,
    high_level::{
        load_cell_capacity, load_cell_data, load_cell_lock, load_cell_lock_hash, load_cell_type,
        load_input_since,
    },
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

    debug!("begin verify since");
    let auction_time = verify_since()?;
    debug!("begin verify input");
    verify_inputs(toCKB_lock_hash.as_ref(), lot_amount)?;
    debug!("begin verify output");
    verify_outputs(input_data, auction_time)?;

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

fn verify_inputs(toCKB_lock_hash: &[u8], lot_amount: u128) -> Result<(), Error> {
    // check XT cell on inputs
    let mut input_index = 1;
    let mut sum_amount = 0;
    loop {
        let res = load_cell_type(input_index, Source::Input);
        if res.is_err() {
            break;
        }
        let script = res.unwrap();
        if script.is_none() || !is_XT_typescript(&script.unwrap(), toCKB_lock_hash) {
            return Err(Error::InvalidInputs);
        }

        let cell_data = load_cell_data(input_index, Source::Input)?;
        let mut data = [0u8; UDT_LEN];
        data.copy_from_slice(&cell_data);
        let amount = u128::from_le_bytes(data);
        sum_amount += amount;

        input_index += 1;
    }

    if sum_amount < lot_amount {
        return Err(Error::FundingNotEnough);
    }
    Ok(())
}

fn verify_outputs(input_data: &ToCKBCellDataView, auction_time: u64) -> Result<(), Error> {
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
    let collateral = load_cell_capacity(0, Source::GroupInput)? - XT_CELL_CAPACITY;
    let mut to_bidder = collateral;
    let init_collateral = collateral * AUCTION_INIT_PERCENT as u64 / 100;
    if auction_time == 0 {
        to_bidder = init_collateral
    } else if auction_time < AUCTION_MAX_TIME {
        to_bidder =
            init_collateral + (collateral - init_collateral) * auction_time / AUCTION_MAX_TIME
    }
    let to_trigger = collateral - to_bidder + XT_CELL_CAPACITY;

    // - 2. check the repayment to bidder
    // expect bidder_cell_cap == repayment_to_bidder + (cap_sum of inputs_xt_cell)
    if load_cell_capacity(output_index, Source::Output)? <= to_bidder {
        return Err(Error::InvalidAuctionBidderCell);
    }
    debug!("2. check bidder repayment success! ");

    debug!("collateral: {}, ", collateral);
    debug!("to_bidder: {}, to_trigger: {}", to_bidder, to_trigger);

    // check trigger cell
    debug!("begin check trigger cell, output_index={}", output_index);
    output_index += 1;
    if to_trigger != load_cell_capacity(output_index, Source::Output)?
        || input_data.liquidation_trigger_lockscript.as_ref()
            != load_cell_lock(output_index, Source::Output)?.as_slice()
    {
        return Err(Error::InvalidTriggerOrSignerCell);
    }

    // check no other output cell
    output_index += 1;
    if load_cell_capacity(output_index, Source::Output).is_ok() {
        return Err(Error::InvalidOutputsNum);
    }
    debug!("4. check no other output cell success!");

    Ok(())
}

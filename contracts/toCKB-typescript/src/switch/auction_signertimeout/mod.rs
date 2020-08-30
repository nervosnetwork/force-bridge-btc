use crate::switch::ToCKBCellDataTuple;
use crate::utils::{
    config::{
        AUCTION_INIT_PERCENT, AUCTION_MAX_TIME, LOCK_TYPE_FLAG, METRIC_TYPE_FLAG_MASK,
        REMAIN_FLAGS_BITS, SINCE_TYPE_TIMESTAMP, UDT_LEN, VALUE_MASK,
    },
    tools::{get_xchain_kind, is_XT_typescript, XChainKind},
    types::{Error, ToCKBCellDataView},
};
use ckb_std::{
    ckb_constants::Source,
    high_level::{
        load_cell_capacity, load_cell_data, load_cell_lock, load_cell_lock_hash, load_cell_type,
        load_input_since,
    },
};
use core::result::Result;
use molecule::prelude::Entity;

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    let input_data = toCKB_data_tuple
        .0
        .as_ref()
        .expect("inputs should contain toCKB cell");
    let toCKB_lock_hash = load_cell_lock_hash(0, Source::GroupInput)?;

    let auction_time = verify_since()?;
    verify_input(toCKB_lock_hash.as_ref())?;
    verify_output(input_data, auction_time, toCKB_lock_hash.as_ref())?;

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

    let auction_time = since | VALUE_MASK;
    Ok(auction_time)
}

fn verify_input(toCKB_lock_hash: &[u8]) -> Result<(), Error> {
    // check XT cell on inputs
    let script = load_cell_type(1, Source::Input)?.unwrap();
    if !is_XT_typescript(script, toCKB_lock_hash) {
        return Err(Error::InvalidAuctionXTCell);
    }
    Ok(())
}

fn verify_output(
    input_data: &ToCKBCellDataView,
    auction_time: u64,
    toCKB_lock_hash: &[u8],
) -> Result<(), Error> {
    // check bidder cell
    let mut output_index = 0;
    // 1. check bidder lock
    if load_cell_lock_hash(1, Source::Input)? != load_cell_lock_hash(output_index, Source::Output)?
    {
        return Err(Error::InvalidAuctionBidderCell);
    }

    // expect paying ckb to bidder,trigger and signer
    let collateral = load_cell_capacity(0, Source::GroupInput)?;
    let mut to_bidder = collateral;
    let init_collateral = collateral * AUCTION_INIT_PERCENT as u64 / 100;
    if auction_time == 0 {
        to_bidder = init_collateral
    } else if auction_time < AUCTION_MAX_TIME {
        to_bidder =
            init_collateral + (collateral - init_collateral) * auction_time / AUCTION_MAX_TIME
    }

    let to_trigger = (collateral - to_bidder) / 2;
    let to_signer = collateral - to_bidder - to_trigger;

    // 2. check the repayment to bidder
    if to_bidder != load_cell_capacity(output_index, Source::Output)? {
        return Err(Error::InvalidAuctionBidderCell);
    }

    // check trigger cell
    if to_trigger > 0 {
        output_index += 1;
        if to_trigger != load_cell_capacity(output_index, Source::Output)?
            || input_data.liquidation_trigger_lockscript.as_ref()
                != load_cell_lock(output_index, Source::Output)?
                    .as_bytes()
                    .as_ref()
        {
            return Err(Error::InvalidTriggerOrSignerCell);
        }
    }

    // check signer cell
    if to_signer > 0 {
        output_index += 1;
        if to_signer != load_cell_capacity(output_index, Source::Output)?
            || input_data.signer_lockscript.as_ref()
                != load_cell_lock(output_index, Source::Output)?
                    .as_bytes()
                    .as_ref()
        {
            return Err(Error::InvalidTriggerOrSignerCell);
        }
    }

    // check XT cell
    output_index += 1;
    // 1. check if lock is redeemer's lockscript
    let script = load_cell_lock(output_index, Source::Output)?;
    if script.as_bytes().as_ref() != input_data.redeemer_lockscript.as_ref() {
        return Err(Error::InvalidAuctionXTCell);
    }

    // 2. check if typescript is sudt typescript
    let script = load_cell_type(output_index, Source::Output)?.expect("sudt typescript must exist");
    if !is_XT_typescript(script, toCKB_lock_hash) {
        return Err(Error::InvalidAuctionXTCell);
    }

    // 3. check XT amount
    let cell_data = load_cell_data(output_index, Source::Output)?;
    let mut data = [0u8; UDT_LEN];
    data.copy_from_slice(&cell_data);
    let to_redeemer_amount = u128::from_le_bytes(data);

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

    if to_redeemer_amount != lot_amount {
        return Err(Error::InvalidAuctionXTCell);
    }

    Ok(())
}

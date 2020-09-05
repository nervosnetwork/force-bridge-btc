use super::ToCKBCellData;
use crate::toCKB_typescript::utils::{
    config::*,
    types::{generated::basic, Error, ToCKBStatus},
};
use crate::*;
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_tool::ckb_types::{
    bytes::Bytes,
    core::{TransactionBuilder, TransactionView},
    packed::*,
    prelude::*,
};
use ckb_tool::{ckb_error::assert_error_eq, ckb_script::ScriptError};
use molecule::prelude::*;

const MAX_CYCLES: u64 = 10_000_000;

#[test]
fn test_correct_tx_max_time() {
    let since_max_auction_time = LOCK_TYPE_FLAG | SINCE_TYPE_TIMESTAMP | AUCTION_MAX_TIME;
    let (context, tx) = build_test_context(
        3_750_000 * CKB_UNITS,
        3_750_000 * CKB_UNITS,
        0,
        XT_CELL_CAPACITY,
        since_max_auction_time,
        25_000_000,
    );

    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_correct_tx_trigger() {
    let auction_time = 2 * 24 * 3600;
    let since = LOCK_TYPE_FLAG | SINCE_TYPE_TIMESTAMP | auction_time;
    let collateral = 3_750_000 * CKB_UNITS;
    let bidder_cap = {
        let init_repayment = collateral * AUCTION_INIT_PERCENT as u64 / 100;
        init_repayment + (collateral - init_repayment) / AUCTION_MAX_TIME * auction_time
    };

    let trigger_cap = (collateral - bidder_cap) / 2;

    let (context, tx) = build_test_context(
        collateral,
        bidder_cap,
        trigger_cap,
        XT_CELL_CAPACITY,
        since,
        25_000_000,
    );

    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_wrong_since() {
    let since = LOCK_TYPE_FLAG | AUCTION_MAX_TIME;
    let (context, tx) = build_test_context(
        3_750_000 * CKB_UNITS,
        3_750_000 * CKB_UNITS,
        0,
        XT_CELL_CAPACITY,
        since,
        25_000_000,
    );

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::InputSinceInvalid as i8)
    );
}

#[test]
fn test_wrong_XT_amount() {
    let since = LOCK_TYPE_FLAG | SINCE_TYPE_TIMESTAMP | AUCTION_MAX_TIME;
    let wrong_lot_amount: u128 = 999;
    let (context, tx) = build_test_context(
        3_750_000 * CKB_UNITS,
        3_750_000 * CKB_UNITS,
        0,
        XT_CELL_CAPACITY,
        since,
        wrong_lot_amount,
    );

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::FundingNotEnough as i8)
    );
}

#[test]
fn test_wrong_XT_cap() {
    let since = LOCK_TYPE_FLAG | SINCE_TYPE_TIMESTAMP | AUCTION_MAX_TIME;
    // let (context, tx) = build_test_context(3_750_000 * CKB_UNITS, 3_750_000 * CKB_UNITS, 0, wrong_xt_cap, since, 25_000_000);
    let (context, tx) = build_test_context(
        3_750_000 * CKB_UNITS,
        3_750_000 * CKB_UNITS,
        0,
        XT_CELL_CAPACITY + 1,
        since,
        25_000_000,
    );

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::InvalidAuctionXTCell as i8)
    );
}

#[test]
fn test_wrong_bidder_cell() {
    let since = LOCK_TYPE_FLAG | SINCE_TYPE_TIMESTAMP | (2 * 24 * 3600);
    let collateral = 3_750_000 * CKB_UNITS;
    let wrong_bidder_capacity = 100;
    let (context, tx) = build_test_context(
        collateral,
        wrong_bidder_capacity,
        0,
        XT_CELL_CAPACITY,
        since,
        25_000_000,
    );

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::InvalidAuctionBidderCell as i8)
    );
}

#[test]
fn test_wrong_trigger() {
    let time = 2 * 24 * 3600;
    let since = LOCK_TYPE_FLAG | SINCE_TYPE_TIMESTAMP | time;
    let collateral = 3_750_000 * CKB_UNITS;
    let bidder_cap = {
        let init_repayment = collateral * AUCTION_INIT_PERCENT as u64 / 100;
        init_repayment + (collateral - init_repayment) / AUCTION_MAX_TIME * time
    };

    let (context, tx) = build_test_context(
        collateral,
        bidder_cap,
        0,
        XT_CELL_CAPACITY,
        since,
        25_000_000,
    );

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::InvalidTriggerOrSignerCell as i8)
    );
}

fn build_test_context(
    asset_collateral: u64,
    bidder_capacity: u64,
    trigger_capacity: u64,
    xt_capacity: u64,
    since: u64,
    lot_amount: u128,
) -> (Context, TransactionView) {
    // deploy contract
    let mut context = Context::default();
    let toCKB_typescript_bin: Bytes = Loader::default().load_binary("toCKB-typescript");
    let toCKB_typescript_out_point = context.deploy_cell(toCKB_typescript_bin);
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let sudt_typescript_bin: Bytes = load_dep_binary("simple_udt");
    let sudt_typescript_out_point = context.deploy_cell(sudt_typescript_bin.clone());

    // prepare scripts
    let toCKB_typescript = context
        .build_script(&toCKB_typescript_out_point, [1u8; 1].to_vec().into())
        .expect("script");
    let toCKB_typescript_dep = CellDep::new_builder()
        .out_point(toCKB_typescript_out_point)
        .build();

    let always_success_lockscript = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let always_success_lockscript_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let lock_hash: [u8; 32] = always_success_lockscript.calc_script_hash().unpack();
    let sudt_script_args: Bytes = lock_hash.to_vec().into();
    let sudt_typescript = context
        .build_script(&sudt_typescript_out_point, sudt_script_args)
        .expect("sudt script");
    let sudt_typescript_dep = CellDep::new_builder()
        .out_point(sudt_typescript_out_point)
        .build();

    // prepare inputs
    // 1. toCKB cell
    let always_lockscript =
        basic::Script::from_slice(always_success_lockscript.as_slice()).unwrap();
    let input_toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(ToCKBStatus::SignerTimeout as u8))
        .lot_size(Byte::new(1u8))
        .redeemer_lockscript(always_lockscript.clone())
        .liquidation_trigger_lockscript(always_lockscript.clone())
        .signer_lockscript(always_lockscript.clone())
        .build();

    let mut inputs = vec![];
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity((asset_collateral + XT_CELL_CAPACITY).pack())
            .lock(always_success_lockscript.clone())
            .type_(Some(toCKB_typescript.clone()).pack())
            .build(),
        input_toCKB_data.as_bytes(),
    );
    inputs.push(
        CellInput::new_builder()
            .previous_output(input_out_point)
            .since(since.pack())
            .build(),
    );

    // 2. XT cell
    let data = &lot_amount.to_le_bytes()[..];
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(XT_CELL_CAPACITY.pack())
            .lock(always_success_lockscript.clone())
            .type_(Some(sudt_typescript.clone()).pack())
            .build(),
        Bytes::copy_from_slice(data),
    );
    inputs.push(
        CellInput::new_builder()
            .previous_output(input_out_point)
            .build(),
    );

    // prepare outputs
    // 1.bidder cell
    let mut outputs = vec![CellOutput::new_builder()
        .capacity((bidder_capacity + XT_CELL_CAPACITY).pack())
        .lock(always_success_lockscript.clone())
        .build()];
    let mut outputs_data = vec![Bytes::new(); 1];

    // 2. trigger and signer
    if trigger_capacity > 0 {
        outputs.push(
            CellOutput::new_builder()
                .capacity(trigger_capacity.pack())
                .lock(always_success_lockscript.clone())
                .build(),
        );
        outputs_data.push(Bytes::new());
    }
    let signer_capacity = asset_collateral - bidder_capacity - trigger_capacity;
    if signer_capacity > 0 {
        outputs.push(
            CellOutput::new_builder()
                .capacity(signer_capacity.pack())
                .lock(always_success_lockscript.clone())
                .build(),
        );
        outputs_data.push(Bytes::new());
    }

    // 3.XT cell
    outputs.push(
        CellOutput::new_builder()
            .capacity(xt_capacity.pack())
            .lock(always_success_lockscript.clone())
            .type_(Some(sudt_typescript.clone()).pack())
            .build(),
    );
    outputs_data.push(Bytes::copy_from_slice(data));

    // build transaction
    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(toCKB_typescript_dep)
        .cell_dep(always_success_lockscript_dep)
        .cell_dep(sudt_typescript_dep)
        .build();
    let tx = context.complete_tx(tx);

    (context, tx)
}

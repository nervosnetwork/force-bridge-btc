use super::{Error, Script, ToCKBCellData};
use crate::toCKB_typescript::utils::config::*;
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
use tockb_types::{generated::*, tockb_cell_data::*};

const MAX_CYCLES: u64 = 10_000_000;

const ETH_BURN: u128 = 250_000_000_000_000_000;
const BTC_BURN: u128 = 25_000_000;

#[test]
fn test_correct_tx_eth() {
    let correct_eth_address = basic::Bytes::new_builder()
        .set([Byte::new(1u8); 20].to_vec())
        .build();
    let toCKB_data = build_to_ckb_data(
        1,
        Script::new_builder().build(),
        correct_eth_address.clone(),
        Script::new_builder().build(),
        correct_eth_address,
        Script::new_builder().build(),
        build_extra(2),
    );

    let (context, tx) = build_test_context(
        2,
        ETH_BURN,
        SINCE_AT_TERM_REDEEM,
        toCKB_data.as_bytes(),
        11000,
        false,
    );
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_correct_tx_btc() {
    let correct_btc_address = basic::Bytes::new_builder()
        .set(
            "bc1qzulv8gfw9zd3qtuwmnqafmxnkkuf8cku8mf3ah"
                .as_bytes()
                .iter()
                .map(|c| Byte::new(*c))
                .collect::<Vec<_>>()
                .into(),
        )
        .build();
    let toCKB_data = build_to_ckb_data(
        1,
        Script::new_builder().build(),
        correct_btc_address.clone(),
        Script::new_builder().build(),
        correct_btc_address,
        Script::new_builder().build(),
        build_extra(1),
    );

    let (context, tx) = build_test_context(
        1,
        BTC_BURN,
        SINCE_AT_TERM_REDEEM,
        toCKB_data.as_bytes(),
        11000,
        false,
    );
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_wrong_tx_eth_address_invalid() {
    let correct_eth_address = basic::Bytes::new_builder()
        .set([Byte::new(1u8); 20].to_vec())
        .build();
    let wrong_eth_address = basic::Bytes::new_builder()
        .set([Byte::new(1u8); 21].to_vec())
        .build();
    let toCKB_data = build_to_ckb_data(
        1,
        Script::new_builder().build(),
        correct_eth_address,
        Script::new_builder().build(),
        wrong_eth_address,
        Script::new_builder().build(),
        build_extra(2),
    );

    let (context, tx) = build_test_context(
        2,
        ETH_BURN,
        SINCE_AT_TERM_REDEEM,
        toCKB_data.as_bytes(),
        11000,
        false,
    );
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::XChainAddressInvalid as i8)
    );
}

#[test]
fn test_wrong_tx_btc_address_invalid() {
    let correct_btc_address = basic::Bytes::new_builder()
        .set(
            "bc1qzulv8gfw9zd3qtuwmnqafmxnkkuf8cku8mf3ah"
                .as_bytes()
                .iter()
                .map(|c| Byte::new(*c))
                .collect::<Vec<_>>()
                .into(),
        )
        .build();
    let wrong_btc_address = basic::Bytes::new_builder()
        .set(
            "bc1qq2pw0kr5yhz3xcs978desw5anfmtwynutwq8quz00"
                .as_bytes()
                .iter()
                .map(|c| Byte::new(*c))
                .collect::<Vec<_>>()
                .into(),
        )
        .build();
    let toCKB_data = build_to_ckb_data(
        1,
        Script::new_builder().build(),
        correct_btc_address,
        Script::new_builder().build(),
        wrong_btc_address,
        Script::new_builder().build(),
        build_extra(1),
    );

    let (context, tx) = build_test_context(
        1,
        BTC_BURN,
        SINCE_AT_TERM_REDEEM,
        toCKB_data.as_bytes(),
        11000,
        false,
    );
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::XChainAddressInvalid as i8)
    );
}

#[test]
fn test_wrong_tx_lot_size_mutated() {
    let correct_eth_address = basic::Bytes::new_builder()
        .set([Byte::new(1u8); 20].to_vec())
        .build();
    let toCKB_data = build_to_ckb_data(
        2,
        Script::new_builder().build(),
        correct_eth_address.clone(),
        Script::new_builder().build(),
        correct_eth_address,
        Script::new_builder().build(),
        build_extra(2),
    );

    let (context, tx) = build_test_context(
        2,
        ETH_BURN,
        SINCE_AT_TERM_REDEEM,
        toCKB_data.as_bytes(),
        11000,
        false,
    );
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::InvariantDataMutated as i8)
    );
}

#[test]
fn test_wrong_tx_user_lock_script_mutated() {
    let correct_eth_address = basic::Bytes::new_builder()
        .set([Byte::new(1u8); 20].to_vec())
        .build();
    let mutated_user_lock_script = Script::new_builder().hash_type(Byte::new(2)).build();
    let toCKB_data = build_to_ckb_data(
        1,
        mutated_user_lock_script,
        correct_eth_address.clone(),
        Script::new_builder().build(),
        correct_eth_address,
        Script::new_builder().build(),
        build_extra(2),
    );

    let (context, tx) = build_test_context(
        2,
        ETH_BURN,
        SINCE_AT_TERM_REDEEM,
        toCKB_data.as_bytes(),
        11000,
        false,
    );
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::InvariantDataMutated as i8)
    );
}

#[test]
fn test_wrong_tx_user_lock_address_mutated() {
    let correct_eth_address = basic::Bytes::new_builder()
        .set([Byte::new(1u8); 20].to_vec())
        .build();
    let mutated_lock_address = basic::Bytes::new_builder()
        .set([Byte::new(2u8); 20].to_vec())
        .build();
    let toCKB_data = build_to_ckb_data(
        1,
        Script::new_builder().build(),
        mutated_lock_address,
        Script::new_builder().build(),
        correct_eth_address,
        Script::new_builder().build(),
        build_extra(2),
    );

    let (context, tx) = build_test_context(
        2,
        ETH_BURN,
        SINCE_AT_TERM_REDEEM,
        toCKB_data.as_bytes(),
        11000,
        false,
    );
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::InvariantDataMutated as i8)
    );
}

#[test]
fn test_wrong_tx_user_signer_script_mutated() {
    let correct_eth_address = basic::Bytes::new_builder()
        .set([Byte::new(1u8); 20].to_vec())
        .build();
    let mutated_signer_script = Script::new_builder().hash_type(Byte::new(2u8)).build();
    let toCKB_data = build_to_ckb_data(
        1,
        Script::new_builder().build(),
        correct_eth_address.clone(),
        mutated_signer_script,
        correct_eth_address,
        Script::new_builder().build(),
        build_extra(2),
    );

    let (context, tx) = build_test_context(
        2,
        ETH_BURN,
        SINCE_AT_TERM_REDEEM,
        toCKB_data.as_bytes(),
        11000,
        false,
    );
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::InvariantDataMutated as i8)
    );
}

#[test]
fn test_wrong_tx_extra_mutated() {
    let correct_eth_address = basic::Bytes::new_builder()
        .set([Byte::new(1u8); 20].to_vec())
        .build();
    let eth_extra = EthExtra::new_builder()
        .dummy(correct_eth_address.clone())
        .build();
    let x_extra = XExtraUnion::EthExtra(eth_extra);
    let extra = XExtra::new_builder().set(x_extra).build();

    let toCKB_data = build_to_ckb_data(
        1,
        Script::new_builder().build(),
        correct_eth_address.clone(),
        Script::new_builder().build(),
        correct_eth_address,
        Script::new_builder().build(),
        extra,
    );

    let (context, tx) = build_test_context(
        2,
        ETH_BURN,
        SINCE_AT_TERM_REDEEM,
        toCKB_data.as_bytes(),
        11000,
        false,
    );
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::InvariantDataMutated as i8)
    );
}

#[test]
fn test_wrong_tx_output_has_xt_cell() {
    let correct_eth_address = basic::Bytes::new_builder()
        .set([Byte::new(1u8); 20].to_vec())
        .build();
    let toCKB_data = build_to_ckb_data(
        1,
        Script::new_builder().build(),
        correct_eth_address.clone(),
        Script::new_builder().build(),
        correct_eth_address,
        Script::new_builder().build(),
        build_extra(2),
    );

    let (context, tx) = build_test_context(
        2,
        ETH_BURN,
        SINCE_AT_TERM_REDEEM,
        toCKB_data.as_bytes(),
        11000,
        true,
    );
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::XTBurnInvalid as i8)
    );
}

#[test]
fn test_wrong_tx_xt_burn_invalid() {
    let correct_eth_address = basic::Bytes::new_builder()
        .set([Byte::new(1u8); 20].to_vec())
        .build();
    let toCKB_data = build_to_ckb_data(
        1,
        Script::new_builder().build(),
        correct_eth_address.clone(),
        Script::new_builder().build(),
        correct_eth_address,
        Script::new_builder().build(),
        build_extra(2),
    );

    let (context, tx) = build_test_context(
        2,
        ETH_BURN + 1,
        SINCE_AT_TERM_REDEEM,
        toCKB_data.as_bytes(),
        11000,
        false,
    );
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::XTBurnInvalid as i8)
    );
}
#[test]
fn test_wrong_tx_since_invalid() {
    let correct_eth_address = basic::Bytes::new_builder()
        .set([Byte::new(1u8); 20].to_vec())
        .build();
    let toCKB_data = build_to_ckb_data(
        1,
        Script::new_builder().build(),
        correct_eth_address.clone(),
        Script::new_builder().build(),
        correct_eth_address,
        Script::new_builder().build(),
        build_extra(2),
    );

    let (context, tx) = build_test_context(
        2,
        ETH_BURN,
        SINCE_AT_TERM_REDEEM + 1,
        toCKB_data.as_bytes(),
        11000,
        false,
    );
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::InputSinceInvalid as i8)
    );
}

#[test]
fn test_wrong_tx_capacity_mismatch() {
    let correct_eth_address = basic::Bytes::new_builder()
        .set([Byte::new(1u8); 20].to_vec())
        .build();
    let toCKB_data = build_to_ckb_data(
        1,
        Script::new_builder().build(),
        correct_eth_address.clone(),
        Script::new_builder().build(),
        correct_eth_address,
        Script::new_builder().build(),
        build_extra(2),
    );

    let (context, tx) = build_test_context(
        2,
        ETH_BURN,
        SINCE_AT_TERM_REDEEM,
        toCKB_data.as_bytes(),
        10000,
        false,
    );
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::CapacityInvalid as i8)
    );
}

fn build_extra(kind: u8) -> XExtra {
    let extra = match kind {
        1 => {
            let btc_extra = BtcExtra::new_builder().build();
            let x_extra = XExtraUnion::BtcExtra(btc_extra);
            XExtra::new_builder().set(x_extra).build()
        }
        2 => {
            let eth_extra = EthExtra::new_builder().build();
            let x_extra = XExtraUnion::EthExtra(eth_extra);
            XExtra::new_builder().set(x_extra).build()
        }
        _ => {
            let btc_extra = BtcExtra::new_builder().build();
            let x_extra = XExtraUnion::BtcExtra(btc_extra);
            XExtra::new_builder().set(x_extra).build()
        }
    };
    extra
}

fn build_to_ckb_data(
    lot_size: u8,
    user_lockscript: Script,
    lock_address: basic::Bytes,
    signer_lockscript: Script,
    unlock_address: basic::Bytes,
    redeemer_lockscript: Script,
    extra: XExtra,
) -> ToCKBCellData {
    ToCKBCellData::new_builder()
        .status(Byte::new(4u8))
        .lot_size(Byte::new(lot_size))
        .user_lockscript(user_lockscript)
        .x_lock_address(lock_address)
        .signer_lockscript(signer_lockscript)
        .x_unlock_address(unlock_address)
        .redeemer_lockscript(redeemer_lockscript)
        .x_extra(extra)
        .build()
}

fn build_test_context(
    kind: u8,
    xt_burn: u128,
    since: u64,
    output_toCKB_data: Bytes,
    output_capacity: u64,
    output_xt_cell: bool,
) -> (Context, TransactionView) {
    // deploy contract
    let mut context = Context::default();
    let toCKB_typescript_bin: Bytes = Loader::default().load_binary("toCKB-typescript");
    let toCKB_typescript_out_point = context.deploy_cell(toCKB_typescript_bin);
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let sudt_bin = include_bytes!("../../../deps/simple_udt");
    let sudt_out_point = context.deploy_cell(Bytes::from(sudt_bin.as_ref()));
    // prepare scripts
    let toCKB_typescript = context
        .build_script(&toCKB_typescript_out_point, [kind; 1].to_vec().into())
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
        .build_script(&sudt_out_point, sudt_script_args)
        .expect("script");
    let sudt_typescript_dep = CellDep::new_builder().out_point(sudt_out_point).build();

    let lock_address = match kind {
        1 => basic::Bytes::new_builder()
            .set(
                "bc1qzulv8gfw9zd3qtuwmnqafmxnkkuf8cku8mf3ah"
                    .as_bytes()
                    .iter()
                    .map(|c| Byte::new(*c))
                    .collect::<Vec<_>>()
                    .into(),
            )
            .build(),
        2 => toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
            .set([Byte::new(1u8); 20].to_vec())
            .build(),
        _ => toCKB_typescript::utils::types::generated::basic::Bytes::new_builder().build(),
    };
    let input_toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(3u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .x_lock_address(lock_address)
        .signer_lockscript(Script::new_builder().build())
        .x_extra(build_extra(kind))
        .build();

    let input_ckb_cell_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity((11000 * CKB_UNITS).pack())
            .lock(always_success_lockscript.clone())
            .type_(Some(toCKB_typescript.clone()).pack())
            .build(),
        input_toCKB_data.as_bytes(),
    );
    let input_ckb_cell = CellInput::new_builder()
        .previous_output(input_ckb_cell_out_point)
        .since(since.pack())
        .build();

    let input_xt_amount = xt_burn;

    let input_xt_cell_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity((210 * CKB_UNITS).pack())
            .lock(always_success_lockscript.clone())
            .type_(Some(sudt_typescript.clone()).pack())
            .build(),
        input_xt_amount.to_le_bytes().to_vec().into(),
    );
    let input_xt_cell = CellInput::new_builder()
        .previous_output(input_xt_cell_out_point)
        .build();
    let inputs = vec![input_ckb_cell, input_xt_cell];

    let output_ckb_cell = CellOutput::new_builder()
        .capacity((output_capacity * CKB_UNITS).pack())
        .type_(Some(toCKB_typescript.clone()).pack())
        .lock(always_success_lockscript.clone())
        .build();
    let mut outputs = vec![output_ckb_cell];
    let mut outputs_data = vec![output_toCKB_data];

    if output_xt_cell {
        let output_xt_cell = CellOutput::new_builder()
            .capacity((100 * CKB_UNITS).pack())
            .type_(Some(sudt_typescript.clone()).pack())
            .lock(always_success_lockscript)
            .build();
        outputs.push(output_xt_cell);
        outputs_data.push(1u128.to_le_bytes().to_vec().into())
    }
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

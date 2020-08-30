use super::{Error, Script, ToCKBCellData};
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

const ETH_COLLATERAL: u64 = 15 * 250_000_000_000_000_0 + 11000;
const BTC_COLLATERAL: u64 = 15 * 25_000_0 + 11000;

#[test]
fn test_correct_tx_eth() {
    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(2u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .x_lock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .build();
    let (context, tx) = build_test_context(2, ETH_COLLATERAL, toCKB_data.as_bytes());
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_correct_tx_btc() {
    let mole_address =
        molecule::bytes::Bytes::from("bc1qq2pw0kr5yhz3xcs978desw5anfmtwynutwq8quz0t");
    let mole_iter = mole_address.into_iter();
    let mut v = Vec::new();
    for mole in mole_iter {
        v.push(Byte::new(mole));
    }

    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(2u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .x_lock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set(v)
                .build(),
        )
        .build();
    let (context, tx) = build_test_context(1, BTC_COLLATERAL, toCKB_data.as_bytes());
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_wrong_tx_btc_address_invalid() {
    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(2u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .x_lock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .build();

    let (context, tx) = build_test_context(1, BTC_COLLATERAL, toCKB_data.as_bytes());

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::XChainAddressInvalid as i8)
    );
}

#[test]
fn test_wrong_tx_eth_address_invalid() {
    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(2u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .x_lock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set([Byte::new(1u8); 21].to_vec())
                .build(),
        )
        .build();

    let (context, tx) = build_test_context(2, ETH_COLLATERAL, toCKB_data.as_bytes());

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::XChainAddressInvalid as i8)
    );
}

#[test]
fn test_wrong_tx_status_mismatch() {
    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(1u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .x_lock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .build();

    let (context, tx) = build_test_context(2, ETH_COLLATERAL, toCKB_data.as_bytes());

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(err, ScriptError::ValidationFailure(Error::TxInvalid as i8));
}

#[test]
fn test_wrong_tx_kind_mismatch() {
    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(2u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .x_lock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .build();

    let (context, tx) = build_test_context(3, ETH_COLLATERAL, toCKB_data.as_bytes());

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(err, ScriptError::ValidationFailure(Error::Encoding as i8));
}

#[test]
fn test_wrong_tx_lot_size_mismatch() {
    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(2u8))
        .lot_size(Byte::new(2u8))
        .user_lockscript(Script::new_builder().build())
        .x_lock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .build();

    let (context, tx) = build_test_context(2, ETH_COLLATERAL, toCKB_data.as_bytes());

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::InvariantDataMutated as i8)
    );
}

fn test_wrong_tx_collateral_wrong() {
    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(2u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .x_lock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .build();
    let (context, tx) = build_test_context(2, ETH_COLLATERAL + 1, toCKB_data.as_bytes());
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::InvariantDataMutated as i8)
    );
}

fn build_test_context(
    kind: u8,
    value: u64,
    output_toCKB_data: Bytes,
) -> (Context, TransactionView) {
    // deploy contract
    let mut context = Context::default();
    let toCKB_typescript_bin: Bytes = Loader::default().load_binary("toCKB-typescript");
    let toCKB_typescript_out_point = context.deploy_cell(toCKB_typescript_bin);
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

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

    // prepare cells
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(11000u64.pack())
            .lock(always_success_lockscript.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    let input_toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(1u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .build();

    let input_ckb_cell_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(11000u64.pack())
            .lock(always_success_lockscript.clone())
            .type_(Some(toCKB_typescript.clone()).pack())
            .build(),
        input_toCKB_data.as_bytes(),
    );
    let input_ckb_cell = CellInput::new_builder()
        .previous_output(input_ckb_cell_out_point)
        .build();

    let inputs = vec![input_ckb_cell, input];

    let outputs = vec![CellOutput::new_builder()
        .capacity(value.pack())
        .type_(Some(toCKB_typescript.clone()).pack())
        .lock(always_success_lockscript)
        .build()];
    let outputs_data = vec![output_toCKB_data; 1];

    let price: u128 = 10;
    let price_data: [u8; 16] = price.to_le_bytes();

    let witness = WitnessArgs::new_builder()
        .input_type(Some(Bytes::copy_from_slice(&price_data)).pack())
        .build(); // build transaction

    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .witness(witness.as_bytes().pack())
        .outputs_data(outputs_data.pack())
        .cell_dep(toCKB_typescript_dep)
        .cell_dep(always_success_lockscript_dep)
        .build();
    let tx = context.complete_tx(tx);

    (context, tx)
}

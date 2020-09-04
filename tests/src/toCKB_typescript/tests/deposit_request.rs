use super::{Error, Script, ToCKBCellData, PLEDGE};
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
fn test_correct_tx() {
    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(1u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .build();
    let (context, tx) = build_test_context(1, PLEDGE, toCKB_data.as_bytes());

    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_wrong_pledge() {
    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(1u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .build();
    let (context, tx) = build_test_context(1, 9999, toCKB_data.as_bytes());

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::PledgeInvalid as i8)
    );
}

#[test]
fn test_wrong_status() {
    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(2u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .build();
    let (context, tx) = build_test_context(1, PLEDGE, toCKB_data.as_bytes());

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(err, ScriptError::ValidationFailure(Error::TxInvalid as i8));
}

#[test]
fn test_wrong_xchain() {
    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(1u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .build();
    let (context, tx) = build_test_context(3, PLEDGE, toCKB_data.as_bytes());

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(err, ScriptError::ValidationFailure(Error::Encoding as i8));
}

#[test]
fn test_wrong_lot_size() {
    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(1u8))
        .lot_size(Byte::new(9u8))
        .user_lockscript(Script::new_builder().build())
        .build();
    let (context, tx) = build_test_context(1, PLEDGE, toCKB_data.as_bytes());

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::LotSizeInvalid as i8)
    );
}

fn build_test_context(kind: u8, pledge: u64, toCKB_data: Bytes) -> (Context, TransactionView) {
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
    let outputs = vec![CellOutput::new_builder()
        .capacity(pledge.pack())
        .type_(Some(toCKB_typescript.clone()).pack())
        .lock(always_success_lockscript)
        .build()];
    let outputs_data = vec![toCKB_data; 1];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(toCKB_typescript_dep)
        .cell_dep(always_success_lockscript_dep)
        .build();
    let tx = context.complete_tx(tx);

    (context, tx)
}

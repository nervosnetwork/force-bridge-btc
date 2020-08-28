use super::ToCKBCellData;
use crate::toCKB_typescript::utils::types::{Error, ToCKBStatus};
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
    let input_toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(ToCKBStatus::Warranty as u8))
        .lot_size(Byte::new(1u8))
        .build();

    let output_toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(ToCKBStatus::Undercollateral as u8))
        .lot_size(Byte::new(1u8))
        .build();

    let (context, tx) = build_test_context(
        1,
        0,
        1,
        input_toCKB_data.as_bytes(),
        output_toCKB_data.as_bytes(),
    );

    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_wrong_price() {
    let input_toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(ToCKBStatus::Warranty as u8))
        .lot_size(Byte::new(1u8))
        .build();

    let output_toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(ToCKBStatus::Undercollateral as u8))
        .lot_size(Byte::new(1u8))
        .build();

    let (context, tx) = build_test_context(
        1,
        0,
        10,
        input_toCKB_data.as_bytes(),
        output_toCKB_data.as_bytes(),
    );

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::UndercollateralInvalid as i8)
    );
}

#[test]
fn test_wrong_lot_size() {
    let input_toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(ToCKBStatus::Warranty as u8))
        .lot_size(Byte::new(1u8))
        .build();

    let output_toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(ToCKBStatus::Undercollateral as u8))
        .lot_size(Byte::new(2u8))
        .build();

    let (context, tx) = build_test_context(
        1,
        0,
        1,
        input_toCKB_data.as_bytes(),
        output_toCKB_data.as_bytes(),
    );

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::InvariantDataMutated as i8)
    );
}

#[test]
fn test_wrong_status() {
    let input_toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(ToCKBStatus::Undercollateral as u8))
        .lot_size(Byte::new(1u8))
        .build();

    let output_toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(ToCKBStatus::Undercollateral as u8))
        .lot_size(Byte::new(1u8))
        .build();

    let (context, tx) = build_test_context(
        1,
        0,
        1,
        input_toCKB_data.as_bytes(),
        output_toCKB_data.as_bytes(),
    );

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(err, ScriptError::ValidationFailure(Error::TxInvalid as i8));
}

fn build_test_context(
    kind: u8,
    since: u64,
    price: u8,
    input_toCKB_data: Bytes,
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

    let capacity = 3_7500_0000u64;

    // prepare inputs
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(capacity.pack())
            .lock(always_success_lockscript.clone())
            .type_(Some(toCKB_typescript.clone()).pack())
            .build(),
        input_toCKB_data,
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .since(since.pack())
        .build();

    // prepare outputs
    let outputs = vec![CellOutput::new_builder()
        .capacity(capacity.pack())
        .type_(Some(toCKB_typescript.clone()).pack())
        .lock(always_success_lockscript)
        .build()];
    let outputs_data = vec![output_toCKB_data; 1];

    // witness
    let witness = WitnessArgs::new_builder()
        .input_type(Some(Bytes::from(vec![price])).pack())
        .build(); // build transaction

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .witness(witness.as_bytes().pack())
        .outputs_data(outputs_data.pack())
        .cell_dep(toCKB_typescript_dep)
        .cell_dep(always_success_lockscript_dep)
        .build();
    let tx = context.complete_tx(tx);

    (context, tx)
}

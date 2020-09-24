use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_tool::ckb_types::{
    bytes::Bytes,
    core::{TransactionBuilder, TransactionView},
    packed::*,
    prelude::*,
};
use ckb_tool::{ckb_error::assert_error_eq, ckb_script::ScriptError};

use crate::Loader;
use tockb_types::basic;
use tockb_types::generated::tockb_cell_data::ToCKBTypeArgs;

const MAX_CYCLES: u64 = 10_000_000;

#[derive(Clone, Copy)]
struct IsValidToCKBCell {
    is_toCKB_cell_type: bool,
    is_toCKB_cell_valid: bool,
}

#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    // Add customized errors here...
    InvalidToCKBCell,
}

fn load_context_and_out_points() -> (Context, OutPoint, OutPoint) {
    // deploy contract
    let mut context = Context::default();
    let toCKB_lockscript_bin: Bytes = Loader::default().load_binary("toCKB-lockscript");
    let toCKB_lockscript_out_point = context.deploy_cell(toCKB_lockscript_bin);
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

    return (
        context,
        toCKB_lockscript_out_point,
        always_success_out_point,
    );
}

fn build_cell(is_valid_cell: IsValidToCKBCell) -> CellOutput {
    let (mut context, toCKB_lockscript_out_point, always_success_out_point) =
        load_context_and_out_points();

    let args = ToCKBTypeArgs::new_builder()
        .cell_id(basic::OutPoint::from(always_success_out_point.clone()))
        .xchain_kind(Byte::new(1))
        .build();

    // prepare scripts
    let mock_toCKB_typescript = context
        .build_script(&always_success_out_point, args.as_bytes())
        .expect("script");

    let lock_script = if is_valid_cell.is_toCKB_cell_valid {
        let lockscript_args = if is_valid_cell.is_toCKB_cell_type {
            mock_toCKB_typescript.as_bytes()[0..54].to_vec().into()
        } else {
            mock_toCKB_typescript.as_bytes().to_vec().into()
        };

        context
            .build_script(&toCKB_lockscript_out_point, lockscript_args)
            .expect("script")
    } else {
        context
            .build_script(&always_success_out_point, Bytes::new())
            .expect("script")
    };
    // build cell output
    CellOutput::new_builder()
        .capacity(11000u64.pack())
        .lock(lock_script)
        .type_(Some(mock_toCKB_typescript).pack())
        .build()
}

fn build_test_context(
    is_valid_inputs: Vec<IsValidToCKBCell>,
    is_valid_outputs: Vec<IsValidToCKBCell>,
) -> (Context, TransactionView) {
    let (mut context, toCKB_lockscript_out_point, always_success_out_point) =
        load_context_and_out_points();

    let fake_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point.clone())
        .build();
    let toCKB_lockscript_dep = CellDep::new_builder()
        .out_point(toCKB_lockscript_out_point.clone())
        .build();

    let mut inputs = vec![];
    let mut outputs = vec![];
    let mut outputs_data = vec![];

    //prepare input cell
    for &is_valid_cell in is_valid_inputs.iter() {
        let cell = build_cell(is_valid_cell);

        let input_out_point = context.create_cell(cell, Bytes::new());

        let input = CellInput::new_builder()
            .previous_output(input_out_point)
            .build();

        inputs.push(input);
    }

    //prepare output cells
    for &is_valid_cell in is_valid_outputs.iter() {
        let cell = build_cell(is_valid_cell);

        outputs.push(cell);
        outputs_data.push(Bytes::new());
    }

    // build transaction
    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(toCKB_lockscript_dep)
        .cell_dep(fake_script_dep)
        .build();
    let tx = context.complete_tx(tx);

    (context, tx)
}

#[test]
fn test_valid_toCKB_cell() {
    // two input_cell and two output_cell are all valid toCKBCell
    let valid_cell = IsValidToCKBCell {
        is_toCKB_cell_type: true,
        is_toCKB_cell_valid: true,
    };

    let (mut context, tx) =
        build_test_context(vec![valid_cell, valid_cell], vec![valid_cell, valid_cell]);

    let tx = context.complete_tx(tx);

    context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
}

#[test]
fn test_none_toCKB_cell() {
    let not_toCKB_cell_type1 = IsValidToCKBCell {
        is_toCKB_cell_type: false,
        is_toCKB_cell_valid: false,
    };
    let not_toCKB_cell_type2 = IsValidToCKBCell {
        is_toCKB_cell_type: false,
        is_toCKB_cell_valid: true,
    };
    // all the input_cell and output_cell are not toCKBCell
    let (mut context, tx) = build_test_context(
        vec![not_toCKB_cell_type1, not_toCKB_cell_type2],
        vec![not_toCKB_cell_type2, not_toCKB_cell_type1],
    );
    let tx = context.complete_tx(tx);

    context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
}

#[test]
fn test_invalid_toCKB_cell() {
    let invalid_cell = IsValidToCKBCell {
        is_toCKB_cell_type: true,
        is_toCKB_cell_valid: false,
    };
    let valid_cell = IsValidToCKBCell {
        is_toCKB_cell_type: true,
        is_toCKB_cell_valid: true,
    };
    // one input_cell and one output_cell is valid toCKBCell, the others are not invalid
    let (mut context, tx) = build_test_context(
        vec![valid_cell, invalid_cell],
        vec![valid_cell, invalid_cell],
    );
    let tx = context.complete_tx(tx);
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::InvalidToCKBCell as i8)
    );
}

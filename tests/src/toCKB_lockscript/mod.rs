use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_tool::ckb_types::{
    bytes::Bytes,
    core::{TransactionBuilder, TransactionView},
    packed::*,
    prelude::*,
};
use ckb_tool::{ckb_error::assert_error_eq, ckb_script::ScriptError};

use crate::Loader;

const MAX_CYCLES: u64 = 10_000_000;

#[derive(Clone, Copy)]
struct CellCase {
    is_toCKB_typescript_hash: bool,
    is_toCKB_lockscript: bool,
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

fn get_lock_script_args(type_script: Script, is_ToCKBCell: bool) -> Bytes {
    if is_ToCKBCell {
        let type_hash: [u8; 32] = type_script.calc_script_hash().unpack();
        type_hash.to_vec().into()
    } else {
        [0u8; 32].to_vec().into()
    }
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

fn build_cell(cell_case: CellCase) -> CellOutput {
    let (mut context, toCKB_lockscript_out_point, always_success_out_point) =
        load_context_and_out_points();

    // prepare scripts
    let mock_toCKB_typescript = context
        .build_script(&always_success_out_point, [3; 1].to_vec().into())
        .expect("script");

    let lock_script = if cell_case.is_toCKB_lockscript {
        let lockscript_args: Bytes = get_lock_script_args(
            mock_toCKB_typescript.clone(),
            cell_case.is_toCKB_typescript_hash,
        );

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
    input_valid: Vec<CellCase>,
    output_valid: Vec<CellCase>,
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
    for &cell_case in input_valid.iter() {
        let cell = build_cell(cell_case);

        let input_out_point = context.create_cell(cell, Bytes::new());

        let input = CellInput::new_builder()
            .previous_output(input_out_point)
            .build();

        inputs.push(input);
    }

    //prepare output cells
    for &cell_case in output_valid.iter() {
        let cell = build_cell(cell_case);

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
fn test_lock_basic_all_valid() {
    // two input_cell and two output_cell are all valid toCKBCell
    let valid_cell = CellCase {
        is_toCKB_typescript_hash: true,
        is_toCKB_lockscript: true,
    };

    let (mut context, tx) =
        build_test_context(vec![valid_cell, valid_cell], vec![valid_cell, valid_cell]);

    let tx = context.complete_tx(tx);

    context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
}

#[test]
fn test_lock_no_toCKB_cell() {
    let not_ckb_cell_type = CellCase {
        is_toCKB_typescript_hash: false,
        is_toCKB_lockscript: false,
    };
    let valid_cell = CellCase {
        is_toCKB_typescript_hash: false,
        is_toCKB_lockscript: true,
    };
    // all the input_cell and output_cell are not toCKBCell
    let (mut context, tx) = build_test_context(
        vec![not_ckb_cell_type, valid_cell],
        vec![not_ckb_cell_type, not_ckb_cell_type],
    );
    let tx = context.complete_tx(tx);

    context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
}

#[test]
fn test_lock_simple_invalid() {
    let invalid_cell = CellCase {
        is_toCKB_typescript_hash: true,
        is_toCKB_lockscript: false,
    };
    let valid_cell = CellCase {
        is_toCKB_typescript_hash: true,
        is_toCKB_lockscript: true,
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

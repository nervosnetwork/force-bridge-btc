use crate::Loader;
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_tool::ckb_types::{
    bytes::Bytes,
    core::{TransactionBuilder, TransactionView},
    h256,
    packed::*,
    prelude::*,
    H256,
};
use ckb_tool::{ckb_error::assert_error_eq, ckb_script::ScriptError};
use tockb_types::basic;
use tockb_types::generated::tockb_cell_data::ToCKBTypeArgs;

const MAX_CYCLES: u64 = 10_000_000;
// total_size(4 byte) + offset(4 byte) * 3 + code_hash(32 byte) + hash_type(1 byte) + args_size(4 byte) + xchain_kind(1 byte) = 54 byte
const MOLECULE_TYPESCRIPT_SIZE: usize = 54;

#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    // Add customized errors here...
    InvalidToCKBCell,
}

#[test]
fn test_wrong_xchain_kind() {
    let valid_cell = build_cell(
        Byte::new(1),
        Default::default(),
        Byte::new(1),
        Default::default(),
    );
    let invalid_cell = build_cell(
        Byte::new(1),
        Default::default(),
        Byte::new(2),
        Default::default(),
    );

    let (mut context, tx) = build_test_context(vec![&valid_cell, &invalid_cell], vec![&valid_cell]);
    let tx = context.complete_tx(tx);

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::InvalidToCKBCell as i8)
    );
}

#[test]
fn test_different_cell_id() {
    let valid_cell_1 = build_cell(
        Byte::new(1),
        basic::OutPoint::from(OutPoint::new(h256!("0x12345").pack(), 0)),
        Byte::new(1),
        Default::default(),
    );
    let valid_cell_2 = build_cell(
        Byte::new(1),
        basic::OutPoint::from(OutPoint::new(h256!("0x67890").pack(), 0)),
        Byte::new(1),
        Default::default(),
    );
    let (mut context, tx) =
        build_test_context(vec![&valid_cell_1, &valid_cell_2], vec![&valid_cell_2]);
    let tx = context.complete_tx(tx);

    context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
}

#[test]
fn test_same_args() {
    let valid_cell = build_cell(
        Byte::new(1),
        Default::default(),
        Byte::new(1),
        Default::default(),
    );
    let (mut context, tx) = build_test_context(vec![&valid_cell, &valid_cell], vec![&valid_cell]);
    let tx = context.complete_tx(tx);

    context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
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

fn build_cell(
    expect_xchain_kind: Byte,
    expect_cell_id: basic::OutPoint,
    actual_xchain_kind: Byte,
    actual_cell_id: basic::OutPoint,
) -> CellOutput {
    let (mut context, toCKB_lockscript_out_point, always_success_out_point) =
        load_context_and_out_points();

    // prepare lock_script args
    let args = ToCKBTypeArgs::new_builder()
        .cell_id(expect_cell_id)
        .xchain_kind(expect_xchain_kind)
        .build();
    let mock_toCKB_typescript = context
        .build_script(&always_success_out_point, args.as_bytes())
        .expect("script");

    let lock_script_args = mock_toCKB_typescript.as_bytes()[0..MOLECULE_TYPESCRIPT_SIZE]
        .to_vec()
        .into();

    // prepare lock_script
    let lock_script = context
        .build_script(&toCKB_lockscript_out_point, lock_script_args)
        .expect("script");

    // prepare type_script
    let type_script_args = ToCKBTypeArgs::new_builder()
        .cell_id(actual_cell_id)
        .xchain_kind(actual_xchain_kind)
        .build();
    let type_script = context
        .build_script(&always_success_out_point, type_script_args.as_bytes())
        .expect("script");

    // build cell output
    CellOutput::new_builder()
        .capacity(11000u64.pack())
        .lock(lock_script)
        .type_(Some(type_script).pack())
        .build()
}

fn build_test_context(
    input_cells: Vec<&CellOutput>,
    output_cells: Vec<&CellOutput>,
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
    for &cell in input_cells.iter() {
        let input_out_point = context.create_cell(cell.clone(), Bytes::new());
        let input = CellInput::new_builder()
            .previous_output(input_out_point)
            .build();

        inputs.push(input);
    }

    //prepare output cells
    for &cell in output_cells.iter() {
        outputs.push(cell.clone());
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

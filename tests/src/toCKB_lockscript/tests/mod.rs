use crate::toCKB_lockscript::utils::Error;
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

fn build_test_context(
    input_valid: Vec<(bool, bool)>,
    output_valid: Vec<(bool, bool)>,
) -> (Context, TransactionView) {
    // deploy contract
    let mut context = Context::default();
    let toCKB_lockscript_bin: Bytes = Loader::default().load_binary("toCKB-lockscript");
    let toCKB_lockscript_out_point = context.deploy_cell(toCKB_lockscript_bin);
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

    // prepare scripts
    let fake_script = context
        .build_script(&always_success_out_point, [3; 1].to_vec().into())
        .expect("script");

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
    for &is_valid in input_valid.iter() {
        let lockscript_args: Bytes = get_lock_script_args(fake_script.clone(), is_valid.0);
        let lock_script = if is_valid.1 {
            context
                .build_script(&toCKB_lockscript_out_point, lockscript_args)
                .expect("script")
        } else {
            context
                .build_script(&always_success_out_point, lockscript_args)
                .expect("script")
        };

        let input_out_point = context.create_cell(
            CellOutput::new_builder()
                .capacity(11000u64.pack())
                .lock(lock_script)
                .type_(Some(fake_script.clone()).pack())
                .build(),
            Bytes::new(),
        );

        let input = CellInput::new_builder()
            .previous_output(input_out_point)
            .build();

        inputs.push(input);
    }

    //prepare output cell
    for &is_valid in output_valid.iter() {
        let lockscript_args: Bytes = get_lock_script_args(fake_script.clone(), is_valid.0);
        let lock_script = if is_valid.1 {
            context
                .build_script(&toCKB_lockscript_out_point, lockscript_args)
                .expect("script")
        } else {
            context
                .build_script(&always_success_out_point, lockscript_args)
                .expect("script")
        };

        let output = CellOutput::new_builder()
            .capacity(1u64.pack())
            .lock(lock_script)
            .type_(Some(fake_script.clone()).pack())
            .build();

        outputs.push(output);
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

fn get_lock_script_args(type_script: Script, is_ToCKBCell: bool) -> Bytes {
    if is_ToCKBCell {
        let type_hash: [u8; 32] = type_script.calc_script_hash().unpack();
        type_hash.to_vec().into()
    } else {
        [0u8; 32].to_vec().into()
    }
}

#[test]
fn test_lock_basic_all_valid() {
    // two input_cell and two output_cell are all valid toCKBCell
    let (mut context, tx) = build_test_context(
        vec![(true, true), (true, true)],
        vec![(true, true), (true, true)],
    );

    let tx = context.complete_tx(tx);

    context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
}

#[test]
fn test_lock_no_toCKB_cell() {
    // all the input_cell and output_cell are not toCKBCell
    let (mut context, tx) = build_test_context(
        vec![(false, true), (false, false)],
        vec![(false, true), (false, false)],
    );
    let tx = context.complete_tx(tx);

    context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
}

#[test]
fn test_lock_simple_invalid() {
    // one input_cell and one output_cell is valid toCKBCell, the others are not invalid
    let (mut context, tx) = build_test_context(
        vec![(true, true), (true, false)],
        vec![(true, true), (true, false)],
    );
    let tx = context.complete_tx(tx);
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::InvalidToCKBCell as i8)
    );
}

#[test]
fn test_lock_complex_invalid() {
    let (mut context, tx) = build_test_context(
        vec![(true, true), (false, true)],
        vec![(true, true), (false, true)],
    );
    let tx = context.complete_tx(tx);
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::InvalidToCKBCell as i8)
    );
}

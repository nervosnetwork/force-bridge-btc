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

const ETH_COLLATERAL: u128 = 250_000_000_000_000_000;
const BTC_COLLATERAL: u128 = 25_000_000;
const SINGER_FEE: (u128, u128) = (2, 1000);

#[test]
fn test_correct_tx_eth() {
    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(4u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .x_lock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .signer_lockscript(Script::new_builder().build())
        .x_unlock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .redeemer_lockscript(Script::new_builder().build())
        .build();
    let (context, tx) = build_test_context(2, ETH_COLLATERAL, toCKB_data.as_bytes(), false);
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
        .status(Byte::new(4u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .x_lock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set(v.clone())
                .build(),
        )
        .x_unlock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set(v)
                .build(),
        )
        .signer_lockscript(Script::new_builder().build())
        .redeemer_lockscript(Script::new_builder().build())
        .build();
    let (context, tx) = build_test_context(1, BTC_COLLATERAL, toCKB_data.as_bytes(), false);
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_wrong_tx_eth_address_invalid() {
    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(4u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .x_lock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .signer_lockscript(Script::new_builder().build())
        .x_unlock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set([Byte::new(1u8); 21].to_vec())
                .build(),
        )
        .redeemer_lockscript(Script::new_builder().build())
        .build();
    let (context, tx) = build_test_context(2, ETH_COLLATERAL, toCKB_data.as_bytes(), false);
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::XChainAddressInvalid as i8)
    );
}

#[test]
fn test_wrong_tx_btc_address_invalid() {
    let mole_address =
        molecule::bytes::Bytes::from("bc1qq2pw0kr5yhz3xcs978desw5anfmtwynutwq8quz0t");
    let mole_iter = mole_address.into_iter();
    let mut v = Vec::new();
    for mole in mole_iter {
        v.push(Byte::new(mole));
    }

    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(4u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .x_lock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set(v.clone())
                .build(),
        )
        .x_unlock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .signer_lockscript(Script::new_builder().build())
        .redeemer_lockscript(Script::new_builder().build())
        .build();
    let (context, tx) = build_test_context(1, BTC_COLLATERAL, toCKB_data.as_bytes(), false);
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::XChainAddressInvalid as i8)
    );
}

#[test]
fn test_wrong_tx_lot_size_mutated() {
    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(4u8))
        .lot_size(Byte::new(2u8))
        .user_lockscript(Script::new_builder().build())
        .x_lock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .signer_lockscript(Script::new_builder().build())
        .x_unlock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .redeemer_lockscript(Script::new_builder().build())
        .build();
    let (context, tx) = build_test_context(2, ETH_COLLATERAL, toCKB_data.as_bytes(), false);
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::InvariantDataMutated as i8)
    );
}

#[test]
fn test_wrong_tx_user_lock_script_mutated() {
    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(4u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().hash_type(Byte::new(2)).build())
        .x_lock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .signer_lockscript(Script::new_builder().build())
        .x_unlock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .redeemer_lockscript(Script::new_builder().build())
        .build();
    let (context, tx) = build_test_context(2, ETH_COLLATERAL, toCKB_data.as_bytes(), false);
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::InvariantDataMutated as i8)
    );
}

#[test]
fn test_wrong_tx_user_lock_address_mutated() {
    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(4u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .x_lock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set([Byte::new(2u8); 20].to_vec())
                .build(),
        )
        .signer_lockscript(Script::new_builder().build())
        .x_unlock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .redeemer_lockscript(Script::new_builder().build())
        .build();
    let (context, tx) = build_test_context(2, ETH_COLLATERAL, toCKB_data.as_bytes(), false);
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::InvariantDataMutated as i8)
    );
}

#[test]
fn test_wrong_tx_user_signer_script_mutated() {
    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(4u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .x_lock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set([Byte::new(2u8); 20].to_vec())
                .build(),
        )
        .signer_lockscript(Script::new_builder().hash_type(Byte::new(2)).build())
        .x_unlock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .redeemer_lockscript(Script::new_builder().build())
        .build();
    let (context, tx) = build_test_context(2, ETH_COLLATERAL, toCKB_data.as_bytes(), false);
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::InvariantDataMutated as i8)
    );
}

#[test]
fn test_wrong_tx_sudt_value_mismatch() {
    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(4u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .x_lock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .signer_lockscript(Script::new_builder().build())
        .x_unlock_address(
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .redeemer_lockscript(Script::new_builder().build())
        .build();
    let (context, tx) = build_test_context(2, ETH_COLLATERAL * 2, toCKB_data.as_bytes(), false);
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::XTBurnInvalid as i8)
    );
}

fn build_test_context(
    kind: u8,
    value: u128,
    output_toCKB_data: Bytes,
    deposit_requestor_flag: bool,
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
        1 => {
            let mole_address =
                molecule::bytes::Bytes::from("bc1qq2pw0kr5yhz3xcs978desw5anfmtwynutwq8quz0t");
            let mole_iter = mole_address.into_iter();
            let mut v = Vec::new();
            for mole in mole_iter {
                v.push(Byte::new(mole));
            }
            toCKB_typescript::utils::types::generated::basic::Bytes::new_builder()
                .set(v)
                .build()
        }
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

    let signer_fee = value * SINGER_FEE.0 / SINGER_FEE.1;

    let input_xt_amount: u128;
    if deposit_requestor_flag {
        input_xt_amount = value;
    } else {
        input_xt_amount = value + signer_fee;
    }

    let input_xt_cell_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(210u64.pack())
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
        .capacity(100.pack())
        .type_(Some(toCKB_typescript.clone()).pack())
        .lock(always_success_lockscript.clone())
        .build();
    let mut outputs = vec![output_ckb_cell];
    let mut outputs_data = vec![output_toCKB_data];

    if !deposit_requestor_flag {
        let output_xt_cell = CellOutput::new_builder()
            .capacity(100.pack())
            .type_(Some(sudt_typescript.clone()).pack())
            .lock(always_success_lockscript)
            .build();
        outputs.push(output_xt_cell);
        outputs_data.push(signer_fee.to_le_bytes().to_vec().into())
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

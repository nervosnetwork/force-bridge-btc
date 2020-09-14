use super::{Error, Script, ToCKBCellData};
use crate::toCKB_typescript::utils::config::CKB_UNITS;
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

const ETH_PRICE: u128 = 100_000_000_000_000;
const BTC_PRICE: u128 = 100_000;

const ETH_COLLATERAL_WEI: u64 = 1_000_000_000_000_000_000 / (4 * 100) * 150;
const BTC_COLLATERAL_SAT: u64 = 100_000_000 / (4 * 100) * 150;

#[test]
fn test_correct_tx_eth() {
    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(2u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .x_lock_address(
            basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .x_extra(build_extra(2))
        .build();
    let (context, tx) = build_test_context(2, ETH_COLLATERAL_WEI, toCKB_data.as_bytes());
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_correct_tx_btc() {
    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(2u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .x_lock_address(
            basic::Bytes::new_builder()
                .set(
                    "bc1qq2pw0kr5yhz3xcs978desw5anfmtwynutwq8quz0t"
                        .as_bytes()
                        .iter()
                        .map(|c| Byte::new(*c))
                        .collect::<Vec<_>>()
                        .into(),
                )
                .build(),
        )
        .x_extra(build_extra(1))
        .build();
    let (context, tx) = build_test_context(1, BTC_COLLATERAL_SAT, toCKB_data.as_bytes());
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
            basic::Bytes::new_builder()
                .set(
                    "bc1qq2pw0kr5yhz3xcs978desw5anfmtwynutwq8quz00"
                        .as_bytes()
                        .iter()
                        .map(|c| Byte::new(*c))
                        .collect::<Vec<_>>()
                        .into(),
                )
                .build(),
        )
        .x_extra(build_extra(1))
        .build();

    let (context, tx) = build_test_context(1, BTC_COLLATERAL_SAT, toCKB_data.as_bytes());

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
            basic::Bytes::new_builder()
                .set([Byte::new(1u8); 21].to_vec())
                .build(),
        )
        .x_extra(build_extra(2))
        .build();

    let (context, tx) = build_test_context(2, ETH_COLLATERAL_WEI, toCKB_data.as_bytes());

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
            basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .x_extra(build_extra(2))
        .build();

    let (context, tx) = build_test_context(2, ETH_COLLATERAL_WEI, toCKB_data.as_bytes());

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(err, ScriptError::ValidationFailure(Error::TxInvalid as i8));
}

#[test]
fn test_wrong_tx_kind_invalid() {
    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(2u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .x_lock_address(
            basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .x_extra(build_extra(3))
        .build();

    let (context, tx) = build_test_context(3, ETH_COLLATERAL_WEI, toCKB_data.as_bytes());

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
            basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .x_extra(build_extra(2))
        .build();

    let (context, tx) = build_test_context(2, ETH_COLLATERAL_WEI, toCKB_data.as_bytes());

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::InvariantDataMutated as i8)
    );
}

#[test]
fn test_wrong_tx_collateral_wrong() {
    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(2u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .x_lock_address(
            basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .x_extra(build_extra(2))
        .build();
    let (context, tx) = build_test_context(2, ETH_COLLATERAL_WEI * 10, toCKB_data.as_bytes());
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::CollateralInvalid as i8)
    );
}

#[test]
fn test_wrong_tx_extra_mismatch() {
    let eth_extra = EthExtra::new_builder()
        .dummy(
            basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .build();
    let x_extra = XExtraUnion::EthExtra(eth_extra);
    let extra = XExtra::new_builder().set(x_extra).build();

    let toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(2u8))
        .lot_size(Byte::new(1u8))
        .user_lockscript(Script::new_builder().build())
        .x_lock_address(
            basic::Bytes::new_builder()
                .set([Byte::new(1u8); 20].to_vec())
                .build(),
        )
        .x_extra(extra)
        .build();
    let (context, tx) = build_test_context(2, ETH_COLLATERAL_WEI * 10, toCKB_data.as_bytes());
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(Error::InvariantDataMutated as i8)
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

fn build_test_context(
    kind: u8,
    collateral: u64,
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

    let price = match kind {
        1 => BTC_PRICE,
        2 => ETH_PRICE,
        _ => ETH_PRICE,
    };

    let value = (11000 + 2 * 200) * CKB_UNITS + (collateral / price as u64) * CKB_UNITS;
    // prepare cells
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(value.pack())
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
        .build();

    let inputs = vec![input_ckb_cell, input];

    let outputs = vec![CellOutput::new_builder()
        .capacity(value.pack())
        .type_(Some(toCKB_typescript.clone()).pack())
        .lock(always_success_lockscript)
        .build()];
    let outputs_data = vec![output_toCKB_data; 1];

    let price_data: [u8; 16] = price.to_le_bytes();
    let dep_data = Bytes::copy_from_slice(&price_data);
    let data_out_point = context.deploy_cell(dep_data);
    let data_dep = CellDep::new_builder().out_point(data_out_point).build();

    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(data_dep)
        .cell_dep(toCKB_typescript_dep)
        .cell_dep(always_success_lockscript_dep)
        .build();
    let tx = context.complete_tx(tx);

    (context, tx)
}

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
use toCKB_typescript::utils::types::generated::*;

const MAX_CYCLES: u64 = 10_000_000;

const ETH_PRICE: u128 = 100_000_000_000_000;
const BTC_PRICE: u128 = 100_000;

const ETH_BURN: u128 = 250_000_000_000_000_000;
const BTC_BURN: u128 = 25_000_000;

#[test]
fn test_correct_tx_eth() {
    let (context, tx) = build_test_context(2, ETH_BURN);
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
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

fn build_test_context(kind: u8, xt_burn: u128) -> (Context, TransactionView) {
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

    let price = match kind {
        1 => BTC_PRICE,
        2 => ETH_PRICE,
        _ => BTC_PRICE,
    };

    let signer_lockscript =
        basic::Script::from_slice(always_success_lockscript.as_slice()).unwrap();
    let input_toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(3u8))
        .lot_size(Byte::new(1u8))
        .signer_lockscript(signer_lockscript)
        .x_extra(build_extra(kind))
        .build();

    let input_ckb_cell_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(((xt_burn / price) as u64 * CKB_UNITS).pack())
            .lock(always_success_lockscript.clone())
            .type_(Some(toCKB_typescript.clone()).pack())
            .build(),
        input_toCKB_data.as_bytes(),
    );
    let input_ckb_cell = CellInput::new_builder()
        .previous_output(input_ckb_cell_out_point)
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

    let output_signer_cell = CellOutput::new_builder()
        .capacity(((xt_burn / price) as u64 * CKB_UNITS).pack())
        .lock(always_success_lockscript.clone())
        .build();
    let outputs = vec![output_signer_cell];
    let outputs_data = vec![Bytes::new(); 1];

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
        .cell_dep(sudt_typescript_dep)
        .build();
    let tx = context.complete_tx(tx);

    (context, tx)
}

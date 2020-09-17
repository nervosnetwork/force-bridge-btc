use super::{Error, PLEDGE};
use crate::toCKB_typescript::utils::{
    case_builder::{
        CapacityCell, CapacityCells, ScriptView, TestCase, ToCKBCell, ToCKBCellDataView,
        ToCKBCells, ToCKBTypeArgsView, ALWAYS_SUCCESS_OUTPOINT_KEY,
    },
    case_runner,
};
use ckb_tool::ckb_types::bytes::Bytes;
use tockb_types::{BtcExtraView, EthExtraView, XExtraView};

#[test]
fn test_correct_tx() {
    let case = get_correct_case();
    case_runner::run_test(case)
}

#[test]
fn test_wrong_pledge() {
    let mut case = get_correct_case();
    case.toCKB_cells.outputs[0].capacity = 1234;
    case.expect_return_code = Error::PledgeInvalid as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_cell_id() {
    let mut case = get_correct_case();
    case.toCKB_cells.outputs[0].type_args.cell_id = Some(Bytes::from("wrong_cell_id"));
    case.expect_return_code = Error::Encoding as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_status() {
    let mut case = get_correct_case();
    case.toCKB_cells.outputs[0].data.status = 2;
    case.expect_return_code = Error::TxInvalid as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_xchain() {
    let mut case = get_correct_case();
    case.toCKB_cells.outputs[0].type_args.xchain_kind = 3;
    case.expect_return_code = Error::Encoding as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_lot_size() {
    let mut case = get_correct_case();
    case.toCKB_cells.outputs[0].data.lot_size = 9;
    case.expect_return_code = Error::LotSizeInvalid as i8;
    case_runner::run_test(case)
}

fn get_correct_case() -> TestCase {
    TestCase {
        cell_deps: vec![],
        toCKB_cells: ToCKBCells {
            inputs: vec![],
            outputs: vec![ToCKBCell {
                capacity: PLEDGE,
                data: ToCKBCellDataView {
                    status: 1,
                    lot_size: 1,
                    user_lockscript: ScriptView {
                        outpoint_key: ALWAYS_SUCCESS_OUTPOINT_KEY,
                        args: Default::default(),
                    },
                    x_lock_address: "".to_string(),
                    signer_lockscript: ScriptView {
                        outpoint_key: ALWAYS_SUCCESS_OUTPOINT_KEY,
                        args: Default::default(),
                    },
                    x_unlock_address: "".to_string(),
                    redeemer_lockscript: ScriptView {
                        outpoint_key: ALWAYS_SUCCESS_OUTPOINT_KEY,
                        args: Default::default(),
                    },
                    liquidation_trigger_lockscript: ScriptView {
                        outpoint_key: ALWAYS_SUCCESS_OUTPOINT_KEY,
                        args: Default::default(),
                    },
                    x_extra: XExtraView::Btc(BtcExtraView {
                        lock_tx_hash: hex::decode(
                            "2b21846ae6f15cc29e41b2846c78d756abfedb0d6fea7222263cac0024713bc3",
                        )
                            .unwrap()
                            .into(),
                        lock_vout_index: 0,
                    }),
                },
                type_args: ToCKBTypeArgsView {
                    xchain_kind: 1,
                    cell_id: None,
                },
                index: 0,
            }],
        },
        sudt_cells: Default::default(),
        capacity_cells: CapacityCells {
            inputs: vec![CapacityCell {
                capacity: 100,
                lockscript: ScriptView {
                    outpoint_key: ALWAYS_SUCCESS_OUTPOINT_KEY,
                    args: Default::default(),
                },
                index: 0,
            }],
            outputs: vec![],
        },
        witnesses: vec![],
        expect_return_code: 0,
    }
}

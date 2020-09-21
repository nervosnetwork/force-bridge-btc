use crate::toCKB_typescript::utils::{case_builder::*, case_runner};
use tockb_types::{
    config::{CKB_UNITS, SINCE_WITHDRAW_PLEDGE_COLLATERAL},
    Error,
};

const COLLATERAL: u64 = 100 * CKB_UNITS;

#[test]
fn test_correct_tx() {
    let case = get_correct_btc_case();
    case_runner::run_test(case)
}

#[test]
fn test_wrong_input_since() {
    let mut case = get_correct_btc_case();
    case.toCKB_cells.inputs[0].since = 1;
    case.expect_return_code = Error::InputSinceInvalid as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_withdraw_capacity() {
    let mut case = get_correct_btc_case();
    case.capacity_cells.outputs[0].capacity = 1;
    case.expect_return_code = Error::CapacityInvalid as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_withdrawer() {
    let mut case = get_correct_btc_case();
    case.capacity_cells.outputs[0].lockscript = ScriptView {
        outpoint_key: ALWAYS_SUCCESS_OUTPOINT_KEY,
        args: Bytes::from("the withdrawer is not signer"),
    };
    case.expect_return_code = Error::CapacityInvalid as i8;
    case_runner::run_test(case)
}

fn get_correct_btc_case() -> TestCase {
    TestCase {
        cell_deps: vec![],
        toCKB_cells: ToCKBCells {
            inputs: vec![ToCKBCell {
                capacity: COLLATERAL,
                data: ToCKBCellDataView {
                    status: 2,
                    lot_size: 1,
                    user_lockscript: Default::default(),
                    x_lock_address: "".to_string(),
                    signer_lockscript: Default::default(),
                    x_unlock_address: "".to_string(),
                    redeemer_lockscript: Default::default(),
                    liquidation_trigger_lockscript: Default::default(),
                    x_extra: Default::default(),
                },
                type_args: ToCKBTypeArgsView {
                    xchain_kind: 1,
                    cell_id: ToCKBTypeArgsView::default_cell_id(),
                },
                since: SINCE_WITHDRAW_PLEDGE_COLLATERAL,
                index: 0,
            }],
            outputs: vec![],
        },
        sudt_cells: SudtCells {
            inputs: vec![],
            outputs: vec![],
        },
        capacity_cells: CapacityCells {
            inputs: vec![],
            outputs: vec![CapacityCell {
                capacity: COLLATERAL,
                lockscript: Default::default(),
                index: 0,
            }],
        },
        witnesses: vec![],
        expect_return_code: 0,
    }
}

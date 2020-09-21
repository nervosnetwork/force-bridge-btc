use crate::toCKB_typescript::utils::{case_builder::*, case_runner};
use tockb_types::{
    config::{CKB_UNITS, PRE_UNDERCOLLATERAL_RATE, XT_CELL_CAPACITY},
    Error,
};

const BTC_PRICE: u128 = 100_000;
const BTC_BURN_AMOUNT: u128 = 25_000_000;
const TOCKB_INPUT_CELL_CAPACITY: u64 =
    (BTC_BURN_AMOUNT / BTC_PRICE * CKB_UNITS as u128 * PRE_UNDERCOLLATERAL_RATE as u128 / 100)
        as u64
        + XT_CELL_CAPACITY
        - 1;

#[test]
fn test_correct_btc_tx() {
    let case = get_correct_btc_case();
    case_runner::run_test(case)
}

#[test]
fn test_wrong_price_condition() {
    let mut case = get_correct_btc_case();
    if let CellDepView::PriceOracle(price) = &mut case.cell_deps[0] {
        *price = 2 * BTC_PRICE;
    }
    case.expect_return_code = Error::UndercollateralInvalid as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_redeemer_is_not_signer() {
    let mut case = get_correct_btc_case();
    case.toCKB_cells.inputs[0].data.signer_lockscript = ScriptView {
        outpoint_key: ALWAYS_SUCCESS_OUTPOINT_KEY,
        args: Bytes::from("signer"),
    };
    case.expect_return_code = Error::InputSignerInvalid as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_xt_burn() {
    let mut case = get_correct_btc_case();
    case.sudt_cells.inputs[0].amount = 1;
    case.expect_return_code = Error::XTBurnInvalid as i8;
    case_runner::run_test(case)
}

fn get_correct_btc_case() -> TestCase {
    TestCase {
        cell_deps: vec![CellDepView::PriceOracle(BTC_PRICE)],
        toCKB_cells: ToCKBCells {
            inputs: vec![ToCKBCell {
                capacity: TOCKB_INPUT_CELL_CAPACITY,
                data: ToCKBCellDataView {
                    status: 3,
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
                since: 0,
                index: 0,
            }],
            outputs: vec![],
        },
        sudt_cells: SudtCells {
            inputs: vec![SudtCell {
                capacity: 210 * CKB_UNITS,
                amount: BTC_BURN_AMOUNT,
                lockscript: Default::default(),
                owner_script: Default::default(),
                index: 1,
            }],
            outputs: vec![],
        },
        capacity_cells: CapacityCells {
            inputs: vec![],
            outputs: vec![CapacityCell {
                capacity: TOCKB_INPUT_CELL_CAPACITY,
                lockscript: Default::default(),
                index: 0,
            }],
        },
        witnesses: vec![],
        expect_return_code: 0,
    }
}

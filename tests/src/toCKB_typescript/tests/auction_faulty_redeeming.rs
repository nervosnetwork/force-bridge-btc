use crate::toCKB_typescript::utils::{case_builder::*, case_runner};
use tockb_types::{
    config::{AUCTION_MAX_TIME, CKB_UNITS, LOCK_TYPE_FLAG, SINCE_TYPE_TIMESTAMP, XT_CELL_CAPACITY},
    Error,
};

const BTC_BURN_AMOUNT: u128 = 25_000_000;

const TOCKB_CELL_CAPACITY: u64 = 3_750_000 * CKB_UNITS;
const SINCE: u64 = LOCK_TYPE_FLAG | SINCE_TYPE_TIMESTAMP | AUCTION_MAX_TIME;

#[test]
fn test_correct_btc_tx() {
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
fn test_wrong_auction_capacity() {
    let mut case = get_correct_btc_case();
    case.toCKB_cells.inputs[0].since = LOCK_TYPE_FLAG | SINCE_TYPE_TIMESTAMP | 100;
    case.expect_return_code = Error::InvalidTriggerOrSignerCell as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_xt_burn() {
    let mut case = get_correct_btc_case();
    case.sudt_cells.inputs[0].amount = 1;
    case.expect_return_code = Error::FundingNotEnough as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_xt_refund_amount() {
    let mut case = get_correct_btc_case();
    case.sudt_cells.outputs[0].amount = 1;
    case.expect_return_code = Error::InvalidAuctionXTCell as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_xt_refund_capacity() {
    let mut case = get_correct_btc_case();
    case.sudt_cells.outputs[0].capacity = 1;
    case.expect_return_code = Error::InvalidAuctionXTCell as i8;
    case_runner::run_test(case)
}

fn get_correct_btc_case() -> TestCase {
    TestCase {
        cell_deps: vec![],
        toCKB_cells: ToCKBCells {
            inputs: vec![ToCKBCell {
                capacity: TOCKB_CELL_CAPACITY + XT_CELL_CAPACITY,
                data: ToCKBCellDataView {
                    status: 8,
                    lot_size: 1,
                    user_lockscript: Default::default(),
                    x_lock_address: Default::default(),
                    signer_lockscript: Default::default(),
                    x_unlock_address: Default::default(),
                    redeemer_lockscript: Default::default(),
                    liquidation_trigger_lockscript: Default::default(),
                    x_extra: Default::default(),
                },
                type_args: ToCKBTypeArgsView {
                    xchain_kind: 1,
                    cell_id: ToCKBTypeArgsView::default_cell_id(),
                },
                since: SINCE,
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
            outputs: vec![SudtCell {
                capacity: XT_CELL_CAPACITY,
                amount: BTC_BURN_AMOUNT,
                lockscript: Default::default(),
                owner_script: Default::default(),
                index: 1,
            }],
        },
        capacity_cells: CapacityCells {
            inputs: vec![],
            outputs: vec![CapacityCell {
                capacity: TOCKB_CELL_CAPACITY + 1,
                lockscript: Default::default(),
                index: 0,
            }],
        },
        witnesses: vec![],
        expect_return_code: 0,
    }
}

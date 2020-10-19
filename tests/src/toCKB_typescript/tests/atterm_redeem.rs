use crate::toCKB_typescript::utils::{case_builder::*, case_runner};
use tockb_types::{
    config::{CKB_UNITS, SINCE_AT_TERM_REDEEM},
    Error,
};

const BTC_BURN_AMOUNT: u128 = 25_000_000;

#[test]
fn test_correct_tx() {
    let case = get_correct_btc_case();
    case_runner::run_test(case)
}

#[test]
fn test_wrong_x_address() {
    let mut case = get_correct_btc_case();
    case.toCKB_cells.outputs[0].data.x_unlock_address =
        "wrongbc18gfw9zd3qtuwmnqafmxnkkuf8cku8mf3ah".to_string();
    case.expect_return_code = Error::XChainAddressInvalid as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_lock_address_modified() {
    let mut case = get_correct_btc_case();
    case.toCKB_cells.outputs[0].data.x_lock_address =
        "modifiedbc18gfw9zd3qtuwmnqafmxnkkuf8cku8mf".to_string();
    case.expect_return_code = Error::InvariantDataMutated as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_xt_burned() {
    let mut case = get_correct_btc_case();
    case.sudt_cells.inputs[0].amount = 1;
    case.expect_return_code = Error::XTBurnInvalid as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_input_since() {
    let mut case = get_correct_btc_case();
    case.toCKB_cells.inputs[0].since = 1;
    case.expect_return_code = Error::InputSinceInvalid as i8;
    case_runner::run_test(case)
}

fn get_correct_btc_case() -> TestCase {
    TestCase {
        cell_deps: vec![],
        toCKB_cells: ToCKBCells {
            inputs: vec![ToCKBCell {
                capacity: 11000 * CKB_UNITS,
                data: ToCKBCellDataView {
                    status: 3,
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
                since: SINCE_AT_TERM_REDEEM,
                index: 0,
            }],
            outputs: vec![ToCKBCell {
                capacity: 11000 * CKB_UNITS,
                data: ToCKBCellDataView {
                    status: 4,
                    lot_size: 1,
                    user_lockscript: Default::default(),
                    x_lock_address: Default::default(),
                    signer_lockscript: Default::default(),
                    x_unlock_address: "bcrt1qzulv8gfw9zd3qtuwmnqafmxnkkuf8cku05t03d".to_string(),
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
        capacity_cells: Default::default(),
        witnesses: vec![],
        expect_return_code: 0,
    }
}

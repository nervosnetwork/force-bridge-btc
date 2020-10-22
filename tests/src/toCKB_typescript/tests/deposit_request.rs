use crate::toCKB_typescript::utils::{case_builder::*, case_runner};
use tockb_types::{config::PLEDGE, Error};

#[test]
fn test_correct_tx() {
    let case = get_correct_btc_case();
    case_runner::run_test(case)
}

#[test]
fn test_wrong_pledge() {
    let mut case = get_correct_btc_case();
    case.toCKB_cells.outputs[0].capacity = 1234;
    case.expect_return_code = Error::PledgeInvalid as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_cell_id() {
    let mut case = get_correct_btc_case();
    case.toCKB_cells.outputs[0].type_args.cell_id = Some(Bytes::from("wrong_cell_id"));
    case.expect_return_code = Error::Encoding as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_status() {
    let mut case = get_correct_btc_case();
    case.toCKB_cells.outputs[0].data.status = 2;
    case.expect_return_code = Error::TxInvalid as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_xchain() {
    let mut case = get_correct_btc_case();
    case.toCKB_cells.outputs[0].type_args.xchain_kind = 3;
    case.expect_return_code = Error::Encoding as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_lot_size() {
    let mut case = get_correct_btc_case();
    case.toCKB_cells.outputs[0].data.lot_size = 9;
    case.expect_return_code = Error::LotSizeInvalid as i8;
    case_runner::run_test(case)
}

fn get_correct_btc_case() -> TestCase {
    TestCase {
        cell_deps: vec![],
        toCKB_cells: ToCKBCells {
            inputs: vec![],
            outputs: vec![ToCKBCell {
                capacity: PLEDGE,
                data: ToCKBCellDataView {
                    status: 1,
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
                    cell_id: None,
                },
                since: 0,
                index: 0,
            }],
        },
        sudt_cells: Default::default(),
        capacity_cells: CapacityCells {
            inputs: vec![CapacityCell {
                capacity: 100,
                lockscript: Default::default(),
                index: 0,
            }],
            outputs: vec![],
        },
        witnesses: vec![],
        expect_return_code: 0,
    }
}

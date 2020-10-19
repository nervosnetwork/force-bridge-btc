use crate::toCKB_typescript::utils::{case_builder::*, case_runner};
use tockb_types::{
    config::{CKB_UNITS, COLLATERAL_PERCENT, XT_CELL_CAPACITY},
    tockb_cell::BTC_UNIT,
    Error, ETH_UNIT,
};

const ETH_PRICE: u128 = 100_000_000_000_000;
const BTC_PRICE: u128 = 100_000;
const BRIDGE_ETH_AMOUNT_IN_WEI: u64 = ETH_UNIT as u64 / (4 * 100) * COLLATERAL_PERCENT as u64;
const BRIDGE_BTC_AMOUNT_IN_SAT: u64 = BTC_UNIT as u64 / (4 * 100) * COLLATERAL_PERCENT as u64;
const INPUT_TOCKB_CELL_CAPACITY: u64 = 11000 * CKB_UNITS;
const OUTPUT_TOCKB_CELL_CAPACITY_IF_BTC: u64 = INPUT_TOCKB_CELL_CAPACITY
    + 2 * XT_CELL_CAPACITY
    + (BRIDGE_BTC_AMOUNT_IN_SAT / BTC_PRICE as u64) * CKB_UNITS;
const OUTPUT_TOCKB_CELL_CAPACITY_IF_ETH: u64 = INPUT_TOCKB_CELL_CAPACITY
    + 2 * XT_CELL_CAPACITY
    + (BRIDGE_ETH_AMOUNT_IN_WEI / ETH_PRICE as u64) * CKB_UNITS;

#[test]
fn test_correct_tx() {
    let btc_case = get_correct_btc_case();
    case_runner::run_test(btc_case);

    let eth_case = get_correct_eth_case();
    case_runner::run_test(eth_case)
}

#[test]
fn test_wrong_address() {
    let mut btc_case = get_correct_btc_case();
    btc_case.toCKB_cells.outputs[0].data.x_lock_address =
        "wrong2pw0kr5yhz3xcs978desw5anfmtwynutwq8quz00".to_owned();
    btc_case.expect_return_code = Error::XChainAddressInvalid as i8;
    case_runner::run_test(btc_case);

    let mut eth_case = get_correct_eth_case();
    eth_case.toCKB_cells.outputs[0].data.x_lock_address =
        "wrong2pw0kr5yhz3xcs978desw5anfmtwynutwq8quz00".to_owned();
    eth_case.expect_return_code = Error::XChainAddressInvalid as i8;
    case_runner::run_test(eth_case)
}

#[test]
fn test_wrong_xchain_mismatch() {
    let mut case = get_correct_btc_case();
    case.toCKB_cells.outputs[0].data.x_extra = XExtraView::Eth(Default::default());
    case.expect_return_code = Error::XChainMismatch as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_modified_lot_size() {
    let mut case = get_correct_btc_case();
    case.toCKB_cells.outputs[0].data.lot_size = 3;
    case.expect_return_code = Error::InvariantDataMutated as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_collateral_bond() {
    let mut case = get_correct_btc_case();
    case.toCKB_cells.outputs[0].capacity = 3;
    case.expect_return_code = Error::CollateralInvalid as i8;
    case_runner::run_test(case)
}

fn get_correct_btc_case() -> TestCase {
    TestCase {
        cell_deps: vec![CellDepView::PriceOracle(BTC_PRICE)],
        toCKB_cells: ToCKBCells {
            inputs: vec![ToCKBCell {
                capacity: INPUT_TOCKB_CELL_CAPACITY,
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
                    cell_id: ToCKBTypeArgsView::default_cell_id(),
                },
                since: 0,
                index: 0,
            }],
            outputs: vec![ToCKBCell {
                capacity: OUTPUT_TOCKB_CELL_CAPACITY_IF_BTC,
                data: ToCKBCellDataView {
                    status: 2,
                    lot_size: 1,
                    user_lockscript: Default::default(),
                    x_lock_address: "bcrt1qzulv8gfw9zd3qtuwmnqafmxnkkuf8cku05t03d".to_string(),
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
                since: 0,
                index: 0,
            }],
        },
        sudt_cells: Default::default(),
        capacity_cells: Default::default(),
        witnesses: vec![],
        expect_return_code: 0,
    }
}

fn get_correct_eth_case() -> TestCase {
    let mut case = get_correct_btc_case();
    if let CellDepView::PriceOracle(price) = &mut case.cell_deps[0] {
        *price = ETH_PRICE;
    }
    case.toCKB_cells.outputs[0].capacity = OUTPUT_TOCKB_CELL_CAPACITY_IF_ETH;
    case.toCKB_cells.inputs[0].type_args.xchain_kind = 2;
    case.toCKB_cells.inputs[0].data.x_extra = XExtraView::Eth(Default::default());
    case.toCKB_cells.outputs[0].type_args.xchain_kind = 2;
    case.toCKB_cells.outputs[0].data.x_extra = XExtraView::Eth(Default::default());
    // TODO fix eth address codec
    case.toCKB_cells.outputs[0].data.x_lock_address = "5eE3b766D487d7d1A2eF".to_owned();
    case
}

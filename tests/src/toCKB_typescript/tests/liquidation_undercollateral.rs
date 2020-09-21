use crate::toCKB_typescript::utils::{case_builder::*, case_runner};
use tockb_types::{
    config::{CKB_UNITS, XT_CELL_CAPACITY},
    Error,
};

const PRICE: u128 = 1;
const TOCKB_CELL_CAPACITY: u64 = 3_750_000u64 * CKB_UNITS + XT_CELL_CAPACITY;

#[test]
fn test_correct_tx() {
    let case = get_correct_btc_case();
    case_runner::run_test(case)
}

#[test]
fn test_wrong_price_condition() {
    let mut case = get_correct_btc_case();
    if let CellDepView::PriceOracle(price) = &mut case.cell_deps[0] {
        *price = 10 * PRICE;
    }
    case.expect_return_code = Error::UndercollateralInvalid as i8;
    case_runner::run_test(case)
}

fn get_correct_btc_case() -> TestCase {
    TestCase {
        cell_deps: vec![CellDepView::PriceOracle(PRICE)],
        toCKB_cells: ToCKBCells {
            inputs: vec![ToCKBCell {
                capacity: TOCKB_CELL_CAPACITY,
                data: ToCKBCellDataView {
                    status: 3,
                    lot_size: 1,
                    user_lockscript: Default::default(),
                    x_lock_address: "bc1qzulv8gfw9zd3qtuwmnqafmxnkkuf8cku8mf3ah".to_string(),
                    signer_lockscript: Default::default(),
                    x_unlock_address: Default::default(),
                    redeemer_lockscript: Default::default(),
                    liquidation_trigger_lockscript: Default::default(),
                    x_extra: XExtraView::Btc(BtcExtraView {
                        lock_tx_hash:
                            "5227c5fbad9d9202ade7f02452cf880dac1ed270255ebfe6716e8b3e8956571d"
                                .to_string(),
                        lock_vout_index: 1,
                    }),
                },
                type_args: ToCKBTypeArgsView {
                    xchain_kind: 1,
                    cell_id: ToCKBTypeArgsView::default_cell_id(),
                },
                since: 0,
                index: 0,
            }],
            outputs: vec![ToCKBCell {
                capacity: TOCKB_CELL_CAPACITY,
                data: ToCKBCellDataView {
                    status: 6,
                    lot_size: 1,
                    user_lockscript: Default::default(),
                    x_lock_address: "bc1qzulv8gfw9zd3qtuwmnqafmxnkkuf8cku8mf3ah".to_string(),
                    signer_lockscript: Default::default(),
                    x_unlock_address: Default::default(),
                    redeemer_lockscript: Default::default(),
                    liquidation_trigger_lockscript: Default::default(),
                    x_extra: XExtraView::Btc(BtcExtraView {
                        lock_tx_hash:
                            "5227c5fbad9d9202ade7f02452cf880dac1ed270255ebfe6716e8b3e8956571d"
                                .to_string(),
                        lock_vout_index: 1,
                    }),
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

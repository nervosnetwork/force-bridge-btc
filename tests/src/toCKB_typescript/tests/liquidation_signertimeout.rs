use crate::toCKB_typescript::utils::{case_builder::*, case_runner};
use tockb_types::{
    config::{CKB_UNITS, SINCE_SIGNER_TIMEOUT},
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
fn test_wrong_mint_xt() {
    let mut case = get_correct_btc_case();
    case.sudt_cells.outputs.push(SudtCell {
        capacity: 200 * CKB_UNITS,
        amount: 100,
        lockscript: Default::default(),
        owner_script: Default::default(),
        index: 1,
    });
    case.expect_return_code = Error::TxInvalid as i8;
    case_runner::run_test(case)
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
fn test_wrong_modified_x_lock_address() {
    let mut case = get_correct_btc_case();
    case.toCKB_cells.outputs[0].data.x_lock_address = "".to_string();
    case.expect_return_code = Error::InvariantDataMutated as i8;
    case_runner::run_test(case)
}

fn get_correct_btc_case() -> TestCase {
    TestCase {
        cell_deps: vec![],
        toCKB_cells: ToCKBCells {
            inputs: vec![ToCKBCell {
                capacity: COLLATERAL,
                data: ToCKBCellDataView {
                    status: 4,
                    lot_size: 1,
                    user_lockscript: Default::default(),
                    x_lock_address: "bc1qzulv8gfw9zd3qtuwmnqafmxnkkuf8cku8mf3ah".to_string(),
                    signer_lockscript: Default::default(),
                    x_unlock_address: "bc1qzulv8gfw9zd3qtuwmnqafmxnkkuf8cku8mf3ah".to_string(),
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
                since: SINCE_SIGNER_TIMEOUT,
                index: 0,
            }],
            outputs: vec![ToCKBCell {
                capacity: COLLATERAL,
                data: ToCKBCellDataView {
                    status: 5,
                    lot_size: 1,
                    user_lockscript: Default::default(),
                    x_lock_address: "bc1qzulv8gfw9zd3qtuwmnqafmxnkkuf8cku8mf3ah".to_string(),
                    signer_lockscript: Default::default(),
                    x_unlock_address: "bc1qzulv8gfw9zd3qtuwmnqafmxnkkuf8cku8mf3ah".to_string(),
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

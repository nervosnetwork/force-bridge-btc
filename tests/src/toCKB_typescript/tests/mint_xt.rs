use crate::toCKB_typescript::utils::{case_builder::*, case_runner};
use tockb_types::{
    config::{CKB_UNITS, PLEDGE, XT_CELL_CAPACITY},
    Error::*,
};

const COLLATERAL: u64 = 100_000 * CKB_UNITS;

#[test]
fn test_correct_case() {
    let case = get_correct_btc_case();
    case_runner::run_test(case)
}

#[test]
fn test_wrong_lot_size() {
    let mut case = get_correct_btc_case();
    case.toCKB_cells.inputs[0].data.lot_size = 100;
    case.expect_return_code = LotSizeInvalid as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_x_lock_address() {
    let mut case = get_correct_btc_case();
    case.toCKB_cells.inputs[0].data.x_lock_address =
        "bc1111111111111111111111111111111111111111111".to_owned();
    case.expect_return_code = WrongFundingAddr as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_mint_xt_amount() {
    let mut case = get_correct_btc_case();
    case.sudt_cells.outputs[0].amount = 1;
    case.expect_return_code = InvalidMintOutput as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_cell_dep_index_list_len() {
    let mut case = get_correct_btc_case();
    if let Witness::Btc(btc_witness) = &mut case.witnesses[0] {
        btc_witness.cell_dep_index_list = vec![1, 2];
        case.expect_return_code = InvalidWitness as i8;
        case_runner::run_test(case)
    }
}

#[test]
fn test_wrong_btc_spv() {
    let mut case = get_correct_btc_case();
    if let Witness::Btc(btc_witness) = &mut case.witnesses[0] {
        btc_witness.spv_proof.index = 2;
        case.expect_return_code = BadMerkleProof as i8;
        case_runner::run_test(case)
    }
}

#[test]
fn test_wrong_btc_difficulty() {
    let mut case = get_correct_btc_case();
    if let CellDepView::DifficultyOracle(difficulty) = &mut case.cell_deps[0] {
        difficulty.previous = 1;
        difficulty.current = 1;
        case.expect_return_code = NotAtCurrentOrPreviousDifficulty as i8;
        case_runner::run_test(case)
    }
}

#[test]
fn test_wrong_toCKB_capacity() {
    let mut case = get_correct_btc_case();
    case.toCKB_cells.outputs[0].capacity = 1;
    case.expect_return_code = CapacityInvalid as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_pledge_refund() {
    let mut case = get_correct_btc_case();
    case.sudt_cells.outputs[0].capacity = 1;
    case.expect_return_code = CapacityInvalid as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_signer_xt_cell_capacity() {
    let mut case = get_correct_btc_case();
    case.sudt_cells.outputs[1].capacity = 1;
    case.expect_return_code = CapacityInvalid as i8;
    case_runner::run_test(case)
}

fn get_correct_btc_case() -> TestCase {
    TestCase {
        cell_deps: vec![CellDepView::DifficultyOracle(DifficultyOracle {
            previous: 17557993035167,
            current: 17557993035167,
        })],
        toCKB_cells: ToCKBCells {
            inputs: vec![ToCKBCell {
                capacity: COLLATERAL,
                data: ToCKBCellDataView {
                    status: 2,
                    lot_size: 1,
                    user_lockscript: Default::default(),
                    x_lock_address: "bc1qzulv8gfw9zd3qtuwmnqafmxnkkuf8cku8mf3ah".to_string(),
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
                capacity: COLLATERAL - PLEDGE - XT_CELL_CAPACITY,
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
                        "2b21846ae6f15cc29e41b2846c78d756abfedb0d6fea7222263cac0024713bc3"
                            .to_owned(),
                        lock_vout_index: 0,
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
        sudt_cells: SudtCells {
            inputs: vec![],
            outputs: vec![
                SudtCell {
                    capacity: PLEDGE,
                    amount: 24950000,
                    lockscript: Default::default(),
                    owner_script: Default::default(),
                    index: 1,
                },
                SudtCell {
                    capacity: XT_CELL_CAPACITY,
                    amount: 50000,
                    lockscript: Default::default(),
                    owner_script: Default::default(),
                    index: 2,
                },
            ],
        },
        capacity_cells: Default::default(),
        witnesses: vec![Witness::Btc(BtcWitness {
            cell_dep_index_list: vec![0],
            spv_proof: BTCSPVProofJson {
                version: 2,
                vin: "0x015227c5fbad9d9202ade7f02452cf880dac1ed270255ebfe6716e8b3e8956571d0100000017160014085fc2ea0c102fc4db8dbbb10dd6f93684c178c9feffffff".to_owned(),
                vout: "0x028c79171300000000160014173ec3a12e289b102f8edcc1d4ecd3b5b893e2dc97b2030000000000160014ef9665bcf82fa83e870a350a6551a09ee819e4a3".to_owned(),
                locktime: 645339,
                tx_id: "0x2b21846ae6f15cc29e41b2846c78d756abfedb0d6fea7222263cac0024713bc3".to_owned(),
                index: 3,
                headers: "0x00000020acf05cadf6d066d01f5aca661690f4e1779a8144b90b070000000000000000006bbb5a7851af48d883e8ac5d6f61c6ad9a4132a9a12531c1b6f085760b3b2e427ba0455fea0710177d792e86".to_owned(),
                intermediate_nodes: "0x8546dfccb488115f9c3210255523c0e186fb9b64d16ac68b3d8903bf037dc3ab26069e90c930cc55105d5f8b4ddd798bc33f057641e748fd2e70de0b8747cae802af46fb1e1fccf354b4b46d87f5a85c564fd5284cbe2a5711c16c446fbb6e9e0b3c7beec06a156a8005883b8cf224f665d361a2269b6b21491c1ccbb8160c311b609b5ca21b0a9f708e6124b36871b71c5536d8d556054be435cf0444da70d0814e678eb0e081805d777f9cf84911f9e04b6a80b6cf60dec31527ec73aaa8ba77ec6bff2e04fbb80c8c81b1cc38b415bc21dd732f51a4a903ee265b0eef2c589f751e66e46bb02aa36ed8418ae93317316b84d12f1b1702dd9641ead0ad7f8777526ad7a4ff599946d219a7a932ec8cd2e42649b3d5fa123d2e4532de6d46bddb27a8c02de8fb8fe2c4d88a14132de8cdd7d471bc6a8c8c217aeec600fd295e8925b663332f45bdb6877dd6e0ecd28bfae530ba3ed8bd3959644a82bc418f9c887746e15ae55d82369c3761187ea449c7f7bdff1acaa0b467e1335b3919089d".to_owned(),
                funding_output_index: 0,
                funding_input_index: 0,
            },
        })],
        expect_return_code: 0,
    }
}

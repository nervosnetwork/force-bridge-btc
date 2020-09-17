use super::{CKB_UNITS, PLEDGE, XT_CELL_CAPACITY};
use crate::toCKB_typescript::tests::ToCKBStatus;
use crate::toCKB_typescript::utils::types::{generated::mint_xt_witness, Error::*};
use crate::toCKB_typescript::utils::{
    helper::{deploy, run_test_case, DeployResult},
    test_case::*,
};
use molecule::prelude::*;
use std::convert::TryInto;

const COLLATERAL: u64 = 100_000 * CKB_UNITS;

fn generate_btc_correct_case() -> TestCase {
    let kind = 1;
    let DeployResult {
        context: _,
        toCKB_typescript: _,
        always_success_lockscript,
        sudt_typescript,
    } = deploy(kind);
    let user_lockscript = always_success_lockscript.clone();
    let signer_lockscript = always_success_lockscript.clone();
    let tockb_data = ToCKBCellDataTest {
        lot_size: 1,
        x_lock_address: "bc1qq2pw0kr5yhz3xcs978desw5anfmtwynutwq8quz0t".to_owned(),
        x_unlock_address: "bc1qq2pw0kr5yhz3xcs978desw5anfmtwynutwq8quz0t".to_owned(),
        user_lockscript: user_lockscript.clone(),
        signer_lockscript: signer_lockscript.clone(),
        x_extra: XExtraView::Btc(BtcExtraView {
            lock_tx_hash: hex::decode(clear_0x(
                "0x2b21846ae6f15cc29e41b2846c78d756abfedb0d6fea7222263cac0024713bc3",
            ))
            .unwrap()
            .into(),
            lock_vout_index: 0,
        }),
    };

    let case = TestCase {
        kind,
        input_status: Some(ToCKBStatus::Bonded as u8),
        output_status: Some(ToCKBStatus::Warranty as u8),
        input_capacity: COLLATERAL,
        output_capacity: COLLATERAL - PLEDGE - XT_CELL_CAPACITY,
        input_tockb_cell_data: tockb_data.clone(),
        output_tockb_cell_data: tockb_data,
        inputs: vec![],
        outputs: vec![
            Output {
                typescript: sudt_typescript.clone(),
                lockscript: always_success_lockscript.clone(),
                amount: 24950000,
                capacity: PLEDGE,
            },
            Output {
                typescript: sudt_typescript.clone(),
                lockscript: always_success_lockscript.clone(),
                amount: 50000,
                capacity: XT_CELL_CAPACITY,
            },
        ],
        witness: Witness {
            cell_dep_index_list: vec![0],
            spv_proof: SpvProof::BTC(BTCSPVProofJson{
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
            }.try_into().unwrap()),
        },
        cell_deps_data: CellDepsData::BTC(
            BtcDifficultyTest {
                previous: 17557993035167u64,
                current: 17557993035167u64,
            }
        ),
        expect_return_code: 0,
    };
    case
}

fn generate_eth_correct_case() -> TestCase {
    let kind = 2;
    let DeployResult {
        context: _,
        toCKB_typescript: _,
        always_success_lockscript,
        sudt_typescript,
    } = deploy(kind);
    let user_lockscript = always_success_lockscript.clone();
    let signer_lockscript = always_success_lockscript.clone();
    let tockb_data = ToCKBCellDataTest {
        lot_size: 1,
        x_lock_address: "c4c7497fbe1a886841a195a5d622cd60053c1376".to_owned(),
        x_unlock_address: "c4c7497fbe1a886841a195a5d622cd60053c1376".to_owned(),
        user_lockscript: user_lockscript.clone(),
        signer_lockscript: signer_lockscript.clone(),
        x_extra: XExtraView::Eth(EthExtraView {
            dummy: Default::default(),
        }),
    };

    let case = TestCase {
        kind,
        input_status: Some(ToCKBStatus::Bonded as u8),
        output_status: Some(ToCKBStatus::Warranty as u8),
        input_capacity: COLLATERAL,
        output_capacity: COLLATERAL - PLEDGE - XT_CELL_CAPACITY,
        input_tockb_cell_data: tockb_data.clone(),
        output_tockb_cell_data: tockb_data,
        inputs: vec![],
        outputs: vec![
            Output {
                typescript: sudt_typescript.clone(),
                lockscript: always_success_lockscript.clone(),
                amount: 249500000000000000,
                capacity: PLEDGE,
            },
            Output {
                typescript: sudt_typescript.clone(),
                lockscript: always_success_lockscript.clone(),
                amount: 500000000000000,
                capacity: XT_CELL_CAPACITY,
            },
        ],
        witness: Witness {
            cell_dep_index_list: vec![0],
            spv_proof: SpvProof::ETH(
                ETHSPVProofJson {
                    log_index: 2,
                    log_entry_data: "f9011c94c4c7497fbe1a886841a195a5d622cd60053c1376f863a0dd85dc56b5b4da387bf69c28ec19b1d66e793e0d51b567882fa31dc50bbd32c5a00000000000000000000000007cc4b1851c35959d34e635a470f6b5c43ba3c9c9a0000000000000000000000000df08f82de32b8d460adbe8d72043e3a7e25a3b39b8a000000000000000000000000000000000000000000000000000000000000003e8000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000217261696e626f775f6272696467655f6574685f6f6e5f6e6561725f70726f76657200000000000000000000000000000000000000000000000000000000000000".to_owned(),
                    receipt_index: 0,
                    receipt_data: "f903630182cf73b9010000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000080000000000000200000000000000200000000000000000000000100008040400000000000000000000000000000000000000000000000000000000000000000000000000400000000000200010000000000100000000000000000000000000000000000080000000000000000000200000020000000000000080000000000000000000000000000010000000004000000000000002000000000000000000000000000004000000080000000004000000000010000000000000000000000000000000000000100000000000000000000000f90259f89b947cc4b1851c35959d34e635a470f6b5c43ba3c9c9f863a0ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3efa0000000000000000000000000df08f82de32b8d460adbe8d72043e3a7e25a3b39a0000000000000000000000000c4c7497fbe1a886841a195a5d622cd60053c1376a000000000000000000000000000000000000000000000000000000000000003e8f89b947cc4b1851c35959d34e635a470f6b5c43ba3c9c9f863a08c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925a0000000000000000000000000df08f82de32b8d460adbe8d72043e3a7e25a3b39a0000000000000000000000000c4c7497fbe1a886841a195a5d622cd60053c1376a00000000000000000000000000000000000000000000000000000000000000000f9011c94c4c7497fbe1a886841a195a5d622cd60053c1376f863a0dd85dc56b5b4da387bf69c28ec19b1d66e793e0d51b567882fa31dc50bbd32c5a00000000000000000000000007cc4b1851c35959d34e635a470f6b5c43ba3c9c9a0000000000000000000000000df08f82de32b8d460adbe8d72043e3a7e25a3b39b8a000000000000000000000000000000000000000000000000000000000000003e8000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000217261696e626f775f6272696467655f6574685f6f6e5f6e6561725f70726f76657200000000000000000000000000000000000000000000000000000000000000".to_owned(),
                    receipts_root:
                    "0x335f738baae5dc1f99d785641ed7f18ab96e3c09d3eac414cef1421425928b87"
                        .to_owned(),
                    header_data: "0x02000000".to_owned(),
                    proof: vec!["f9036c822080b90366f903630182cf73b9010000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000080000000000000200000000000000200000000000000000000000100008040400000000000000000000000000000000000000000000000000000000000000000000000000400000000000200010000000000100000000000000000000000000000000000080000000000000000000200000020000000000000080000000000000000000000000000010000000004000000000000002000000000000000000000000000004000000080000000004000000000010000000000000000000000000000000000000100000000000000000000000f90259f89b947cc4b1851c35959d34e635a470f6b5c43ba3c9c9f863a0ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3efa0000000000000000000000000df08f82de32b8d460adbe8d72043e3a7e25a3b39a0000000000000000000000000c4c7497fbe1a886841a195a5d622cd60053c1376a000000000000000000000000000000000000000000000000000000000000003e8f89b947cc4b1851c35959d34e635a470f6b5c43ba3c9c9f863a08c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925a0000000000000000000000000df08f82de32b8d460adbe8d72043e3a7e25a3b39a0000000000000000000000000c4c7497fbe1a886841a195a5d622cd60053c1376a00000000000000000000000000000000000000000000000000000000000000000f9011c94c4c7497fbe1a886841a195a5d622cd60053c1376f863a0dd85dc56b5b4da387bf69c28ec19b1d66e793e0d51b567882fa31dc50bbd32c5a00000000000000000000000007cc4b1851c35959d34e635a470f6b5c43ba3c9c9a0000000000000000000000000df08f82de32b8d460adbe8d72043e3a7e25a3b39b8a000000000000000000000000000000000000000000000000000000000000003e8000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000217261696e626f775f6272696467655f6574685f6f6e5f6e6561725f70726f76657200000000000000000000000000000000000000000000000000000000000000".to_owned()],
                }
                    .try_into()
                    .unwrap(),
            ),
        },
        cell_deps_data: CellDepsData::ETH(),
        expect_return_code: 0,
    };
    case
}

#[test]
fn test_btc_correct_case() {
    let case = generate_btc_correct_case();
    run_test_case(case);
}

#[test]
fn test_eth_correct_case() {
    let case = generate_eth_correct_case();
    run_test_case(case);
}

#[test]
fn test_wrong_lot_size() {
    let mut case = generate_btc_correct_case();
    case.input_tockb_cell_data.lot_size = 100;
    case.expect_return_code = LotSizeInvalid as i8;
    run_test_case(case);
    let mut case = generate_eth_correct_case();
    case.input_tockb_cell_data.lot_size = 100;
    case.output_tockb_cell_data.lot_size = 100;
    case.expect_return_code = LotSizeInvalid as i8;
    run_test_case(case);
}

#[test]
fn test_wrong_x_lock_address() {
    let mut case = generate_btc_correct_case();
    let wrong_addr = "bc1111111111111111111111111111111111111111111".to_owned();
    case.input_tockb_cell_data.x_lock_address = wrong_addr;
    case.expect_return_code = WrongFundingAddr as i8;
    run_test_case(case);
    let mut case = generate_eth_correct_case();
    let wrong_addr = "bc1111111111111111111111111111111111111111111".to_owned();
    case.input_tockb_cell_data.x_lock_address = wrong_addr.clone();
    case.output_tockb_cell_data.x_lock_address = wrong_addr.clone();
    case.expect_return_code = WrongFundingAddr as i8;
    run_test_case(case);
}

#[test]
fn test_wrong_mint_xt_amount() {
    let mut case = generate_btc_correct_case();
    case.outputs[0].amount = 1;
    case.expect_return_code = InvalidMintOutput as i8;
    run_test_case(case);
    let mut case = generate_eth_correct_case();
    case.outputs[0].amount = 1;
    case.expect_return_code = InvalidMintOutput as i8;
    run_test_case(case);
}

#[test]
fn test_wrong_cell_dep_index_list_len() {
    let mut case = generate_btc_correct_case();
    case.witness.cell_dep_index_list = vec![1, 2];
    case.expect_return_code = InvalidWitness as i8;
    run_test_case(case);
}

#[test]
fn test_wrong_btc_spv() {
    let mut case = generate_btc_correct_case();
    case.witness.spv_proof = SpvProof::BTC(mint_xt_witness::BTCSPVProof::default());
    case.expect_return_code = -1;
    run_test_case(case);
}

#[test]
fn test_wrong_btc_difficulty() {
    let mut case = generate_btc_correct_case();
    case.cell_deps_data = CellDepsData::BTC(BtcDifficultyTest {
        previous: 1,
        current: 1,
    });
    case.expect_return_code = NotAtCurrentOrPreviousDifficulty as i8;
    run_test_case(case);
}

#[test]
fn test_wrong_toCKB_capacity() {
    let mut case = generate_btc_correct_case();
    case.output_capacity = 10000;
    case.expect_return_code = CapacityInvalid as i8;
    run_test_case(case);
}

#[test]
fn test_wrong_pledge_refund() {
    let mut case = generate_btc_correct_case();
    case.outputs[0].capacity = 1;
    case.expect_return_code = CapacityInvalid as i8;
    run_test_case(case);
}

#[test]
fn test_wrong_signer_xt_cell_capacity() {
    let mut case = generate_btc_correct_case();
    case.outputs[1].capacity = 1;
    case.expect_return_code = CapacityInvalid as i8;
    run_test_case(case);
}

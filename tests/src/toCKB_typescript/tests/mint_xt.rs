use crate::toCKB_typescript::utils::{case_builder::*, case_runner};
use tockb_types::{
    config::{CKB_UNITS, PLEDGE, XT_CELL_CAPACITY},
    Error::*,
};
use hex::FromHex;
use rlp;
use rlp::RlpStream;

const COLLATERAL: u64 = 100_000 * CKB_UNITS;

#[test]
fn test_correct_case() {
    let case = get_correct_btc_case();
    case_runner::run_test(case)
}

#[test]
fn test_eth_correct_case() {
    let case = get_correct_eth_case();
    case_runner::run_test(case)
}

#[test]
fn test_wrong_lot_size() {
    let mut case = get_correct_btc_case();
    case.toCKB_cells.inputs[0].data.lot_size = 100;
    case.expect_return_code = LotSizeInvalid as i8;
    case_runner::run_test(case);

    case = get_correct_eth_case();
    case.toCKB_cells.inputs[0].data.lot_size = 100;
    case.toCKB_cells.outputs[0].data.lot_size = 100;
    case.expect_return_code = LotSizeInvalid as i8;
    case_runner::run_test(case);

}

#[test]
fn test_wrong_x_lock_address() {
    let mut case = get_correct_btc_case();
    case.toCKB_cells.inputs[0].data.x_lock_address =
        "bc1111111111111111111111111111111111111111111".to_owned();
    case.expect_return_code = WrongFundingAddr as i8;
    case_runner::run_test(case);

    case = get_correct_eth_case();
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
    case_runner::run_test(case);

    case = get_correct_eth_case();
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

    case = get_correct_eth_case();
    if let Witness::Eth(eth_witness) = &mut case.witnesses[0] {
        eth_witness.cell_dep_index_list = vec![1, 2];
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
fn test_wrong_eth_spv() {
    let mut case = get_correct_eth_case();
    if let Witness::Eth(eth_witness) = &mut case.witnesses[0] {
        eth_witness.spv_proof.receipt_index = 1;
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
fn test_wrong_eth_headers() {
    let mut case = get_correct_eth_case();
    if let CellDepView::HeadersOracle(headers_oracle) = &mut case.cell_deps[0] {
        headers_oracle.headers = vec!["f90210a07643ad4d6a41bf6b016ea6020e5e791cf4863b7d5dd6da04d37b9e5742b0f35ea01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934794ea674fdde714fd979de3edf0f56aa9716b898ec8a0d634a30ae5660a4ce4af7e39698fdb65e720d785dcdb68bab05a8c21414df45ca0593fde2e80b96ec9416ce19576e7ee34945f26885f071e80c5f64985ce68c0ada0fd2abf56a48a3119274b723606114b8274c5fafd51b500161e9d2e9f7548d3f8b9010003a0420a6006200011904080b181736a008055801600863260c1051904000101008476455010490451209106910603304a1000004d22804846928cb0002011aa64110e428201e789ed000c8812804a28120100e4226802444046d0c08440d4141854211c2214840c4448230480001a1016020270480eac23112a8116000814502902c0564782a287b0c1000028640c014c09493b89f2021a0101404e23d001511b445ab00c2334c4148822c13082909122a70041c0004218784246b040810841212910428004108016620041c0565044e0000a2444096410cd7800c208483b00011620400100000108301550000282800c298121b078005c3011000c02b34210870bc1c0a041deaa83a697cd83bebc2083be961e845f6b09568f65746865726d696e652d7573312d32a08e699bb3ed79f57c141ee5462bc3634b22f58773e94ed9c71e4b7836408a8b9a88fcd33b22f4aa5877".to_owned()];
        case.expect_return_code = HeaderIsNotOnMainChain as i8;
        case_runner::run_test(case)
    }
}

#[test]
fn test_wrong_toCKB_capacity() {
    let mut case = get_correct_btc_case();
    case.toCKB_cells.outputs[0].capacity = 1;
    case.expect_return_code = CapacityInvalid as i8;
    case_runner::run_test(case);

    case = get_correct_eth_case();
    case.toCKB_cells.outputs[0].capacity = 1;
    case.expect_return_code = CapacityInvalid as i8;
    case_runner::run_test(case)
}

#[test]
fn test_wrong_pledge_refund() {
    let mut case = get_correct_btc_case();
    case.sudt_cells.outputs[0].capacity = 1;
    case.expect_return_code = CapacityInvalid as i8;
    case_runner::run_test(case);

    case = get_correct_eth_case();
    case.sudt_cells.outputs[0].capacity = 1;
    case.expect_return_code = CapacityInvalid as i8;
    case_runner::run_test(case);
}

#[test]
fn test_wrong_signer_xt_cell_capacity() {
    let mut case = get_correct_btc_case();
    case.sudt_cells.outputs[1].capacity = 1;
    case.expect_return_code = CapacityInvalid as i8;
    case_runner::run_test(case);

    case = get_correct_eth_case();
    case.sudt_cells.outputs[1].capacity = 1;
    case.expect_return_code = CapacityInvalid as i8;
    case_runner::run_test(case);
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

fn get_correct_eth_case() -> TestCase {
    TestCase {
        cell_deps:
        vec![CellDepView::HeadersOracle(HeadersOracle {
            headers: vec!["f9021aa0f779e50b45bc27e4ed236840e5dbcf7afab50beaf553be56bf76da977e10cc73a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d493479452bc44d5378309ee2abf1539bf71de1b7d7be3b5a014c996b6934d7991643669e145b8355c63aa02cbde63d390fcf4e6181d5eea45a079b7e79dc739c31662fe6f25f65bf5a5d14299c7a7aa42c3f75b9fb05474f54ca0e28dc05418692cb7baab7e7f85c1dedb8791c275b797ea3b1ffcaec5ef2aa271b9010000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000010000000000000000000000000000000000000000000000000000000408000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000010000000000000000000000000000000000000000000000000000000400000000000100000000000000000000000000080000000000000000000000000000000000000000000100002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000903234373439353837313930323034343383890fe68395ba8e82d0d9845dd84a079150505945206e616e6f706f6f6c2e6f7267a0a35425f443452cf94ba4b698b00fd7b3ff4fc671dea3d5cc2dcbedbc3766f45e88af7fec6031063a17".to_owned()]
        })],
        toCKB_cells: ToCKBCells {
            inputs: vec![ToCKBCell {
                capacity: COLLATERAL,
                data: ToCKBCellDataView {
                    status: 2,
                    lot_size: 1,
                    user_lockscript: Default::default(),
                    x_lock_address: "dac17f958d2ee523a2206206994597c13d831ec7".to_string(),
                    signer_lockscript: Default::default(),
                    x_unlock_address: Default::default(),
                    redeemer_lockscript: Default::default(),
                    liquidation_trigger_lockscript: Default::default(),
                    x_extra: XExtraView::Eth(Default::default()),
                },
                type_args: ToCKBTypeArgsView {
                    xchain_kind: 2,
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
                    x_lock_address: "dac17f958d2ee523a2206206994597c13d831ec7".to_string(),
                    signer_lockscript: Default::default(),
                    x_unlock_address: Default::default(),
                    redeemer_lockscript: Default::default(),
                    liquidation_trigger_lockscript: Default::default(),
                    x_extra: XExtraView::Eth(Default::default()),
                },
                type_args: ToCKBTypeArgsView {
                    xchain_kind: 2,
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
                    amount: 249500000000000000,
                    lockscript: Default::default(),
                    owner_script: Default::default(),
                    index: 1,
                },
                SudtCell {
                    capacity: XT_CELL_CAPACITY,
                    amount: 500000000000000,
                    lockscript: Default::default(),
                    owner_script: Default::default(),
                    index: 2,
                },
            ],
        },
        capacity_cells: Default::default(),
        witnesses: vec![Witness::Eth(EthWitness {
            cell_dep_index_list: vec![0],
            spv_proof: ETHSPVProofJson {
                    log_index: 0,
                    log_entry_data:"f89b94dac17f958d2ee523a2206206994597c13d831ec7f863a0ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3efa00000000000000000000000006cc5f688a315f3dc28a7781717a9a798a59fda7ba00000000000000000000000007e7a32d9dc98c485c489be8e732f97b4ffe3a4cda000000000000000000000000000000000000000000000000000000001a13b8600".to_owned(),
                    receipt_index: 0,
                    receipt_data:"f901a60182d0d9b9010000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000010000000000000000000000000000000000000000000000000000000408000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000010000000000000000000000000000000000000000000000000000000400000000000100000000000000000000000000080000000000000000000000000000000000000000000100002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000f89df89b94dac17f958d2ee523a2206206994597c13d831ec7f863a0ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3efa00000000000000000000000006cc5f688a315f3dc28a7781717a9a798a59fda7ba00000000000000000000000007e7a32d9dc98c485c489be8e732f97b4ffe3a4cda000000000000000000000000000000000000000000000000000000001a13b8600".to_owned(),
                    header_data: "f9021aa0f779e50b45bc27e4ed236840e5dbcf7afab50beaf553be56bf76da977e10cc73a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d493479452bc44d5378309ee2abf1539bf71de1b7d7be3b5a014c996b6934d7991643669e145b8355c63aa02cbde63d390fcf4e6181d5eea45a079b7e79dc739c31662fe6f25f65bf5a5d14299c7a7aa42c3f75b9fb05474f54ca0e28dc05418692cb7baab7e7f85c1dedb8791c275b797ea3b1ffcaec5ef2aa271b9010000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000010000000000000000000000000000000000000000000000000000000408000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000010000000000000000000000000000000000000000000000000000000400000000000100000000000000000000000000080000000000000000000000000000000000000000000100002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000903234373439353837313930323034343383890fe68395ba8e82d0d9845dd84a079150505945206e616e6f706f6f6c2e6f7267a0a35425f443452cf94ba4b698b00fd7b3ff4fc671dea3d5cc2dcbedbc3766f45e88af7fec6031063a17".to_owned(),
                    proof: vec![
                        vec![
                            Vec::from_hex("2080").unwrap(),
                            Vec::from_hex("f901a60182d0d9b9010000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000010000000000000000000000000000000000000000000000000000000408000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000010000000000000000000000000000000000000000000000000000000400000000000100000000000000000000000000080000000000000000000000000000000000000000000100002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000f89df89b94dac17f958d2ee523a2206206994597c13d831ec7f863a0ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3efa00000000000000000000000006cc5f688a315f3dc28a7781717a9a798a59fda7ba00000000000000000000000007e7a32d9dc98c485c489be8e732f97b4ffe3a4cda000000000000000000000000000000000000000000000000000000001a13b8600").unwrap(),
                            ],
                    ].iter().map(|node| {
                        let mut stream = RlpStream::new();
                        stream.begin_list(node.len());
                        for item in node {
                        stream.append(item);
                        }
                        stream.out()
                    }).collect(),
            }
        })],
        expect_return_code: 0,
    }
}


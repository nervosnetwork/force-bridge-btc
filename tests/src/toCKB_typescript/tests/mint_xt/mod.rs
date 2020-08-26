use super::{Script, ToCKBCellData};
use crate::toCKB_typescript::utils::types::{basic, btc_difficulty, mint_xt_witness};
use crate::*;
use anyhow::Result;
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_tool::ckb_types::{
    bytes::Bytes,
    core::{TransactionBuilder, TransactionView},
    packed::*,
    prelude::*,
};
use ckb_tool::{ckb_error::assert_error_eq, ckb_script::ScriptError};
use int_enum::IntEnum;
use molecule::prelude::*;
use primitive_types::U256;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

const MAX_CYCLES: u64 = 100_000_000;
// const PLEDGE_INVALID: i8 = 8;
// const LOT_SIZE_INVALID: i8 = 7;
// const TX_INVALID: i8 = 6;
// const ENCODING: i8 = 4;

#[repr(u8)]
#[derive(Clone, Copy, IntEnum)]
pub enum ToCKBStatus {
    Initial = 1,
    Bonded = 2,
    Warranty = 3,
    Redeeming = 4,
    SignerTimeout = 5,
    Undercollateral = 6,
    FaultyWhenWarranty = 7,
    FaultyWhenRedeeming = 8,
}

#[test]
fn test_correct_tx() {
    let kind = 1;
    let pledge = 10000;
    let coin_type = 1;

    let mut context = Context::default();
    let toCKB_typescript_bin: Bytes = Loader::default().load_binary("toCKB-typescript");
    let toCKB_typescript_out_point = context.deploy_cell(toCKB_typescript_bin);
    let toCKB_lockscript_bin: Bytes = Loader::default().load_binary("toCKB-lockscript");
    let toCKB_lockscript_out_point = context.deploy_cell(toCKB_lockscript_bin);
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

    // prepare scripts
    let toCKB_typescript = context
        .build_script(&toCKB_typescript_out_point, [kind; 1].to_vec().into())
        .expect("script");
    let toCKB_typescript_dep = CellDep::new_builder()
        .out_point(toCKB_typescript_out_point)
        .build();
    let always_success_lockscript = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let always_success_lockscript_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    // prepare cells
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(11000u64.pack())
            .lock(always_success_lockscript.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    let signer_lockscript = toCKB_typescript::utils::types::basic::Script::new_builder().build();
    let user_lockscript = toCKB_typescript::utils::types::basic::Script::new_builder().build();
    let x_lock_address = toCKB_typescript::utils::types::basic::Bytes::new_builder()
        .set([Byte::new(1u8); 20].to_vec().into())
        .build();
    let input_toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(ToCKBStatus::Bonded.int_value()))
        .lot_size(Byte::new(1u8))
        .signer_lockscript(signer_lockscript.clone())
        .user_lockscript(user_lockscript.clone())
        .x_lock_address(x_lock_address.clone())
        .build();
    let output_toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(ToCKBStatus::Warranty.int_value()))
        .lot_size(Byte::new(1u8))
        .signer_lockscript(signer_lockscript.clone())
        .user_lockscript(user_lockscript.clone())
        .x_lock_address(x_lock_address.clone())
        .build();

    let input_ckb_cell_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(11000u64.pack())
            .lock(always_success_lockscript.clone())
            .type_(Some(toCKB_typescript.clone()).pack())
            .build(),
        input_toCKB_data.as_bytes(),
    );
    let input_ckb_cell = CellInput::new_builder()
        .previous_output(input_ckb_cell_out_point)
        .build();
    let inputs = vec![input_ckb_cell];
    let outputs = vec![CellOutput::new_builder()
        .capacity(11000u64.pack())
        .type_(Some(toCKB_typescript.clone()).pack())
        .lock(always_success_lockscript)
        .build()];
    let outputs_data = vec![output_toCKB_data.as_bytes()];
    // raw_header: 00000020acf05cadf6d066d01f5aca661690f4e1779a8144b90b070000000000000000006bbb5a7851af48d883e8ac5d6f61c6ad9a4132a9a12531c1b6f085760b3b2e427ba0455fea0710177d792e86
    // raw_tx: 020000000001015227c5fbad9d9202ade7f02452cf880dac1ed270255ebfe6716e8b3e8956571d0100000017160014085fc2ea0c102fc4db8dbbb10dd6f93684c178c9feffffff028c79171300000000160014173ec3a12e289b102f8edcc1d4ecd3b5b893e2dc97b2030000000000160014ef9665bcf82fa83e870a350a6551a09ee819e4a30247304402201fc61e35096b9ba44a5a845932acce596fde7be7583acd9950d25bb937d49632022032b7666d8d72a553ec9376107a6d25444a942ae8b2eb77ac11d71593666f2c7f0121032dcec3808ef6a58b14e8b1ac77e7b5c47ea46120cd429b8c1eae0970b9c87e1edbd80900
    // raw_tx:
    // - 02000000   // version
    // - 0001   // flag
    // - 01     // one input
    //   - 5227c5fbad9d9202ade7f02452cf880dac1ed270255ebfe6716e8b3e8956571d01000000   // outpoint
    //   - 17   // script sig len
    //   - 160014085fc2ea0c102fc4db8dbbb10dd6f93684c178c9   // script sig
    //   - feffffff     // sequence
    // - 02     // 2 output
    //   - 8c79171000000000     // output 1 value
    //   - 16  // len
    //   - 0014173ec3a12e289b102f8edcc1d4ecd3b5b893e2dc     // script
    //   - 97b2030000000000     // output 2
    //   - 16
    //   - 0014ef9665bcf82fa83e870a350a6551a09ee819e4a3
    // - 02     // 2 witness
    //   - 47
    //   - 304402201fc61e35096b9ba44a5a845932acce596fde7be7583acd9950d25bb937d49632022032b7666d8d72a553ec9376107a6d25444a942ae8b2eb77ac11d71593666f2c7f01  // witness 1
    //   - 21
    //   - 032dcec3808ef6a58b14e8b1ac77e7b5c47ea46120cd429b8c1eae0970b9c87e1e  // witness 2
    // - dbd80900  // locktime
    // 02000000015227c5fbad9d9202ade7f02452cf880dac1ed270255ebfe6716e8b3e8956571d0100000017160014085fc2ea0c102fc4db8dbbb10dd6f93684c178c9feffffff028c79171300000000160014173ec3a12e289b102f8edcc1d4ecd3b5b893e2dc97b2030000000000160014ef9665bcf82fa83e870a350a6551a09ee819e4a3dbd80900
    // let btc_spv_proof_str = r#"
    // {
    //     "version": "0x02000000",
    //     "vin": "0x015227c5fbad9d9202ade7f02452cf880dac1ed270255ebfe6716e8b3e8956571d0100000017160014085fc2ea0c102fc4db8dbbb10dd6f93684c178c9feffffff",
    //     "vout": "0x028c79171300000000160014173ec3a12e289b102f8edcc1d4ecd3b5b893e2dc97b2030000000000160014ef9665bcf82fa83e870a350a6551a09ee819e4a3",
    //     "locktime": "0xdbd80900",
    //     "tx_id": "0xc33b712400ac3c262272ea6f0ddbfeab56d7786c84b2419ec25cf1e66a84212b",
    //     "index": 0,
    //     "headers": "0x00000020acf05cadf6d066d01f5aca661690f4e1779a8144b90b070000000000000000006bbb5a7851af48d883e8ac5d6f61c6ad9a4132a9a12531c1b6f085760b3b2e427ba0455fea0710177d792e86",
    //     "intermediate_nodes": "0x",
    //     "funding_output_index": 0
    // }
    // "#;
    let btc_spv_proof_str = r#"
    {
        "version": "0x01000000",
        "vin": "0x0101748906a5c7064550a594c4683ffc6d1ee25292b638c4328bb66403cfceb58a000000006a4730440220364301a77ee7ae34fa71768941a2aad5bd1fa8d3e30d4ce6424d8752e83f2c1b02203c9f8aafced701f59ffb7c151ff2523f3ed1586d29b674efb489e803e9bf93050121029b3008c0fa147fd9db5146e42b27eb0a77389497713d3aad083313d1b1b05ec0ffffffff",
        "vout": "0x0316312f00000000001976a91400cc8d95d6835252e0d95eb03b11691a21a7bac588ac220200000000000017a914e5034b9de4881d62480a2df81032ef0299dcdc32870000000000000000166a146f6d6e69000000000000001f0000000315e17900",
        "locktime": "0x00000000",
        "tx_id": "0x5176f6b03b8bc29f4deafbb7384b673debde6ae712deab93f3b0c91fdcd6d674",
        "index": 26,
        "headers": "0x0000c020c238b601308b7297346ab2ed59942d7d7ecea8d23a1001000000000000000000b61ac92842abc82aa93644b190fc18ad46c6738337e78bc0c69ab21c5d5ee2ddd6376d5d3e211a17d8706a84",
        "intermediate_nodes": "0x8d7a6d53ce27f79802631f1aae5f172c43d128b210ab4962d488c81c96136cfb75c95def872e878839bd93b42c04eb44da44c401a2d580ca343c3262e9c0a2819ed4bbfb9ea620280b31433f43b2512a893873b8c8c679f61e1a926c0ec80bcfc6225a15d72fbd1116f78b14663d8518236b02e765bf0a746a6a08840c122a02afa4df3ab6b9197a20f00495a404ee8e07da2b7554e94609e9ee1d5da0fb7857ea0332072568d0d53a9aedf851892580504a7fcabfbdde076242eb7f4e5f218a14d2a3f357d950b4f6a1dcf93f7c19c44d0fc122d00afa297b9503c1a6ad24cf36cb5f2835bcf490371db2e96047813a24176c3d3416f84b7ddfb7d8c915eb0c5ce7de089b5d9e700ecd12e09163f173b70bb4c9af33051b466b1f55abd66f3121216ad0ad9dfa898535e1d5e51dd07bd0a73d584daace7902f20ece4ba4f4f241c80cb31eda88a244a3c68d0f157c1049b4153d7addd6548aca0885acafbf98a1f8345c89914c24729ad095c7a0b9acd20232ccd90dbd359468fcc4eee7b67d",
        "funding_output_index": 0
    }
    "#;
    let btc_spv_proof = json_to_btc_proof(btc_spv_proof_str).unwrap();
    let witness_data = mint_xt_witness::MintXTWitness::new_builder()
        .spv_proof(btc_spv_proof.as_slice().to_vec().into())
        .cell_dep_index_list(vec![2].into())
        .build();
    let witness = WitnessArgs::new_builder()
        .input_type(Some(witness_data.as_bytes()).pack())
        .build();

    // 0xFF809F0159F
    let diff = 10771996663680u64;
    let difficulty_data = btc_difficulty::BTCDifficulty::new_builder()
        .previous(diff.to_le_bytes().to_vec().into())
        .current(diff.to_le_bytes().to_vec().into())
        .build();

    dbg!(&difficulty_data);
    let difficulty_data_out_point = context.deploy_cell(difficulty_data.as_bytes());
    let difficulty_data_dep = CellDep::new_builder()
        .out_point(difficulty_data_out_point)
        .build();

    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(toCKB_typescript_dep)
        .cell_dep(always_success_lockscript_dep)
        .cell_dep(difficulty_data_dep)
        .witness(witness.as_bytes().pack())
        .build();

    // let tx = context.complete_tx(tx);
    // let toCKB_data = ToCKBCellData::new_builder()
    //     .status(Byte::new(1u8))
    //     .lot_size(Byte::new(1u8))
    //     .user_lockscript(Script::new_builder().build())
    //     .build();
    // let (context, tx) = build_test_context(1, 10000, toCKB_data.as_bytes());

    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[derive(Serialize, Deserialize)]
struct BTCSPVProofJson {
    version: String,
    vin: String,
    vout: String,
    locktime: String,
    tx_id: String,
    index: u64,
    headers: String,
    intermediate_nodes: String,
    funding_output_index: u8,
}

impl TryFrom<BTCSPVProofJson> for mint_xt_witness::BTCSPVProof {
    type Error = anyhow::Error;

    fn try_from(proof: BTCSPVProofJson) -> Result<Self> {
        Ok(mint_xt_witness::BTCSPVProof::new_builder()
            .version(hex::decode(clear_0x(&proof.version))?.into())
            .vin(hex::decode(clear_0x(&proof.vin))?.into())
            .vout(hex::decode(clear_0x(&proof.vout))?.into())
            .locktime(hex::decode(clear_0x(&proof.locktime))?.into())
            .tx_id(hex::decode(clear_0x(&proof.tx_id))?.into())
            .index(proof.index.into())
            .headers(hex::decode(clear_0x(&proof.headers))?.into())
            .intermediate_nodes(hex::decode(clear_0x(&proof.intermediate_nodes))?.into())
            .funding_output_index(proof.funding_output_index.into())
            .build())
    }
}

fn clear_0x(s: &str) -> &str {
    if &s[..2] == "0x" || &s[..2] == "0X" {
        &s[2..]
    } else {
        s
    }
}

fn json_to_btc_proof(proof: &str) -> Result<mint_xt_witness::BTCSPVProof> {
    let proof: BTCSPVProofJson = serde_json::from_str(proof)?;
    proof.try_into()
}

// #[test]
// fn test_wrong_pledge() {
//     let toCKB_data = ToCKBCellData::new_builder()
//         .status(Byte::new(1u8))
//         .lot_size(Byte::new(1u8))
//         .user_lockscript(Script::new_builder().build())
//         .build();
//     let (context, tx) = build_test_context(1, 9999, toCKB_data.as_bytes());

//     let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
//     assert_error_eq!(err, ScriptError::ValidationFailure(PLEDGE_INVALID));
// }

// fn build_test_context(kind: u8, input: Bytes) -> (Context, TransactionView) {
//     // deploy contract
//     let mut context = Context::default();
//     let toCKB_typescript_bin: Bytes = Loader::default().load_binary("toCKB-typescript");
//     let toCKB_typescript_out_point = context.deploy_cell(toCKB_typescript_bin);
//     let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

//     // prepare scripts
//     let toCKB_typescript = context
//         .build_script(&toCKB_typescript_out_point, [kind; 1].to_vec().into())
//         .expect("script");
//     let toCKB_typescript_dep = CellDep::new_builder()
//         .out_point(toCKB_typescript_out_point)
//         .build();
//     let always_success_lockscript = context
//         .build_script(&always_success_out_point, Default::default())
//         .expect("script");
//     let always_success_lockscript_dep = CellDep::new_builder()
//         .out_point(always_success_out_point)
//         .build();

//     // prepare cells
//     let input_out_point = context.create_cell(
//         CellOutput::new_builder()
//             .capacity(11000u64.pack())
//             .lock(always_success_lockscript.clone())
//             .build(),
//         Bytes::new(),
//     );
//     let input = CellInput::new_builder()
//         .previous_output(input_out_point)
//         .build();
//     let outputs = vec![CellOutput::new_builder()
//         .capacity(pledge.pack())
//         .type_(Some(toCKB_typescript.clone()).pack())
//         .lock(always_success_lockscript)
//         .build()];
//     let outputs_data = vec![toCKB_data; 1];

//     // build transaction
//     let tx = TransactionBuilder::default()
//         .input(input)
//         .outputs(outputs)
//         .outputs_data(outputs_data.pack())
//         .cell_dep(toCKB_typescript_dep)
//         .cell_dep(always_success_lockscript_dep)
//         .build();
//     let tx = context.complete_tx(tx);

//     (context, tx)
// }

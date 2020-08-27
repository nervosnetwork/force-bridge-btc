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
    let sudt_bin = include_bytes!("../simple_udt");
    let sudt_out_point = context.deploy_cell(Bytes::from(sudt_bin.as_ref()));

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
    let lock_hash: [u8; 32] = always_success_lockscript.calc_script_hash().unpack();
    // let lock_hash = [0u8; 32];
    dbg!(hex::encode(lock_hash));
    let sudt_script_args: Bytes = lock_hash.to_vec().into();
    let sudt_typescript = context
        .build_script(&sudt_out_point, sudt_script_args)
        .expect("script");
    dbg!(sudt_typescript.code_hash());
    dbg!(sudt_typescript.args());
    // dbg!(sudt_typescript.code_hash().raw_data().to_vec());
    let sudt_typescript_dep = CellDep::new_builder().out_point(sudt_out_point).build();

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

    let user_lockscript = toCKB_typescript::utils::types::basic::Script::from_slice(
        always_success_lockscript.as_slice(),
    )
    .unwrap();
    let signer_lockscript = user_lockscript.clone();
    let x_lock_address_str = b"bc1qq2pw0kr5yhz3xcs978desw5anfmtwynutwq8quz0t";
    let x_lock_address = toCKB_typescript::utils::types::basic::Bytes::new_builder()
        // .set([Byte::new(1u8); 20].to_vec().into())
        .set(
            x_lock_address_str
                .iter()
                .map(|c| Byte::new(*c))
                .collect::<Vec<_>>()
                .into(),
        )
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
    let outputs = vec![
        CellOutput::new_builder()
            .capacity(11000u64.pack())
            .type_(Some(toCKB_typescript.clone()).pack())
            .lock(always_success_lockscript.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(11000u64.pack())
            .type_(Some(sudt_typescript.clone()).pack())
            .lock(always_success_lockscript.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(11000u64.pack())
            .type_(Some(sudt_typescript.clone()).pack())
            .lock(always_success_lockscript.clone())
            .build(),
    ];
    let outputs_data = vec![
        output_toCKB_data.as_bytes(),
        24950000u128.to_le_bytes().to_vec().into(),
        50000u128.to_le_bytes().to_vec().into(),
    ];
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
    let btc_spv_proof_str = r#"
    {
        "version": "0x02000000",
        "vin": "0x015227c5fbad9d9202ade7f02452cf880dac1ed270255ebfe6716e8b3e8956571d0100000017160014085fc2ea0c102fc4db8dbbb10dd6f93684c178c9feffffff",
        "vout": "0x028c79171300000000160014173ec3a12e289b102f8edcc1d4ecd3b5b893e2dc97b2030000000000160014ef9665bcf82fa83e870a350a6551a09ee819e4a3",
        "locktime": "0xdbd80900",
        "tx_id": "0x2b21846ae6f15cc29e41b2846c78d756abfedb0d6fea7222263cac0024713bc3",
        "index": 3,
        "headers": "0x00000020acf05cadf6d066d01f5aca661690f4e1779a8144b90b070000000000000000006bbb5a7851af48d883e8ac5d6f61c6ad9a4132a9a12531c1b6f085760b3b2e427ba0455fea0710177d792e86",
        "intermediate_nodes": "0x8546dfccb488115f9c3210255523c0e186fb9b64d16ac68b3d8903bf037dc3ab26069e90c930cc55105d5f8b4ddd798bc33f057641e748fd2e70de0b8747cae802af46fb1e1fccf354b4b46d87f5a85c564fd5284cbe2a5711c16c446fbb6e9e0b3c7beec06a156a8005883b8cf224f665d361a2269b6b21491c1ccbb8160c311b609b5ca21b0a9f708e6124b36871b71c5536d8d556054be435cf0444da70d0814e678eb0e081805d777f9cf84911f9e04b6a80b6cf60dec31527ec73aaa8ba77ec6bff2e04fbb80c8c81b1cc38b415bc21dd732f51a4a903ee265b0eef2c589f751e66e46bb02aa36ed8418ae93317316b84d12f1b1702dd9641ead0ad7f8777526ad7a4ff599946d219a7a932ec8cd2e42649b3d5fa123d2e4532de6d46bddb27a8c02de8fb8fe2c4d88a14132de8cdd7d471bc6a8c8c217aeec600fd295e8925b663332f45bdb6877dd6e0ecd28bfae530ba3ed8bd3959644a82bc418f9c887746e15ae55d82369c3761187ea449c7f7bdff1acaa0b467e1335b3919089d",
        "funding_output_index": 0
    }
    "#;
    let btc_spv_proof = json_to_btc_proof(btc_spv_proof_str).unwrap();
    let witness_data = mint_xt_witness::MintXTWitness::new_builder()
        .spv_proof(btc_spv_proof.as_slice().to_vec().into())
        .cell_dep_index_list(vec![0].into())
        .build();
    let witness = WitnessArgs::new_builder()
        .input_type(Some(witness_data.as_bytes()).pack())
        .build();

    let diff = 17557993035167u64;
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
        .cell_dep(difficulty_data_dep)
        .cell_dep(toCKB_typescript_dep)
        .cell_dep(always_success_lockscript_dep)
        .cell_dep(sudt_typescript_dep)
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

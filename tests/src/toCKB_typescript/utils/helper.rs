use crate::toCKB_typescript::utils::types::{
    generated::{
        basic, btc_difficulty, mint_xt_witness, BtcExtra, Byte32, ToCKBCellData, Uint32, XExtra,
        XExtraUnion,
    },
    test_case::*,
    ToCKBStatus,
};
use crate::*;
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_tool::ckb_types::{bytes::Bytes, core::TransactionBuilder, packed::*, prelude::*};
use molecule::prelude::*;
use std::vec::Vec;

pub const MAX_CYCLES: u64 = 100_000_000;
pub const PLEDGE: u64 = 10000;
pub const XT_CELL_CAPACITY: u64 = 200;

pub fn run_test_case(case: TestCase) {
    let kind = case.kind;

    let mut context = Context::default();
    let toCKB_typescript_bin: Bytes = Loader::default().load_binary("toCKB-typescript");
    let toCKB_typescript_out_point = context.deploy_cell(toCKB_typescript_bin);
    // let toCKB_lockscript_bin: Bytes = Loader::default().load_binary("toCKB-lockscript");
    // let toCKB_lockscript_out_point = context.deploy_cell(toCKB_lockscript_bin);
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let sudt_bin = include_bytes!("../../../deps/simple_udt");
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
    let _sudt_typescript = context
        .build_script(&sudt_out_point, sudt_script_args)
        .expect("script");
    let sudt_typescript_dep = CellDep::new_builder().out_point(sudt_out_point).build();

    // prepare cells
    let x_lock_address_str = case.tockb_cell_data.x_lock_address;
    let x_lock_address = basic::Bytes::new_builder()
        .set(
            x_lock_address_str
                .as_bytes()
                .iter()
                .map(|c| Byte::new(*c))
                .collect::<Vec<_>>()
                .into(),
        )
        .build();
    let signer_lockscript =
        basic::Script::from_slice(case.tockb_cell_data.signer_lockscript.as_slice()).unwrap();
    let user_lockscript =
        basic::Script::from_slice(case.tockb_cell_data.user_lockscript.as_slice()).unwrap();
    let input_toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(case.status))
        .lot_size(Byte::new(case.tockb_cell_data.lot_size))
        .signer_lockscript(signer_lockscript.clone())
        .user_lockscript(user_lockscript.clone())
        .x_lock_address(x_lock_address.clone())
        .build();

    let x_extra = match case.tockb_cell_data.x_extra {
        XExtraView::Btc(btc_extra) => {
            let lock_tx_hash = Byte32::new_unchecked(btc_extra.lock_tx_hash);
            let lock_vout_index = Vec::<u8>::from(&btc_extra.lock_vout_index.to_le_bytes()[..]);
            let lock_vout_index = Uint32::new_unchecked(Bytes::from(lock_vout_index));
            let btc_extra = BtcExtra::new_builder()
                .lock_tx_hash(lock_tx_hash)
                .lock_vout_index(lock_vout_index)
                .build();
            let x_extra = XExtraUnion::BtcExtra(btc_extra);
            XExtra::new_builder().set(x_extra).build()
        }
        XExtraView::Eth(_eth_extra) => todo!(),
    };
    let output_toCKB_data = ToCKBCellData::new_builder()
        .status(Byte::new(ToCKBStatus::Warranty as u8))
        .lot_size(Byte::new(case.tockb_cell_data.lot_size))
        .signer_lockscript(signer_lockscript.clone())
        .user_lockscript(user_lockscript.clone())
        .x_lock_address(x_lock_address.clone())
        .x_extra(x_extra)
        .build();

    let input_ckb_cell_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(case.input_capacity.pack())
            .lock(always_success_lockscript.clone())
            .type_(Some(toCKB_typescript.clone()).pack())
            .build(),
        input_toCKB_data.as_bytes(),
    );
    let input_ckb_cell = CellInput::new_builder()
        .previous_output(input_ckb_cell_out_point)
        .build();
    let inputs = vec![input_ckb_cell];
    let mut outputs = vec![CellOutput::new_builder()
        .capacity(case.output_capacity.pack())
        .type_(Some(toCKB_typescript.clone()).pack())
        .lock(always_success_lockscript.clone())
        .build()];
    let mut outputs_data = vec![output_toCKB_data.as_bytes()];
    for output in case.outputs.into_iter() {
        let cell_output = CellOutput::new_builder()
            .capacity(output.capacity.pack())
            .type_(Some(output.typescript).pack())
            .lock(output.lockscript)
            .build();
        outputs.push(cell_output);
        outputs_data.push(output.amount.to_le_bytes().to_vec().into())
    }
    let spv_proof = match case.witness.spv_proof {
        SpvProof::BTC(btc_spv_proof) => btc_spv_proof.as_slice().to_vec(),
    };
    let witness_data = mint_xt_witness::MintXTWitness::new_builder()
        .spv_proof(spv_proof.into())
        .cell_dep_index_list(case.witness.cell_dep_index_list.into())
        .build();
    let witness = WitnessArgs::new_builder()
        .input_type(Some(witness_data.as_bytes()).pack())
        .build();
    let dep_data = match case.cell_deps_data {
        CellDepsData::BTC(difficulty_data) => {
            let data = btc_difficulty::BTCDifficulty::new_builder()
                .previous(difficulty_data.previous.to_le_bytes().to_vec().into())
                .current(difficulty_data.current.to_le_bytes().to_vec().into())
                .build();
            dbg!(&data);
            data.as_bytes()
        }
    };
    let data_out_point = context.deploy_cell(dep_data);
    let data_dep = CellDep::new_builder().out_point(data_out_point).build();

    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(data_dep)
        .cell_dep(toCKB_typescript_dep)
        .cell_dep(always_success_lockscript_dep)
        .cell_dep(sudt_typescript_dep)
        .witness(witness.as_bytes().pack())
        .build();

    let res = context.verify_tx(&tx, MAX_CYCLES);
    dbg!(&res);
    match res {
        Ok(_cycles) => assert_eq!(case.expect_return_code, 0),
        Err(err) => assert!(check_err(err, case.expect_return_code)),
    }
}

pub fn check_err(err: ckb_tool::ckb_error::Error, code: i8) -> bool {
    let get = format!("{}", err);
    let expected = format!("Script(ValidationFailure({}))", code);
    dbg!(&get, &expected);
    get == expected
}

pub struct DeployResult {
    pub context: Context,
    pub toCKB_typescript: Script,
    pub always_success_lockscript: Script,
    pub sudt_typescript: Script,
}

pub fn deploy(kind: u8) -> DeployResult {
    let mut context = Context::default();
    let toCKB_typescript_bin: Bytes = Loader::default().load_binary("toCKB-typescript");
    let toCKB_typescript_out_point = context.deploy_cell(toCKB_typescript_bin);
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let sudt_bin = include_bytes!("../../../deps/simple_udt");
    let sudt_out_point = context.deploy_cell(Bytes::from(sudt_bin.as_ref()));

    // prepare scripts
    let toCKB_typescript = context
        .build_script(&toCKB_typescript_out_point, [kind; 1].to_vec().into())
        .expect("script");
    let always_success_lockscript = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let lock_hash: [u8; 32] = always_success_lockscript.calc_script_hash().unpack();
    let sudt_script_args: Bytes = lock_hash.to_vec().into();
    let sudt_typescript = context
        .build_script(&sudt_out_point, sudt_script_args)
        .expect("script");

    DeployResult {
        context,
        toCKB_typescript,
        always_success_lockscript,
        sudt_typescript,
    }
}

use crate::switch::ToCKBCellDataTuple;
use crate::utils::{
    config::{PLEDGE, SIGNER_FEE_RATE, SUDT_CODE_HASH, XT_CELL_CAPACITY},
    tools::{is_XT_typescript, verify_btc_witness, XChainKind},
    types::{mint_xt_witness::MintXTWitnessReader, Error, ToCKBCellDataView, XExtraView},
};
use ckb_std::{
    ckb_constants::Source,
    debug,
    high_level::{
        load_cell_capacity, load_cell_data, load_cell_lock, load_cell_lock_hash, load_cell_type,
        load_witness_args, QueryIter,
    },
};
use core::result::Result;
use molecule::prelude::{Entity, Reader};

fn verify_data(
    input_data: &ToCKBCellDataView,
    output_data: &ToCKBCellDataView,
    x_extra: &XExtraView,
) -> Result<(), Error> {
    if input_data.signer_lockscript.as_ref() != output_data.signer_lockscript.as_ref()
        || input_data.user_lockscript.as_ref() != output_data.user_lockscript.as_ref()
        || input_data.get_raw_lot_size() != output_data.get_raw_lot_size()
        || input_data.x_lock_address.as_ref() != output_data.x_lock_address.as_ref()
        || &output_data.x_extra != x_extra
    {
        return Err(Error::InvalidDataChange);
    }
    Ok(())
}

/// ensure transfer happen on XChain by verifying the spv proof
fn verify_witness(data: &ToCKBCellDataView) -> Result<XExtraView, Error> {
    let witness_args = load_witness_args(0, Source::GroupInput)?.input_type();
    debug!("witness_args: {:?}", &witness_args);
    if witness_args.is_none() {
        return Err(Error::InvalidWitness);
    }
    let witness_args = witness_args.to_opt().unwrap().raw_data();
    debug!("witness_args parsed: {:?}", &witness_args);
    if MintXTWitnessReader::verify(&witness_args, false).is_err() {
        return Err(Error::InvalidWitness);
    }
    let witness = MintXTWitnessReader::new_unchecked(&witness_args);
    debug!("witness: {:?}", witness);
    let proof = witness.spv_proof().raw_data();
    let cell_dep_index_list = witness.cell_dep_index_list().raw_data();
    match data.get_xchain_kind() {
        XChainKind::Btc => {
            let btc_extra = verify_btc_witness(
                data,
                proof,
                cell_dep_index_list,
                data.x_lock_address.as_ref(),
                data.get_btc_lot_size()?.get_sudt_amount(),
            )?;
            Ok(XExtraView::Btc(btc_extra))
        }
        XChainKind::Eth => todo!(),
    }
}

fn verify_xt_issue(data: &ToCKBCellDataView) -> Result<(), Error> {
    match data.get_xchain_kind() {
        XChainKind::Btc => verify_btc_xt_issue(data),
        XChainKind::Eth => todo!(),
    }
}

fn verify_btc_xt_issue(data: &ToCKBCellDataView) -> Result<(), Error> {
    let lock_hash = load_cell_lock_hash(0, Source::GroupInput)?;
    debug!("lockscript hash: {:?}", hex::encode(lock_hash));
    let input_xt_num = QueryIter::new(load_cell_type, Source::Input)
        .filter(|type_opt| type_opt.is_some())
        .map(|type_opt| type_opt.unwrap())
        .filter(|script| is_XT_typescript(script, lock_hash.as_ref()))
        .count();
    if input_xt_num != 0 {
        return Err(Error::InvalidXTInInputOrOutput);
    }
    let output_xt_num = QueryIter::new(load_cell_type, Source::Output)
        .filter(|type_opt| type_opt.is_some())
        .map(|type_opt| type_opt.unwrap())
        .filter(|script| is_XT_typescript(script, lock_hash.as_ref()))
        .count();
    debug!("output_xt_num: {}", output_xt_num);
    if output_xt_num != 2 {
        return Err(Error::InvalidXTInInputOrOutput);
    }
    let xt_amount = data.get_btc_lot_size()?.get_sudt_amount();
    debug!("xt_amount: {}", xt_amount);
    // fixed order of output cells is required
    // user-sudt-cell should be outputs[1]
    // signer-sudt-cell should be outputs[2]
    let expect = [
        (
            1,
            data.user_lockscript.as_ref(),
            xt_amount - xt_amount * SIGNER_FEE_RATE.0 / SIGNER_FEE_RATE.1,
        ),
        (
            2,
            data.signer_lockscript.as_ref(),
            xt_amount * SIGNER_FEE_RATE.0 / SIGNER_FEE_RATE.1,
        ),
    ];
    debug!("expect: {:?}", expect);

    for (i, lockscript, amount) in expect.iter() {
        let script = load_cell_type(*i, Source::Output)?;
        if script.is_none() {
            return Err(Error::InvalidMintOutput);
        }
        let script = script.unwrap();
        if !(script.code_hash().raw_data().as_ref() == SUDT_CODE_HASH.as_ref()
            && script.args().raw_data().as_ref() == lock_hash.as_ref()
            && script.hash_type() == 0u8.into())
        {
            return Err(Error::InvalidMintOutput);
        }
        let cell_data = load_cell_data(*i, Source::Output)?;
        let mut amount_vec = [0u8; 16];
        amount_vec.copy_from_slice(&cell_data);
        let token_amount = u128::from_le_bytes(amount_vec);
        debug!("token_amount: {}, amout: {}", token_amount, amount);
        if token_amount != *amount {
            return Err(Error::InvalidMintOutput);
        }
        let lock = load_cell_lock(*i, Source::Output)?;
        debug!(
            "lock: {:?}, expect lock: {:?}",
            hex::encode(lock.as_slice()),
            hex::encode(lockscript.as_ref())
        );
        if lock.as_slice() != lockscript.as_ref() {
            return Err(Error::InvalidMintOutput);
        }
    }
    Ok(())
}

pub fn verify_capacity() -> Result<(), Error> {
    let toCKB_output_cap = load_cell_capacity(0, Source::GroupOutput)?;
    let toCKB_input_cap = load_cell_capacity(0, Source::GroupInput)?;
    if toCKB_input_cap - toCKB_output_cap != PLEDGE + XT_CELL_CAPACITY {
        return Err(Error::CapacityInvalid);
    }
    let user_xt_cell_cap = load_cell_capacity(1, Source::Output)?;
    if user_xt_cell_cap != PLEDGE {
        return Err(Error::CapacityInvalid);
    }
    let signer_xt_cell_cap = load_cell_capacity(2, Source::Output)?;
    if signer_xt_cell_cap != XT_CELL_CAPACITY {
        return Err(Error::CapacityInvalid);
    }
    Ok(())
}

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    debug!("start mint_xt");
    let input_data = toCKB_data_tuple.0.as_ref().expect("should not happen");
    let output_data = toCKB_data_tuple.1.as_ref().expect("should not happen");
    verify_capacity()?;
    let x_extra = verify_witness(input_data)?;
    debug!("verify witness finish");
    verify_data(input_data, output_data, &x_extra)?;
    debug!("verify data finish");
    verify_xt_issue(input_data)?;
    debug!("verify xt issue finish");
    Ok(())
}

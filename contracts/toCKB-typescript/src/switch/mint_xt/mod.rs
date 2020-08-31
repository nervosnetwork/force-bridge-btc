use crate::switch::ToCKBCellDataTuple;
use crate::utils::{
    config::{SIGNER_FEE_RATE, SUDT_CODE_HASH, TX_PROOF_DIFFICULTY_FACTOR},
    tools::{get_xchain_kind, XChainKind},
    types::{
        btc_difficulty::BTCDifficultyReader,
        mint_xt_witness::{BTCSPVProofReader, MintXTWitnessReader},
        Error, ToCKBCellDataView,
    },
};
use alloc::string::String;
use bech32::ToBase32;
use bitcoin_spv::{
    btcspv,
    types::{HeaderArray, MerkleArray, PayloadType, Vin, Vout},
    validatespv,
};
use ckb_std::{
    ckb_constants::Source,
    debug,
    high_level::{
        load_cell_data, load_cell_lock, load_cell_lock_hash, load_cell_type, load_witness_args,
        QueryIter,
    },
};
use core::result::Result;
use molecule::prelude::{Entity, Reader};
use primitive_types::U256;

fn verify_data(
    input_data: &ToCKBCellDataView,
    output_data: &ToCKBCellDataView,
) -> Result<(), Error> {
    if input_data.signer_lockscript.as_ref() != output_data.signer_lockscript.as_ref()
        || input_data.user_lockscript.as_ref() != output_data.user_lockscript.as_ref()
        || input_data.get_raw_lot_size() != output_data.get_raw_lot_size()
        || input_data.x_lock_address.as_ref() != output_data.x_lock_address.as_ref()
    {
        return Err(Error::InvalidDataChange);
    }
    Ok(())
}

/// ensure transfer happen on XChain by verifying the spv proof
fn verify_witness(data: &ToCKBCellDataView) -> Result<(), Error> {
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
    match get_xchain_kind()? {
        XChainKind::Btc => verify_btc_witness(data, proof, cell_dep_index_list),
        XChainKind::Eth => todo!(),
    }
}

fn verify_btc_witness(
    data: &ToCKBCellDataView,
    proof: &[u8],
    cell_dep_index_list: &[u8],
) -> Result<(), Error> {
    debug!(
        "proof: {:?}, cell_dep_index_list: {:?}",
        proof, cell_dep_index_list
    );
    // parse difficulty
    if cell_dep_index_list.len() != 1 {
        return Err(Error::InvalidWitness);
    }
    let dep_data = load_cell_data(cell_dep_index_list[0].into(), Source::CellDep)?;
    debug!("dep data is {:?}", &dep_data);
    if BTCDifficultyReader::verify(&dep_data, false).is_err() {
        return Err(Error::DifficultyDataInvalid);
    }
    let difficulty_reader = BTCDifficultyReader::new_unchecked(&dep_data);
    debug!("difficulty_reader: {:?}", difficulty_reader);
    // parse witness
    if BTCSPVProofReader::verify(proof, false).is_err() {
        return Err(Error::InvalidWitness);
    }
    let proof_reader = BTCSPVProofReader::new_unchecked(proof);
    debug!("proof_reader: {:?}", proof_reader);
    // verify btc spv
    verify_btc_spv(proof_reader, difficulty_reader)?;
    // verify transfer amount, to matches
    let funding_output_index: u8 = proof_reader.funding_output_index().into();
    let vout = Vout::new(proof_reader.vout().raw_data())?;
    let tx_out = vout.index(funding_output_index.into())?;
    let script_pubkey = tx_out.script_pubkey();
    debug!("script_pubkey payload: {:?}", script_pubkey.payload()?);
    match script_pubkey.payload()? {
        PayloadType::WPKH(_) => {
            let addr = bech32::encode("bc", (&script_pubkey[1..]).to_base32()).unwrap();
            debug!(
                "hex format: addr: {}, x_lock_address: {}",
                hex::encode(addr.as_bytes().to_vec()),
                hex::encode(data.x_lock_address.as_ref().to_vec())
            );
            debug!(
                "addr: {}, x_lock_address: {}",
                String::from_utf8(addr.as_bytes().to_vec()).unwrap(),
                String::from_utf8(data.x_lock_address.as_ref().to_vec()).unwrap()
            );
            if addr.as_bytes() != data.x_lock_address.as_ref() {
                return Err(Error::WrongFundingAddr);
            }
        }
        _ => return Err(Error::UnsupportedFundingType),
    }
    let expect_value = data.get_btc_lot_size()?.get_sudt_amount();
    let value = tx_out.value() as u128;
    debug!("actual value: {}, expect: {}", value, expect_value);
    if value < expect_value {
        return Err(Error::FundingNotEnough);
    }
    Ok(())
}

fn verify_btc_spv(proof: BTCSPVProofReader, difficulty: BTCDifficultyReader) -> Result<(), Error> {
    debug!("start verify_btc_spv");
    if !btcspv::validate_vin(proof.vin().raw_data()) {
        return Err(Error::SpvProofInvalid);
    }
    debug!("finish validate_vin");
    if !btcspv::validate_vout(proof.vout().raw_data()) {
        return Err(Error::SpvProofInvalid);
    }
    debug!("finish validate_vout");
    let mut ver = [0u8; 4];
    ver.copy_from_slice(proof.version().raw_data());
    let mut lock = [0u8; 4];
    lock.copy_from_slice(proof.locktime().raw_data());
    debug!("ver: {:?}, lock: {:?}", ver, lock);
    // btcspv::hash256(&[version, vin.as_ref(), vout.as_ref(), locktime])
    let vin = Vin::new(proof.vin().raw_data())?;
    let vout = Vout::new(proof.vout().raw_data())?;
    debug!("{:?}", &[&ver, vin.as_ref(), vout.as_ref(), &lock]);
    let tx_id = validatespv::calculate_txid(&ver, &vin, &vout, &lock);
    debug!("tx_id: {:?}", tx_id);
    if tx_id.as_ref() != proof.tx_id().raw_data() {
        return Err(Error::WrongTxId);
    }

    // verify difficulty
    let raw_headers = proof.headers();
    let headers = HeaderArray::new(raw_headers.raw_data())?;
    let observed_diff = validatespv::validate_header_chain(&headers, false)?;
    let previous_diff = U256::from_little_endian(difficulty.previous().raw_data());
    let current_diff = U256::from_little_endian(difficulty.current().raw_data());
    let first_header_diff = headers.index(0).difficulty();
    debug!(
        "previous: {:?}, current: {:?}, first_header_diff: {:?}",
        previous_diff, current_diff, first_header_diff
    );

    let req_diff = if first_header_diff == current_diff {
        current_diff
    } else if first_header_diff == previous_diff {
        previous_diff
    } else {
        return Err(Error::NotAtCurrentOrPreviousDifficulty);
    };

    if observed_diff < req_diff * TX_PROOF_DIFFICULTY_FACTOR {
        return Err(Error::InsufficientDifficulty);
    }
    debug!("finish diff verify");

    // verify tx
    let header = headers.index(headers.len() - 1);
    let mut idx = [0u8; 8];
    idx.copy_from_slice(proof.index().raw_data());
    debug!("tx_id: {}", hex::encode(tx_id.as_ref()));
    debug!("merkle_root: {}", hex::encode(header.tx_root().as_ref()));
    debug!(
        "proof: {}",
        hex::encode(proof.intermediate_nodes().raw_data())
    );
    debug!("index: {}", u64::from_le_bytes(idx));
    if !validatespv::prove(
        tx_id,
        header.tx_root(),
        &MerkleArray::new(proof.intermediate_nodes().raw_data())?,
        u64::from_le_bytes(idx),
    ) {
        return Err(Error::BadMerkleProof);
    }
    debug!("finish merkle proof verify");

    Ok(())
}

fn verify_xt_issue(data: &ToCKBCellDataView) -> Result<(), Error> {
    match get_xchain_kind()? {
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
        .filter(|script| {
            script.code_hash().raw_data().as_ref() == SUDT_CODE_HASH.as_ref()
                && script.args().raw_data().as_ref() == lock_hash.as_ref()
                && script.hash_type() == 0u8.into()
        })
        .count();
    if input_xt_num != 0 {
        return Err(Error::InvalidXTInInputOrOutput);
    }
    let output_xt_num = QueryIter::new(load_cell_type, Source::Output)
        .filter(|type_opt| type_opt.is_some())
        .map(|type_opt| type_opt.unwrap())
        .filter(|script| {
            script.code_hash().raw_data().as_ref() == SUDT_CODE_HASH.as_ref()
                && script.args().raw_data().as_ref() == lock_hash.as_ref()
        })
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

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    debug!("start mint_xt");
    let input_data = toCKB_data_tuple.0.as_ref().expect("should not happen");
    let output_data = toCKB_data_tuple.1.as_ref().expect("should not happen");
    verify_data(input_data, output_data)?;
    debug!("verify data finish");
    verify_witness(input_data)?;
    debug!("verify witness finish");
    verify_xt_issue(input_data)?;
    debug!("verify xt issue finish");
    Ok(())
}

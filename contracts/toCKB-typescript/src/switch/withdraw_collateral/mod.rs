use crate::switch::ToCKBCellDataTuple;
use crate::utils::{
    config::{
        PLEDGE, SIGNER_FEE_RATE, SUDT_CODE_HASH, TX_PROOF_DIFFICULTY_FACTOR, XT_CELL_CAPACITY,
    },
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
use ckb_std::ckb_types::prelude::*;
use ckb_std::{
    ckb_constants::Source,
    debug,
    high_level::{
        load_cell, load_cell_capacity, load_cell_data, load_cell_lock, load_cell_lock_hash,
        load_cell_type, load_witness_args, QueryIter,
    },
};
use core::result::Result;
use molecule::prelude::*;
use primitive_types::U256;

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
                "hex format: addr: {}, x_unlock_address: {}",
                hex::encode(addr.as_bytes().to_vec()),
                hex::encode(data.x_unlock_address.as_ref().to_vec())
            );
            debug!(
                "addr: {}, x_unlock_address: {}",
                String::from_utf8(addr.as_bytes().to_vec()).unwrap(),
                String::from_utf8(data.x_unlock_address.as_ref().to_vec()).unwrap()
            );
            if addr.as_bytes() != data.x_unlock_address.as_ref() {
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

fn verify_capacity(input_data: &ToCKBCellDataView) -> Result<(), Error> {
    let signer_xt_cell_cap = QueryIter::new(load_cell, Source::Output)
        .filter(|cell| cell.lock().as_bytes().as_ref() == input_data.signer_lockscript.as_ref())
        .map(|cell| cell.capacity().unpack())
        .collect::<Vec<u64>>()
        .into_iter()
        .sum::<u64>();
    let ckb_cell_cap = load_cell_capacity(0, Source::GroupInput)?;
    if signer_xt_cell_cap != ckb_cell_cap {
        return Err(Error::CapacityInvalid);
    }
    Ok(())
}

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    debug!("start withdraw collateral");
    let input_data = toCKB_data_tuple.0.as_ref().expect("should not happen");
    verify_capacity(input_data)?;
    debug!("verify capacity finish");
    verify_witness(input_data)?;
    debug!("verify witness finish");
    Ok(())
}

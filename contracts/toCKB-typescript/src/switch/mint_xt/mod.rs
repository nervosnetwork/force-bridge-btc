use crate::switch::ToCKBCellDataTuple;
use crate::utils::config::{SIGNER_FEE_RATE, TX_PROOF_DIFFICULTY_FACTOR};
use crate::utils::types::{
    btc_difficulty::BTCDifficultyReader,
    mint_xt_witness::{BTCSPVProofReader, MintXTWitnessReader},
    BtcLotSize, Error, ToCKBCellDataView, XChainKind,
};
use bitcoin_spv::{
    btcspv,
    types::{HeaderArray, MerkleArray, PayloadType, Vin, Vout},
    validatespv,
};
use ckb_std::{
    ckb_constants::Source,
    debug,
    error::SysError,
    high_level::{
        load_cell_data, load_cell_lock_hash, load_cell_type_hash, load_witness_args, QueryIter,
    },
};
use core::result::Result;
use molecule::prelude::{Entity, Reader};
use num::bigint::BigUint;

fn verify_data(
    input_data: &ToCKBCellDataView,
    output_data: &ToCKBCellDataView,
) -> Result<(), Error> {
    if input_data.kind != output_data.kind
        || input_data.signer_lockscript_hash.as_ref() != output_data.signer_lockscript_hash.as_ref()
        || input_data.user_lockscript_hash.as_ref() != output_data.user_lockscript_hash.as_ref()
        || input_data.x_lock_address.as_ref() != output_data.x_lock_address.as_ref()
    {
        return Err(Error::InvalidDataChange);
    }
    Ok(())
}

/// ensure transfer happen on XChain by verifying the spv proof
fn verify_witness(data: &ToCKBCellDataView) -> Result<(), Error> {
    let witness_args = load_witness_args(0, Source::GroupInput)?.input_type();
    if witness_args.is_none() {
        return Err(Error::InvalidWitness);
    }
    let witness_args = witness_args.to_opt().unwrap().as_bytes();
    if MintXTWitnessReader::verify(&witness_args, false).is_err() {
        return Err(Error::InvalidWitness);
    }
    let witness = MintXTWitnessReader::new_unchecked(&witness_args);
    let proof = witness.spv_proof().raw_data();
    let cell_dep_index_list = witness.cell_dep_index_list().raw_data();
    match data.kind {
        XChainKind::Btc => verify_btc_witness(data, proof, cell_dep_index_list),
        XChainKind::Eth => todo!(),
    }
}

fn verify_btc_witness(
    data: &ToCKBCellDataView,
    proof: &[u8],
    cell_dep_index_list: &[u8],
) -> Result<(), Error> {
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
    // parse witness
    if BTCSPVProofReader::verify(proof, false).is_err() {
        return Err(Error::InvalidWitness);
    }
    let proof_reader = BTCSPVProofReader::new_unchecked(proof);
    // verify btc spv
    verify_btc_spv(proof_reader, difficulty_reader)?;
    // verify transfer amount, to matches
    let funding_output_index: u8 = proof_reader.funding_output_index().into();
    let vout = Vout::new(proof_reader.vout().raw_data())?;
    let tx_out = vout.index(funding_output_index.into())?;
    if let PayloadType::WPKH(wpkh) = tx_out.script_pubkey().payload()? {
        // TODO: should calc and compare base58 encoding addr here
        if wpkh != data.x_lock_address.as_ref() {
            return Err(Error::WrongFundingAddr);
        }
    } else {
        return Err(Error::UnsupportedFundingType);
    }
    let lot_size = data.get_btc_lot_size()?;
    let value = tx_out.value();
    if btc_lot_size_to_u128(lot_size) != value as u128 {
        return Err(Error::FundingNotEnough);
    }
    Ok(())
}

fn btc_lot_size_to_u128(lot_size: BtcLotSize) -> u128 {
    match lot_size {
        BtcLotSize::Single => 100_000_000,
        BtcLotSize::Half => 50_000_000,
        BtcLotSize::Quarter => 25_000_000,
    }
}

fn verify_btc_spv(proof: BTCSPVProofReader, difficulty: BTCDifficultyReader) -> Result<(), Error> {
    if !btcspv::validate_vin(proof.vin().raw_data()) {
        return Err(Error::SpvProofInvalid);
    }
    if !btcspv::validate_vout(proof.vout().raw_data()) {
        return Err(Error::SpvProofInvalid);
    }
    let mut ver = [0u8; 4];
    ver.copy_from_slice(proof.version().raw_data());
    let mut lock = [0u8; 4];
    lock.copy_from_slice(proof.locktime().raw_data());
    let tx_id = validatespv::calculate_txid(
        &ver,
        &Vin::new(proof.vin().raw_data())?,
        &Vout::new(proof.vout().raw_data())?,
        &lock,
    );
    if tx_id.as_ref() != proof.tx_id().raw_data() {
        return Err(Error::WrongTxId);
    }

    // verify difficulty
    let raw_headers = proof.headers();
    let headers = HeaderArray::new(raw_headers.raw_data())?;
    let observed_diff = validatespv::validate_header_chain(&headers, false)?;
    let previous_diff = BigUint::from_bytes_be(difficulty.previous().raw_data());
    let current_diff = BigUint::from_bytes_be(difficulty.current().raw_data());
    let first_header_diff = headers.index(0).difficulty();

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

    // verify tx
    let header = headers.index(headers.len());
    let mut idx = [0u8; 8];
    idx.copy_from_slice(proof.index().raw_data());
    if !validatespv::prove(
        tx_id,
        header.tx_root(),
        &MerkleArray::new(proof.intermediate_nodes().raw_data())?,
        u64::from_le_bytes(idx),
    ) {
        return Err(Error::BadMerkleProof);
    }

    Ok(())
}

fn verify_xt_issue(data: &ToCKBCellDataView) -> Result<(), Error> {
    match data.kind {
        XChainKind::Btc => verify_btc_xt_issue(data),
        XChainKind::Eth => todo!(),
    }
}

fn verify_btc_xt_issue(data: &ToCKBCellDataView) -> Result<(), Error> {
    // todo: change to btc_xt type hash
    let btc_xt_type_hash = [0u8; 32];
    let input_xt_num = QueryIter::new(load_cell_type_hash, Source::Input)
        .map(|hash_opt| hash_opt.unwrap_or_default())
        .filter(|hash| hash == &btc_xt_type_hash)
        .count();
    if input_xt_num != 0 {
        return Err(Error::InvalidXTInInput);
    }
    let mut output_index = 0;
    let mut user_checked = false;
    let mut signer_checked = false;
    let xt_amount = btc_lot_size_to_u128(data.get_btc_lot_size()?);
    loop {
        let type_hash_res = load_cell_type_hash(output_index, Source::Output);
        match type_hash_res {
            Err(SysError::IndexOutOfBound) => break,
            Err(err) => {
                debug!("iter output error {:?}", err);
                panic!("iter output return an error")
            }
            Ok(type_hash) => {
                if !(type_hash.is_some() && type_hash.unwrap() == btc_xt_type_hash) {
                    continue;
                }
                let lock_hash = load_cell_lock_hash(output_index, Source::Output)?;
                let cell_data = load_cell_data(output_index, Source::Output)?;
                let mut amount_vec = [0u8; 16];
                amount_vec.copy_from_slice(&cell_data);
                let token_amount = u128::from_le_bytes(amount_vec);
                if lock_hash.as_ref() == data.user_lockscript_hash.as_ref() {
                    if user_checked {
                        return Err(Error::InvalidXTMint);
                    }
                    if token_amount != xt_amount - xt_amount * SIGNER_FEE_RATE.0 / SIGNER_FEE_RATE.1
                    {
                        return Err(Error::InvalidXTMint);
                    }
                    user_checked = true;
                } else if lock_hash.as_ref() == data.signer_lockscript_hash.as_ref() {
                    if signer_checked {
                        return Err(Error::InvalidXTMint);
                    }
                    if token_amount != xt_amount * SIGNER_FEE_RATE.0 / SIGNER_FEE_RATE.1 {
                        return Err(Error::InvalidXTMint);
                    }
                    signer_checked = true;
                } else {
                    return Err(Error::InvalidXTMint);
                }
                output_index += 1;
            }
        }
    }
    if !(user_checked && signer_checked) {
        return Err(Error::InvalidXTMint);
    }
    Ok(())
}

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    let input_data = toCKB_data_tuple.0.as_ref().expect("should not happen");
    let output_data = toCKB_data_tuple.1.as_ref().expect("should not happen");
    verify_data(input_data, output_data)?;
    verify_witness(input_data)?;
    verify_xt_issue(input_data)?;
    Ok(())
}

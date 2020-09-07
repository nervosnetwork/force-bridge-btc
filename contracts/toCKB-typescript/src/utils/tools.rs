use crate::utils::{
    config::{SUDT_CODE_HASH, TX_PROOF_DIFFICULTY_FACTOR, UDT_LEN},
    types::{
        btc_difficulty::BTCDifficultyReader, mint_xt_witness::BTCSPVProofReader, Error,
        ToCKBCellDataView,
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
    ckb_types::{bytes::Bytes, packed::Script, prelude::Unpack},
    debug,
    high_level::{load_cell_data, load_cell_type, load_script},
};
use core::result::Result;
use int_enum::IntEnum;
use molecule::prelude::Reader;
use primitive_types::U256;

#[repr(u8)]
#[derive(Clone, Copy, IntEnum)]
pub enum XChainKind {
    Btc = 1,
    Eth = 2,
}

pub fn get_xchain_kind() -> Result<XChainKind, Error> {
    let script_args: Bytes = load_script()?.args().unpack();
    if script_args.len() != 1 {
        return Err(Error::Encoding);
    }
    let mut buf = [0u8; 1];
    buf.copy_from_slice(script_args.as_ref());
    let kind = u8::from_le_bytes(buf);
    XChainKind::from_int(kind).map_err(|_| Error::Encoding)
}

pub fn get_price(kind: XChainKind) -> Result<u128, Error> {
    let price_cell_data = load_cell_data(0, Source::CellDep)?;
    if price_cell_data.len() != 16 {
        return Err(Error::Encoding);
    }
    let mut buf = [0u8; 16];
    buf.copy_from_slice(&price_cell_data);
    let price: u128 = u128::from_le_bytes(buf);

    match kind {
        XChainKind::Btc => todo!(),
        XChainKind::Eth => todo!(),
    }
    Ok(price)
}

pub fn is_XT_typescript(script: &Script, toCKB_lock_hash: &[u8]) -> bool {
    if script.code_hash().raw_data().as_ref() == SUDT_CODE_HASH.as_ref()
        && script.args().raw_data().as_ref() == toCKB_lock_hash
        && script.hash_type() == 0u8.into()
    {
        return true;
    }
    false
}

pub fn get_sum_sudt_amount(
    start_index: usize,
    source: Source,
    toCKB_lock_hash: &[u8],
) -> Result<u128, Error> {
    let mut index = start_index;
    let mut sum_amount = 0;
    loop {
        let res = load_cell_type(index, source);
        if res.is_err() {
            break;
        }
        let script = res.unwrap();
        if script.is_none() || !is_XT_typescript(&script.unwrap(), toCKB_lock_hash) {
            return Err(Error::TxInvalid);
        }

        let cell_data = load_cell_data(index, source)?;
        let mut data = [0u8; UDT_LEN];
        data.copy_from_slice(&cell_data);
        let amount = u128::from_le_bytes(data);
        sum_amount += amount;

        index += 1;
    }

    Ok(sum_amount)
}

pub fn verify_btc_witness(
    data: &ToCKBCellDataView,
    proof: &[u8],
    cell_dep_index_list: &[u8],
    expect_address: &[u8],
    expect_value: u128,
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
            if addr.as_bytes() != expect_address {
                return Err(Error::WrongFundingAddr);
            }
        }
        _ => return Err(Error::UnsupportedFundingType),
    }
    let value = tx_out.value() as u128;
    debug!("actual value: {}, expect: {}", value, expect_value);
    if value < expect_value {
        return Err(Error::FundingNotEnough);
    }
    Ok(())
}

pub fn verify_btc_spv(
    proof: BTCSPVProofReader,
    difficulty: BTCDifficultyReader,
) -> Result<(), Error> {
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

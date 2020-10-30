use crate::utils::{
    config::{
        LOCK_TYPE_FLAG, METRIC_TYPE_FLAG_MASK, REMAIN_FLAGS_BITS, SINCE_TYPE_TIMESTAMP, VALUE_MASK,
    },
    transaction::{get_sum_sudt_amount, XChainKind},
    types::{Error, ToCKBCellDataView},
};
use alloc::string::String;
use alloc::vec::Vec;
use bech32::ToBase32;
use bitcoin_spv::types::{HeaderArray, MerkleArray, PayloadType, Vin, Vout};
use bitcoin_spv::{btcspv, validatespv};
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::{bytes::Bytes, prelude::*};
use ckb_std::debug;
use ckb_std::high_level::{
    load_cell, load_cell_capacity, load_cell_data, load_input_since, QueryIter,
};
use primitive_types::U256;
use tockb_types::config::{BTC_ADDRESS_PREFIX, TX_PROOF_DIFFICULTY_FACTOR};
use tockb_types::generated::btc_difficulty::BTCDifficultyReader;
use tockb_types::generated::mint_xt_witness::BTCSPVProofReader;
use tockb_types::{BtcExtraView, XExtraView};

pub fn verify_since() -> Result<u64, Error> {
    let since = load_input_since(0, Source::GroupInput).map_err(|_| Error::InputSinceInvalid)?;

    if since & REMAIN_FLAGS_BITS != 0 // check flags is valid
        || since & LOCK_TYPE_FLAG == 0 // check if it is relative_flag
        || since & METRIC_TYPE_FLAG_MASK != SINCE_TYPE_TIMESTAMP
    // check if it is timestamp value
    {
        return Err(Error::InputSinceInvalid);
    }

    let auction_time = since & VALUE_MASK;
    Ok(auction_time)
}

pub fn verify_since_by_value(value: u64) -> Result<(), Error> {
    let since = load_input_since(0, Source::GroupInput)?;
    if since != value {
        return Err(Error::InputSinceInvalid);
    }
    Ok(())
}

pub fn verify_auction_inputs(
    toCKB_lock_hash: &[u8],
    lot_amount: u128,
    signer_fee: u128,
) -> Result<u128, Error> {
    // inputs[0]: toCKB cell
    // inputs[1:]: XT cell the bidder provides
    // check XT cell on inputs
    let inputs_amount = get_sum_sudt_amount(1, Source::Input, toCKB_lock_hash)?;

    if inputs_amount < lot_amount + signer_fee {
        return Err(Error::FundingNotEnough);
    }
    Ok(inputs_amount)
}

pub fn verify_capacity() -> Result<(), Error> {
    let cap_input = load_cell_capacity(0, Source::GroupInput).expect("get input capacity");
    let cap_output = load_cell_capacity(0, Source::GroupOutput).expect("get output capacity");
    if cap_input != cap_output {
        return Err(Error::CapacityInvalid);
    }
    Ok(())
}

pub fn verify_capacity_with_value(input_data: &ToCKBCellDataView, value: u64) -> Result<(), Error> {
    let sum = QueryIter::new(load_cell, Source::Output)
        .filter(|cell| cell.lock().as_bytes() == input_data.signer_lockscript)
        .map(|cell| cell.capacity().unpack())
        .collect::<Vec<u64>>()
        .into_iter()
        .sum::<u64>();
    if sum < value {
        return Err(Error::CapacityInvalid);
    }
    Ok(())
}

pub fn verify_data(
    input_toCKB_data: &ToCKBCellDataView,
    out_toCKB_data: &ToCKBCellDataView,
) -> Result<u128, Error> {
    let lot_size = match input_toCKB_data.get_xchain_kind() {
        XChainKind::Btc => {
            if out_toCKB_data.get_btc_lot_size()? != input_toCKB_data.get_btc_lot_size()? {
                return Err(Error::InvariantDataMutated);
            }
            verify_btc_address(out_toCKB_data.x_unlock_address.as_ref())?;
            out_toCKB_data.get_btc_lot_size()?.get_sudt_amount()
        }
        XChainKind::Eth => {
            if out_toCKB_data.get_eth_lot_size()? != input_toCKB_data.get_eth_lot_size()? {
                return Err(Error::InvariantDataMutated);
            }
            if out_toCKB_data.x_unlock_address.as_ref().len() != 20 {
                return Err(Error::XChainAddressInvalid);
            }
            out_toCKB_data.get_eth_lot_size()?.get_sudt_amount()
        }
    };
    if input_toCKB_data.user_lockscript != out_toCKB_data.user_lockscript
        || input_toCKB_data.x_lock_address != out_toCKB_data.x_lock_address
        || input_toCKB_data.signer_lockscript != out_toCKB_data.signer_lockscript
        || input_toCKB_data.x_extra != out_toCKB_data.x_extra
    {
        return Err(Error::InvariantDataMutated);
    }
    Ok(lot_size)
}

pub fn verify_btc_witness(
    _data: &ToCKBCellDataView,
    proof: &[u8],
    cell_dep_index_list: &[u8],
    expect_address: &[u8],
    expect_value: u128,
    is_return_vin: bool,
) -> Result<BtcExtraView, Error> {
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
    let tx_hash = verify_btc_spv(proof_reader, difficulty_reader)?;

    // verify transfer amount, to matches
    let funding_output_index = proof_reader.funding_output_index().into();

    let vout = Vout::new(proof_reader.vout().raw_data())?;
    let tx_out = vout.index(funding_output_index as usize)?;
    let script_pubkey = tx_out.script_pubkey();
    debug!("script_pubkey payload: {:?}", script_pubkey.payload()?);
    match script_pubkey.payload()? {
        PayloadType::WPKH(pkh) => {
            let mut addr_u5 = Vec::with_capacity(33);
            addr_u5.push(bech32::u5::try_from_u8(0).unwrap());
            addr_u5.extend(pkh.to_base32());
            debug!("addr_u5: {:?}", &addr_u5);
            let addr = bech32::encode(BTC_ADDRESS_PREFIX, addr_u5)
                .expect("bech32 encode should not return error");
            debug!(
                "hex format: addr: {}, expect_address: {}",
                hex::encode(addr.as_bytes().to_vec()),
                hex::encode(expect_address.as_ref().to_vec())
            );
            debug!(
                "addr: {}, expect_address: {}",
                String::from_utf8(addr.as_bytes().to_vec()).unwrap(),
                String::from_utf8(expect_address.as_ref().to_vec()).unwrap()
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
    if is_return_vin {
        let funding_input_index: u32 = proof_reader.funding_input_index().into();
        let vin = Vin::new(proof_reader.vin().raw_data())?;
        let tx_in = vin.index(funding_input_index as usize)?;
        debug!(
            "vin tx_id {}",
            hex::encode(tx_in.outpoint().txid_le().as_ref().as_ref())
        );
        debug!("vin output index {}", tx_in.outpoint().vout_index());
        Ok(BtcExtraView {
            lock_tx_hash: tx_in.outpoint().txid_le().as_ref().as_ref().into(),
            lock_vout_index: tx_in.outpoint().vout_index(),
        })
    } else {
        Ok(BtcExtraView {
            lock_tx_hash: tx_hash,
            lock_vout_index: funding_output_index,
        })
    }
}

pub fn verify_btc_faulty_witness(
    data: &ToCKBCellDataView,
    proof: &[u8],
    cell_dep_index_list: &[u8],
    is_when_redeeming: bool,
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

    // get tx in
    let funding_input_index: u32 = proof_reader.funding_input_index().into();

    let vin = Vin::new(proof_reader.vin().raw_data())?;
    let tx_in = vin.index(funding_input_index as usize)?;

    // get mint_xt's funding_output info from cell_data
    let btc_extra = match &data.x_extra {
        XExtraView::Btc(extra) => Ok(extra),
        _ => Err(Error::FaultyBtcWitnessInvalid),
    }?;

    // check if the locked btc is transferred by signer
    let btc_extra_txid: Vec<u8> = btc_extra.lock_tx_hash.clone().into();
    debug!(
        "btc_extra_txid: {},  tx_in.outpoint().txid_le(): {}",
        hex::encode(btc_extra_txid.as_slice()),
        hex::encode(tx_in.outpoint().txid_le().as_ref().as_ref())
    );

    debug!(
        "btc_extra.lock_vout_index: {},   tx_in.outpoint().vout_index(): {}",
        btc_extra.lock_vout_index,
        tx_in.outpoint().vout_index()
    );

    if tx_in.outpoint().txid_le().as_ref().as_ref() != btc_extra_txid.as_slice()
        || tx_in.outpoint().vout_index() != btc_extra.lock_vout_index
    {
        return Err(Error::FaultyBtcWitnessInvalid);
    }

    // if is_when_redeeming, check if signer transferred insufficient btc_amount to user_unlock_addr
    if is_when_redeeming {
        debug!("verify_btc_faulty_witness is_when_redeeming");
        // verify transfer amount, to matches
        let vout = Vout::new(proof_reader.vout().raw_data())?;
        let mut index: usize = 0;
        let mut sum_amount: u128 = 0;
        let expect_address = data.x_unlock_address.as_ref();
        let lot_amount = data.get_btc_lot_size()?.get_sudt_amount();

        // calc sum_amount which signer transferred to user
        debug!("begin calc sum_amount which signer transferred to user");
        loop {
            let tx_out = match vout.index(index.into()) {
                Ok(out) => out,
                Err(_) => {
                    break;
                }
            };
            index += 1;

            let script_pubkey = tx_out.script_pubkey();
            match script_pubkey.payload()? {
                PayloadType::WPKH(pkh) => {
                    let mut addr_u5 = Vec::with_capacity(33);
                    addr_u5.push(bech32::u5::try_from_u8(0).unwrap());
                    addr_u5.extend(pkh.to_base32());
                    debug!("addr_u5: {:?}", &addr_u5);
                    let addr = bech32::encode(BTC_ADDRESS_PREFIX, addr_u5)
                        .expect("bech32 encode should not return error");
                    debug!(
                        "hex format: addr: {}, x_lock_address: {}",
                        hex::encode(addr.as_bytes().to_vec()),
                        hex::encode(data.x_lock_address.as_ref().to_vec())
                    );
                    debug!(
                        "addr: {}, x_unlock_address: {}",
                        String::from_utf8(addr.as_bytes().to_vec()).unwrap(),
                        String::from_utf8(expect_address.to_vec()).unwrap()
                    );
                    if addr.as_bytes() != expect_address {
                        continue;
                    }
                }
                _ => continue,
            }

            sum_amount += tx_out.value() as u128;
        }

        debug!(
            "calc sum_amount: {}, lot_amount: {}",
            sum_amount, lot_amount
        );
        if sum_amount >= lot_amount {
            // it means signer transferred enough amount to user, mismatch FaultyWhenRedeeming condition
            return Err(Error::FaultyBtcWitnessInvalid);
        }
    }
    Ok(())
}

pub fn verify_btc_spv(
    proof: BTCSPVProofReader,
    difficulty: BTCDifficultyReader,
) -> Result<Bytes, Error> {
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

    Ok(Bytes::from(&tx_id.as_ref()[..]))
}

pub fn verify_btc_address(addr: &[u8]) -> Result<(), Error> {
    let (hrp, data) =
        bech32::decode(core::str::from_utf8(addr).map_err(|_| Error::XChainAddressInvalid)?)
            .map_err(|_| Error::XChainAddressInvalid)?;
    if hrp != BTC_ADDRESS_PREFIX {
        return Err(Error::XChainAddressInvalid);
    }
    if data.len() != 33 {
        return Err(Error::XChainAddressInvalid);
    }
    if data[0].to_u8() != 0 {
        return Err(Error::XChainAddressInvalid);
    }
    Ok(())
}

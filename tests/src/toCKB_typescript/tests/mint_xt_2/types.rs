use crate::toCKB_typescript::utils::types::generated::mint_xt_witness;
use crate::toCKB_typescript::utils::types::generated::{Bytes, Bytes2};
use anyhow::Result;
use ckb_tool::ckb_types::{packed::*, prelude::*};
use molecule::prelude::*;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

pub struct ToCKBCellDataTest {
    pub lot_size: u8,
    pub x_lock_address: String,
    pub user_lockscript: Script,
    pub signer_lockscript: Script,
}

pub struct Output {
    pub typescript: Script,
    pub lockscript: Script,
    pub amount: u128,
    pub capacity: u64,
}

pub enum SpvProof {
    BTC(mint_xt_witness::BTCSPVProof),
    ETH(mint_xt_witness::ETHSPVProof),
}

pub struct BtcDifficultyTest {
    pub previous: u64,
    pub current: u64,
}

pub struct Witness {
    pub cell_dep_index_list: Vec<u8>,
    pub spv_proof: SpvProof,
}

pub enum CellDepsData {
    BTC(BtcDifficultyTest),
}

pub struct TestCase {
    pub kind: u8,
    pub input_capacity: u64,
    pub output_capacity: u64,
    pub tockb_cell_data: ToCKBCellDataTest,
    pub outputs: Vec<Output>,
    pub witness: Witness,
    pub cell_deps_data: CellDepsData,
    pub expect_return_code: i8,
}

#[derive(Serialize, Deserialize, Default)]
pub struct BTCSPVProofJson {
    pub version: String,
    pub vin: String,
    pub vout: String,
    pub locktime: String,
    pub tx_id: String,
    pub index: u64,
    pub headers: String,
    pub intermediate_nodes: String,
    pub funding_output_index: u8,
}

#[derive(Serialize, Deserialize, Default)]
pub struct ETHSPVProofJson {
    pub log_index: u64,
    pub log_entry_data: String,
    pub receipt_index: u64,
    pub receipt_data: String,
    pub receipts_root: String,
    pub header_data: String,
    pub proof: Vec<String>,
}

impl TryFrom<ETHSPVProofJson> for mint_xt_witness::ETHSPVProof {
    type Error = anyhow::Error;
    fn try_from(proof: ETHSPVProofJson) -> Result<Self> {
        let mut proofVec: Vec<Bytes> = vec![];
        for i in 0..proof.proof.len() {
            proofVec.push(hex::decode(clear_0x(&proof.proof[i]))?.into())
        }
        Ok(mint_xt_witness::ETHSPVProof::new_builder()
            .log_index(proof.log_index.into())
            .log_entry_data(hex::decode(clear_0x(&proof.log_entry_data))?.into())
            .receipt_index(proof.receipt_index.into())
            .receipt_data(hex::decode(clear_0x(&proof.receipt_data))?.into())
            .receipts_root(hex::decode(clear_0x(&proof.receipts_root))?.into())
            .header_data(hex::decode(clear_0x(&proof.header_data))?.into())
            .proof(Bytes2::new_builder().set(proofVec).build())
            .build())
    }
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

pub fn clear_0x(s: &str) -> &str {
    if &s[..2] == "0x" || &s[..2] == "0X" {
        &s[2..]
    } else {
        s
    }
}

pub fn json_to_btc_proof(proof: &str) -> Result<mint_xt_witness::BTCSPVProof> {
    let proof: BTCSPVProofJson = serde_json::from_str(proof)?;
    proof.try_into()
}

use crate::toCKB_typescript::utils::types::generated::mint_xt_witness;
use anyhow::Result;
use ckb_tool::ckb_types::{bytes::Bytes, packed::*, prelude::*};
use molecule::prelude::*;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

#[derive(Clone)]
pub struct ToCKBCellDataTest {
    pub lot_size: u8,
    pub x_lock_address: String,
    pub x_unlock_address: String,
    pub user_lockscript: Script,
    pub signer_lockscript: Script,
    pub x_extra: XExtraView,
}

#[derive(Clone)]
pub enum XExtraView {
    Btc(BtcExtraView),
    Eth(EthExtraView),
}

#[derive(Clone)]
pub struct BtcExtraView {
    pub lock_tx_hash: Bytes,
    pub lock_vout_index: u32,
}

#[derive(Clone)]
pub struct EthExtraView {
    pub dummy: Bytes,
}

pub struct Output {
    pub typescript: Script,
    pub lockscript: Script,
    pub amount: u128,
    pub capacity: u64,
}

pub enum SpvProof {
    BTC(mint_xt_witness::BTCSPVProof),
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
    pub input_status: u8,
    pub output_status: u8,
    pub input_capacity: u64,
    pub output_capacity: u64,
    pub input_tockb_cell_data: ToCKBCellDataTest,
    pub output_tockb_cell_data: ToCKBCellDataTest,
    pub outputs: Vec<Output>,
    pub witness: Witness,
    pub cell_deps_data: CellDepsData,
    pub expect_return_code: i8,
}

#[derive(Serialize, Deserialize, Default)]
pub struct BTCSPVProofJson {
    pub version: u32,
    pub vin: String,
    pub vout: String,
    pub locktime: u32,
    pub tx_id: String,
    pub index: u64,
    pub headers: String,
    pub intermediate_nodes: String,
    pub funding_output_index: u32,
    pub funding_input_index: u32,
}

impl TryFrom<BTCSPVProofJson> for mint_xt_witness::BTCSPVProof {
    type Error = anyhow::Error;

    fn try_from(proof: BTCSPVProofJson) -> Result<Self> {
        Ok(mint_xt_witness::BTCSPVProof::new_builder()
            .version(proof.version.into())
            .vin(hex::decode(clear_0x(&proof.vin))?.into())
            .vout(hex::decode(clear_0x(&proof.vout))?.into())
            .locktime(proof.locktime.into())
            .tx_id(hex::decode(clear_0x(&proof.tx_id))?.into())
            .index(proof.index.into())
            .headers(hex::decode(clear_0x(&proof.headers))?.into())
            .intermediate_nodes(hex::decode(clear_0x(&proof.intermediate_nodes))?.into())
            .funding_output_index(proof.funding_output_index.into())
            .funding_input_index(proof.funding_input_index.into())
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

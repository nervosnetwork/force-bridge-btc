use anyhow::Result;
use bitcoin::{consensus::encode::serialize_hex, Block};
use bitcoin_spv::types::Hash256Digest;
use bitcoin_spv::{btcspv::hash256_merkle_step, types::MerkleArray, validatespv};
use molecule::prelude::{Builder, Entity};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

use tockb_types::generated::mint_xt_witness::BTCSPVProof;

#[derive(Serialize, Deserialize, Default)]
pub struct MintXTProof {
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

impl TryFrom<MintXTProof> for BTCSPVProof {
    type Error = anyhow::Error;

    fn try_from(proof: MintXTProof) -> Result<Self> {
        Ok(BTCSPVProof::new_builder()
            .version(proof.version.into())
            .vin(hex::decode(clear_0x(&proof.vin))?.into())
            .vout(hex::decode(clear_0x(&proof.vout))?.into())
            .locktime(proof.locktime.into())
            .tx_id(hex::decode(clear_0x(&proof.tx_id))?.try_into()?)
            .index(proof.index.into())
            .headers(hex::decode(clear_0x(&proof.headers))?.into())
            .intermediate_nodes(hex::decode(clear_0x(&proof.intermediate_nodes))?.into())
            .funding_output_index(proof.funding_output_index.into())
            .funding_input_index(proof.funding_input_index.into())
            .build())
    }
}

pub fn generate_mint_xt_proof(
    block: &Block,
    tx_hash: &str,
    funding_output_index: u32,
    funding_input_index: u32,
) -> Result<MintXTProof> {
    let tx_id = hex::decode(clear_0x(tx_hash))?
        .into_iter()
        .rev()
        .collect::<Vec<_>>();
    let tx_index = block
        .txdata
        .iter()
        .enumerate()
        .filter(|&t| t.1.txid().as_ref() == &tx_id[..])
        .collect::<Vec<_>>();
    assert_eq!(tx_index.len(), 1);
    let tx_index = tx_index[0].0;
    println!("txindex {}", tx_index);
    let tx = block.txdata[tx_index].clone();
    let proof = get_merkle_proof(block, tx_index).unwrap();
    let flat_proof = proof
        .into_iter()
        .flat_map(|p| p.into_iter())
        .collect::<Vec<u8>>();

    Ok(MintXTProof {
        version: tx.version,
        vin: serialize_hex(&tx.input),
        vout: serialize_hex(&tx.output),
        locktime: tx.lock_time,
        tx_id: hex::encode(tx.txid().as_ref()),
        index: tx_index as u64,
        headers: serialize_hex(&block.header),
        intermediate_nodes: hex::encode(flat_proof),
        funding_output_index,
        funding_input_index,
    })
}

pub fn spv_prove(block: &Block, proof: &MintXTProof) -> Result<bool> {
    let prove_res = validatespv::prove(
        bytes_to_hash256digest(&hex::decode(&proof.tx_id)?),
        bytes_to_hash256digest(block.merkle_root().as_ref()),
        &MerkleArray::new(&hex::decode(&proof.intermediate_nodes)?).unwrap(),
        proof.index,
    );
    Ok(prove_res)
}

fn get_merkle_proof(block: &Block, index: usize) -> Result<Vec<Vec<u8>>> {
    let tx_len = block.txdata.len();
    assert!(tx_len >= 1);
    if tx_len == 1 {
        return Ok(vec![block.txdata[0].txid().to_vec()]);
    }
    let mut layers = Vec::new();
    let mut proof = Vec::new();
    let layer: Vec<_> = block
        .txdata
        .iter()
        .map(|tx| tx.txid().as_ref().to_vec())
        .collect();
    layers.push(layer);
    let mut current_layer_index = 0;
    let mut proof_index_in_layer = index;
    loop {
        let mut current_layer = layers[current_layer_index].iter();
        let current_layer_len = current_layer.len();
        if current_layer_len <= 1 {
            break;
        }
        let new_layer_len = current_layer_len / 2 + current_layer_len % 2;
        let mut new_layer = Vec::with_capacity(new_layer_len);
        while let Some(hash1) = current_layer.next() {
            let hash2 = current_layer.next().unwrap_or(hash1);
            new_layer.push(hash256_merkle_step(hash1, hash2).as_ref().to_vec());
        }
        layers.push(new_layer);
        let current_layer = layers[current_layer_index].clone();
        if proof_index_in_layer % 2 == 0 {
            proof.push(current_layer[proof_index_in_layer + 1].clone());
        } else {
            proof.push(current_layer[proof_index_in_layer - 1].clone());
        }
        proof_index_in_layer >>= 1;
        current_layer_index += 1;
    }
    Ok(proof)
}

fn clear_0x(s: &str) -> &str {
    if &s[..2] == "0x" || &s[..2] == "0X" {
        &s[2..]
    } else {
        s
    }
}

fn bytes_to_hash256digest(b: &[u8]) -> Hash256Digest {
    let mut tmp = [0u8; 32];
    tmp.copy_from_slice(b);
    tmp.into()
}

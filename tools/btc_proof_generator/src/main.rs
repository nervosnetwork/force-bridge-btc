use anyhow::Result;
use bitcoin::{
    consensus::{deserialize, encode::serialize_hex},
    Block,
};
use bitcoin_spv::{
    btcspv::hash256_merkle_step,
    types::{Hash256Digest, MerkleArray},
    validatespv,
};
use clap::{load_yaml, value_t, App};
use hex::FromHex;
use serde::{Deserialize, Serialize};

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

fn fetch_block(block_hash: &str) -> Result<Block> {
    let url = format!("https://blockchain.info/block/{}?format=hex", block_hash);
    let resp = reqwest::blocking::get(&url)?.text()?;
    let raw_block = Vec::from_hex(resp)?;
    let block: Block = deserialize(&raw_block)?;
    Ok(block)
}

#[derive(Serialize, Deserialize, Default)]
pub struct MintXTProof {
    pub version: String,
    pub vin: String,
    pub vout: String,
    pub locktime: String,
    pub tx_id: String,
    pub index: u64,
    pub headers: String,
    pub intermediate_nodes: String,
    pub funding_output_index: u32,
    pub funding_input_index: u32,
}

fn generate_mint_xt_proof(
    block_hash: &str,
    tx_index: usize,
    funding_output_index: u32,
    funding_input_index: u32,
) -> Result<(MintXTProof, Block)> {
    let block = fetch_block(block_hash)?;
    let tx = block.txdata[tx_index].clone();
    let proof = get_merkle_proof(&block, tx_index)?;
    let flat_proof = proof
        .into_iter()
        .flat_map(|p| p.into_iter())
        .collect::<Vec<u8>>();

    Ok((
        MintXTProof {
            version: serialize_hex(&tx.version),
            vin: serialize_hex(&tx.input),
            vout: serialize_hex(&tx.output),
            locktime: serialize_hex(&tx.lock_time),
            tx_id: hex::encode(tx.txid().as_ref()),
            index: tx_index as u64,
            headers: serialize_hex(&block.header),
            intermediate_nodes: hex::encode(flat_proof),
            funding_output_index,
            funding_input_index,
        },
        block,
    ))
}

fn spv_prove(block: &Block, proof: &MintXTProof) -> Result<bool> {
    let prove_res = validatespv::prove(
        bytes_to_hash256digest(&hex::decode(&proof.tx_id)?),
        bytes_to_hash256digest(block.merkle_root().as_ref()),
        &MerkleArray::new(&hex::decode(&proof.intermediate_nodes)?).unwrap(),
        proof.index,
    );
    Ok(prove_res)
}

fn bytes_to_hash256digest(b: &[u8]) -> Hash256Digest {
    let mut tmp = [0u8; 32];
    tmp.copy_from_slice(b);
    tmp.into()
}

fn main() -> Result<()> {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();
    match matches.subcommand() {
        ("mint_xt", mint_xt_matches) => {
            let mint_xt_matches = mint_xt_matches.unwrap();
            let block_hash = mint_xt_matches.value_of("block_hash").unwrap();
            let tx_index = value_t!(mint_xt_matches, "tx_index", usize).unwrap();
            let funding_output_index =
                value_t!(mint_xt_matches, "funding_output_index", u32).unwrap();
            let funding_input_index =
                value_t!(mint_xt_matches, "funding_input_index", u32).unwrap();

            let (mint_xt_proof, block) = generate_mint_xt_proof(
                block_hash,
                tx_index,
                funding_output_index,
                funding_input_index,
            )?;
            assert!(spv_prove(&block, &mint_xt_proof)?);
            println!(
                "btc mint xt proof:\n\n{}",
                serde_json::to_string_pretty(&mint_xt_proof)?
            );
        }
        _ => {}
    }
    Ok(())
}

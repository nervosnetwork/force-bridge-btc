use anyhow::Result;
use std::convert::{TryFrom, TryInto};
use bitcoin::{
    consensus::{deserialize, encode::serialize_hex},
    Block,
};
use molecule::prelude::{Entity, Builder};
use bitcoin_spv::{
    btcspv::hash256_merkle_step,
    types::{Hash256Digest, MerkleArray},
    validatespv,
};
use clap::Clap;
use hex::FromHex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tockb_types::generated::mint_xt_witness::BTCSPVProof;

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
    // let raw_block = include_bytes!("../data/block_648783.raw");
    // let block: Block = deserialize(&hex::decode(raw_block)?)?;
    Ok(block)
}

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

fn generate_mint_xt_proof(
    block_hash: &str,
    tx_hash: &str,
    funding_output_index: u32,
    funding_input_index: u32,
) -> Result<(MintXTProof, Block)> {
    let block = fetch_block(block_hash)?;
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
    // dbg!(&tx_index);
    let tx_index = tx_index[0].0;
    let tx = block.txdata[tx_index].clone();
    let proof = get_merkle_proof(&block, tx_index)?;
    let flat_proof = proof
        .into_iter()
        .flat_map(|p| p.into_iter())
        .collect::<Vec<u8>>();

    Ok((
        MintXTProof {
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

pub fn clear_0x(s: &str) -> &str {
    if &s[..2] == "0x" || &s[..2] == "0X" {
        &s[2..]
    } else {
        s
    }
}

fn hex_string_le_be_transform(hex_str: &str) -> Result<String> {
    let mut bytes = hex::decode(clear_0x(hex_str).as_bytes())?;
    bytes.reverse();
    Ok(hex::encode(bytes.as_slice()))
}

pub fn fetch_transaction(tx_hash: &str) -> Result<Value> {
    let url = format!("https://api.blockcypher.com/v1/btc/main/txs/{}", tx_hash);
    let resp = reqwest::blocking::get(&url)?.json()?;
    Ok(resp)
}

/// generate btc proof for toCKB
#[derive(Clap)]
#[clap(version = "0.1", author = "Wenchao Hu <me@huwenchao.com>")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    MintXt(MintXt),
}

/// generate proof for mint_xt
#[derive(Clap)]
struct MintXt {
    #[clap(short, long)]
    tx_hash: String,
    #[clap(short = "i", long)]
    funding_input_index: u32,
    #[clap(short = "o", long)]
    funding_output_index: u32,
}

fn process_mint_xt(args: MintXt) -> Result<()> {
    let tx = fetch_transaction(&args.tx_hash)?;
    // println!("{}", serde_json::to_string_pretty(&tx)?);
    let block_hash = tx["block_hash"].as_str().expect("can not find block_hash");
    // dbg!(&block_hash);
    let (mint_xt_proof, block) = generate_mint_xt_proof(
        block_hash,
        &args.tx_hash,
        args.funding_output_index,
        args.funding_input_index,
    )?;
    assert!(spv_prove(&block, &mint_xt_proof)?);
    println!(
        "btc mint xt proof:\n\n{}",
        serde_json::to_string_pretty(&mint_xt_proof)?
    );
    let btc_spv_proof: BTCSPVProof = mint_xt_proof.try_into()?;
    println!("\n\nproof in molecule bytes:\n\n{}", hex::encode(btc_spv_proof.as_slice()));
    Ok(())
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    match opts.subcmd {
        SubCommand::MintXt(mint_xt) => process_mint_xt(mint_xt),
    }
}

#[test]
fn test_reverse() {
    let tx_id_be = hex_string_le_be_transform(
        "0x2b21846ae6f15cc29e41b2846c78d756abfedb0d6fea7222263cac0024713bc3",
    )
    .unwrap();
    assert_eq!(
        "c33b712400ac3c262272ea6f0ddbfeab56d7786c84b2419ec25cf1e66a84212b".to_owned(),
        tx_id_be
    );
}

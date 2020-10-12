pub mod utils;
use anyhow::Result;
use bitcoin::hashes::hex::FromHex;
use bitcoin::Txid;
use clap::Clap;
use serde::Deserialize;
use std::convert::TryInto;
use std::fs;

use bitcoincore_rpc::{Auth, Client, RpcApi};

use molecule::prelude::Entity;
use tockb_types::generated::mint_xt_witness::BTCSPVProof;
use utils::{generate_mint_xt_proof, spv_prove};

/// generate btc proof for toCKB
#[derive(Clap)]
#[clap(version = "0.1", author = "jacobdenver007 <jacobdenver@163.com>")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    MintXt(MintXt),
}

#[derive(Clap)]
struct MintXt {
    #[clap(short, long)]
    tx_hash: String,
    #[clap(short = 'i', long)]
    funding_input_index: u32,
    #[clap(short = 'o', long)]
    funding_output_index: u32,
}

#[derive(Deserialize)]
struct BTCClient {
    node: String,
    user: String,
    password: String,
}

fn process_mint_xt(args: MintXt) -> Result<()> {
    let cli_toml = fs::read_to_string("tools/btc-proof-generator-by-rpc/src/cli.toml").unwrap();
    let cli: BTCClient = toml::from_str(&cli_toml).unwrap();
    let rpc = Client::new(cli.node, Auth::UserPass(cli.user, cli.password)).unwrap();

    let tx_id = Txid::from_hex(args.tx_hash.as_str()).expect("parse to Txid");
    let tx = rpc
        .get_raw_transaction_info(&tx_id, None)
        .expect("rpc get_raw_transaction");

    let block_hash = tx.blockhash.expect("get block_hash from tx");
    let block = rpc.get_block(&block_hash).expect("rpc get_block");

    let mint_xt_proof = generate_mint_xt_proof(
        &block,
        args.tx_hash.as_str(),
        args.funding_output_index,
        args.funding_input_index,
    )
    .expect("generate_mint_xt_proof");

    assert!(spv_prove(&block, &mint_xt_proof)?);

    println!(
        "btc mint xt proof:\n\n{}",
        serde_json::to_string_pretty(&mint_xt_proof)?
    );

    let btc_spv_proof: BTCSPVProof = mint_xt_proof.try_into()?;
    println!(
        "\n\nproof in molecule bytes:\n\n{}",
        hex::encode(btc_spv_proof.as_slice())
    );
    Ok(())
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    match opts.subcmd {
        SubCommand::MintXt(mint_xt) => process_mint_xt(mint_xt),
    }
}

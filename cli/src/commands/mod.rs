pub mod contract;
pub mod types;

use anyhow::{anyhow, Result};
use ckb_hash::blake2b_256;
use ckb_sdk::SECP256K1;
use ckb_sdk::{AddressPayload, HttpRpcClient};
use ckb_types::packed::Script;
use contract::contract_handler;
use molecule::prelude::{Builder, Entity};
use std::str::FromStr;
use tockb_sdk::indexer::IndexerRpcClient;
use tockb_sdk::settings::{BtcDifficulty, OutpointConf, PriceOracle, ScriptConf, Settings};
use tockb_sdk::tx_helper::deploy;
use tockb_sdk::util::{parse_privkey_path, send_tx_sync};
use tockb_types::generated::btc_difficulty::BTCDifficulty;
use types::{ContractSubCommand, DevInitArgs, InitArgs, Opts, SubCommand};

pub fn handler(opt: Opts) -> Result<()> {
    match opt.subcmd {
        SubCommand::Init(args) => init_handler(args),
        SubCommand::DevInit(args) => dev_init_handler(args),
        SubCommand::Contract(args) => contract_handler(args),
        _ => todo!(),
    }
}

pub fn init_handler(args: InitArgs) -> Result<()> {
    let InitArgs { config_path, force } = args;
    if std::path::Path::new(&config_path).exists() && !force {
        return Err(anyhow!(
            "tockb config already exists at {}, use `-f` in command if you want to overwrite it",
            &config_path
        ));
    }
    Settings::default()
        .write(&config_path)
        .map_err(|e| anyhow!(e))?;
    println!("tockb config written to {}", &config_path);
    Ok(())
}

pub fn dev_init_handler(args: DevInitArgs) -> Result<()> {
    let DevInitArgs {
        config_path,
        force,
        rpc_url,
        indexer_url,
        private_key_path,
        typescript_path,
        lockscript_path,
        price,
        btc_difficulty,
        sudt_path,
    } = args;
    if std::path::Path::new(&config_path).exists() && !force {
        return Err(anyhow!(
            "tockb config already exists at {}, use `-f` in command if you want to overwrite it",
            &config_path
        ));
    }

    let mut rpc_client = HttpRpcClient::new(rpc_url.clone());
    let mut indexer_client = IndexerRpcClient::new(indexer_url.clone());

    let private_key = parse_privkey_path(&private_key_path)?;

    // dev deploy
    let typescript_bin = std::fs::read(typescript_path)?;
    let lockscript_bin = std::fs::read(lockscript_path)?;
    let sudt_bin = std::fs::read(sudt_path)?;
    let typescript_code_hash = blake2b_256(&typescript_bin);
    let typescript_code_hash_hex = hex::encode(&typescript_code_hash);
    let lockscript_code_hash = blake2b_256(&lockscript_bin);
    let lockscript_code_hash_hex = hex::encode(&lockscript_code_hash);
    let sudt_code_hash = blake2b_256(&sudt_bin);
    let sudt_code_hash_hex = hex::encode(&sudt_code_hash);
    let btc_difficulty_bytes = BTCDifficulty::new_builder()
        .previous(btc_difficulty.to_le_bytes().to_vec().into())
        .current(btc_difficulty.to_le_bytes().to_vec().into())
        .build()
        .as_bytes()
        .to_vec();
    let data = vec![
        typescript_bin,
        lockscript_bin,
        sudt_bin,
        price.to_le_bytes().to_vec(),
        btc_difficulty_bytes,
    ];

    let tx = deploy(&mut rpc_client, &mut indexer_client, &private_key, data).unwrap();
    let tx_hash = send_tx_sync(&mut rpc_client, &tx, 60).unwrap();
    let tx_hash_hex = hex::encode(tx_hash.as_bytes());
    let settings = Settings {
        typescript: ScriptConf {
            code_hash: typescript_code_hash_hex,
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex.clone(),
                index: 0,
            },
        },
        lockscript: ScriptConf {
            code_hash: lockscript_code_hash_hex,
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex.clone(),
                index: 1,
            },
        },
        sudt: ScriptConf {
            code_hash: sudt_code_hash_hex,
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex.clone(),
                index: 2,
            },
        },
        price_oracle: PriceOracle {
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex.clone(),
                index: 3,
            },
        },
        btc_difficulty_cell: BtcDifficulty {
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex.clone(),
                index: 4,
            },
        },
    };
    log::info!("settings: {:?}", &settings);
    settings.write(&config_path).map_err(|e| anyhow!(e))?;
    println!("tockb config written to {}", &config_path);
    Ok(())
}

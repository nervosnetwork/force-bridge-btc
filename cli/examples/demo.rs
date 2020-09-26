use anyhow::Result;
use ckb_hash::blake2b_256;
use ckb_sdk::{Address, AddressPayload, HttpRpcClient, SECP256K1};
use ckb_types::prelude::Pack;
use ckb_types::H256;
use ckb_types::{core::TransactionView, packed::Script};
use molecule::prelude::{Builder, Entity};
use std::str::FromStr;
use tockb_sdk::settings::{BtcDifficulty, OutpointConf, PriceOracle, ScriptConf};
use tockb_sdk::tx_helper::{deploy, sign};
use tockb_sdk::util::send_tx_sync;
use tockb_sdk::{generator::Generator, indexer::IndexerRpcClient, settings::Settings};
use tockb_types::generated::btc_difficulty::BTCDifficulty;

const TIMEOUT: u64 = 60;

fn main() -> Result<()> {
    let rpc_url = "http://127.0.0.1:8114".to_owned();
    let indexer_url = "http://127.0.0.1:8116".to_owned();
    let mut rpc_client = HttpRpcClient::new(rpc_url.clone());
    let mut indexer_client = IndexerRpcClient::new(indexer_url.clone());

    let private_key_hex = "d00c06bfd800d27397002dca6fb0993d5ba6399b4238b2f29ee9deb97593d2bc";
    let private_key = secp256k1::SecretKey::from_str(private_key_hex)?;
    let public_key = secp256k1::PublicKey::from_secret_key(&SECP256K1, &private_key);
    let address_payload = AddressPayload::from_pubkey(&public_key);
    let from_lockscript = Script::from(&address_payload);

    // dev deploy
    let typescript_bin = std::fs::read("../build/release/toCKB-typescript")?;
    let lockscript_bin = std::fs::read("../build/release/toCKB-lockscript")?;
    let sudt_bin = std::fs::read("../tests/deps/simple_udt")?;
    let typescript_code_hash = blake2b_256(&typescript_bin);
    let typescript_code_hash_hex = hex::encode(&typescript_code_hash);
    let lockscript_code_hash = blake2b_256(&lockscript_bin);
    let lockscript_code_hash_hex = hex::encode(&lockscript_code_hash);
    let sudt_code_hash = blake2b_256(&sudt_bin);
    let sudt_code_hash_hex = hex::encode(&sudt_code_hash);
    let price = 10000u128;
    let btc_difficulty: u64 = 17345997805929;
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
    let tx_hash = send_tx_sync(&mut rpc_client, tx.clone(), TIMEOUT).unwrap();
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
    // dbg!(&settings);

    let user_address = "ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37";
    let user_lockscript = Script::from(Address::from_str(user_address).unwrap().payload());

    let tx_fee = 1000_0000;
    let mut generator = Generator::new(rpc_url, indexer_url, settings).unwrap();
    let unsigned_tx = generator
        .deposit_request(from_lockscript, tx_fee, user_lockscript, 10000, 1, 1)
        .unwrap();
    let tx = sign(unsigned_tx, &mut rpc_client, &private_key).unwrap();
    send_tx_sync(&mut rpc_client, tx.clone(), 60).unwrap();

    Ok(())
}

use super::types::{SudtArgs, SudtSubCommand};
use anyhow::{anyhow, Result};
use ckb_sdk::{Address, AddressPayload, HttpRpcClient, HumanCapacity, SECP256K1};
use ckb_types::packed::{Script, ScriptReader};
use ckb_types::prelude::Entity;
use molecule::prelude::Reader;
use std::str::FromStr;
use tockb_sdk::indexer::IndexerRpcClient;
use tockb_sdk::tx_helper::sign;
use tockb_sdk::util::{ensure_indexer_sync, parse_privkey_path, send_tx_sync};
use tockb_sdk::{generator::Generator, settings::Settings};

pub fn parse_cell(cell: &str) -> Result<Script> {
    let cell_bytes =
        hex::decode(cell).map_err(|e| anyhow!("cell shoule be hex format, err: {}", e))?;
    ScriptReader::verify(&cell_bytes, false).map_err(|e| anyhow!("cell decoding err: {}", e))?;
    let cell_typescript = Script::new_unchecked(cell_bytes.into());
    Ok(cell_typescript)
}

pub fn sudt_handler(args: SudtArgs) -> Result<()> {
    let mut rpc_client = HttpRpcClient::new(args.rpc_url.clone());
    let mut indexer_client = IndexerRpcClient::new(args.indexer_url.clone());
    ensure_indexer_sync(&mut rpc_client, &mut indexer_client, 60).unwrap();
    let settings = Settings::new(&args.config_path)?;
    let mut generator = Generator::new(args.rpc_url.clone(), args.indexer_url.clone(), settings)
        .map_err(|e| anyhow::anyhow!(e))?;

    let kind = args.kind;

    match args.subcmd {
        SudtSubCommand::Transfer(args) => {
            let from_privkey = parse_privkey_path(&args.private_key_path)?;
            let from_public_key = secp256k1::PublicKey::from_secret_key(&SECP256K1, &from_privkey);
            let address_payload = AddressPayload::from_pubkey(&from_public_key);
            let from_lockscript = Script::from(&address_payload);

            let tx_fee: u64 = HumanCapacity::from_str(&args.tx_fee)
                .map_err(|e| anyhow!(e))?
                .into();

            let to_lockscript = Script::from(Address::from_str(&args.to_addr).unwrap().payload());
            let ckb_amount = HumanCapacity::from_str(&args.ckb_amount)
                .map_err(|e| anyhow!(e))?
                .into();
            let unsigned_tx = generator
                .transfer_sudt(
                    from_lockscript,
                    kind,
                    to_lockscript,
                    args.sudt_amount,
                    ckb_amount,
                    tx_fee,
                )
                .unwrap();

            let tx = sign(unsigned_tx, &mut rpc_client, &from_privkey).unwrap();
            log::info!(
                "tx: \n{}",
                serde_json::to_string_pretty(&ckb_jsonrpc_types::TransactionView::from(tx.clone()))
                    .unwrap()
            );
            if args.wait_for_committed {
                send_tx_sync(&mut rpc_client, &tx, 60).map_err(|e| anyhow::anyhow!(e))?;
            }
            let cell_typescript = tx.output(0).unwrap().type_().to_opt().unwrap();
            let print_res = serde_json::json!({
                "tx_hash": hex::encode(tx.hash().as_slice()),
                "cell_typescript": hex::encode(cell_typescript.as_slice()),
            });
            println!("{}", serde_json::to_string_pretty(&print_res)?);
            Ok(())
        }
        SudtSubCommand::GetBalance(args) => {
            let balance = generator.get_sudt_balance(args.addr, kind).unwrap();
            println!("{:?}", balance);
            Ok(())
        }
    }
}

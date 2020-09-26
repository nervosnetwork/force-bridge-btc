use super::types::{ContractArgs, ContractSubCommand};
use anyhow::{anyhow, Result};
use ckb_sdk::{Address, AddressPayload, HttpRpcClient, HumanCapacity, SECP256K1};
use ckb_types::core::TransactionView;
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

pub fn contract_tx_generator(
    config_path: String,
    rpc_url: String,
    indexer_url: String,
    tx_fee: String,
    from_lockscript: Script,
    subcmd: ContractSubCommand,
) -> Result<TransactionView> {
    let mut rpc_client = HttpRpcClient::new(rpc_url.clone());
    let mut indexer_client = IndexerRpcClient::new(indexer_url.clone());
    ensure_indexer_sync(&mut rpc_client, &mut indexer_client, 60).unwrap();
    let settings = Settings::new(&config_path)?;
    let mut generator = Generator::new(rpc_url.clone(), indexer_url.clone(), settings)
        .map_err(|e| anyhow::anyhow!(e))?;
    let tx_fee: u64 = HumanCapacity::from_str(&tx_fee)
        .map_err(|e| anyhow!(e))?
        .into();

    let unsigned_tx = match subcmd {
        ContractSubCommand::DepositRequest(args) => {
            let user_lockscript = Script::from(
                Address::from_str(&args.user_lockscript_addr)
                    .unwrap()
                    .payload(),
            );
            generator
                .deposit_request(
                    from_lockscript,
                    tx_fee,
                    user_lockscript,
                    args.pledge,
                    args.kind,
                    args.lot_size,
                )
                .unwrap()
        }
        ContractSubCommand::Bonding(args) => {
            let signer_lockscript = Script::from(
                Address::from_str(&args.signer_lockscript_addr)
                    .unwrap()
                    .payload(),
            );
            let cell_typescript = parse_cell(&args.cell)?;
            generator
                .bonding(
                    from_lockscript,
                    tx_fee,
                    cell_typescript,
                    signer_lockscript,
                    args.lock_address,
                )
                .unwrap()
        }
        ContractSubCommand::MintXt(args) => {
            let cell_typescript = parse_cell(&args.cell)?;
            let spv_proof = hex::decode(&args.spv_proof)?;
            generator
                .mint_xt(from_lockscript, tx_fee, cell_typescript, spv_proof)
                .unwrap()
        }
        ContractSubCommand::PreTermRedeem(args) => {
            let cell_typescript = parse_cell(&args.cell)?;
            let redeemer_lockscript = Script::from(
                Address::from_str(&args.redeemer_lockscript_addr)
                    .unwrap()
                    .payload(),
            );
            generator
                .pre_term_redeem(
                    from_lockscript,
                    tx_fee,
                    cell_typescript,
                    args.unlock_address,
                    redeemer_lockscript,
                )
                .unwrap()
        }
    };
    Ok(unsigned_tx)
}

pub fn contract_handler(args: ContractArgs) -> Result<()> {
    let mut rpc_client = HttpRpcClient::new(args.rpc_url.clone());
    let from_privkey = parse_privkey_path(&args.private_key_path)?;
    let from_public_key = secp256k1::PublicKey::from_secret_key(&SECP256K1, &from_privkey);
    let address_payload = AddressPayload::from_pubkey(&from_public_key);
    let from_lockscript = Script::from(&address_payload);
    let unsigned_tx = contract_tx_generator(
        args.config_path.clone(),
        args.rpc_url.clone(),
        args.indexer_url.clone(),
        args.tx_fee.clone(),
        from_lockscript,
        args.subcmd,
    )?;
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

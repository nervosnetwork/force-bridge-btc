use super::types::{ContractArgs, ContractSubCommand};
use crate::tx_helper::sign;
use crate::util::{parse_privkey_path, send_tx_sync};
use crate::{generator::Generator, settings::Settings};
use anyhow::{anyhow, Result};
use ckb_sdk::{Address, AddressPayload, HttpRpcClient, HumanCapacity, SECP256K1};
use ckb_types::packed::Script;
use std::str::FromStr;

pub fn contract_handler(args: ContractArgs) -> Result<()> {
    dbg!(&args);
    let mut rpc_client = HttpRpcClient::new(args.rpc_url.clone());
    let settings = Settings::new(&args.config_path)?;
    let mut generator = Generator::new(args.rpc_url.clone(), args.indexer_url.clone(), settings)
        .map_err(|e| anyhow::anyhow!(e))?;
    let from_privkey = parse_privkey_path(&args.private_key)?;
    let from_public_key = secp256k1::PublicKey::from_secret_key(&SECP256K1, &from_privkey);
    let address_payload = AddressPayload::from_pubkey(&from_public_key);
    let from_lockscript = Script::from(&address_payload);
    let tx_fee: u64 = HumanCapacity::from_str(&args.tx_fee)
        .map_err(|e| anyhow!(e))?
        .into();

    let unsigned_tx = match args.subcmd {
        ContractSubCommand::DepositRequest(deposit_request_args) => {
            let user_lockscript = Script::from(
                Address::from_str(&deposit_request_args.user_lockscript_addr)
                    .unwrap()
                    .payload(),
            );
            generator
                .deposit_request(
                    from_lockscript,
                    tx_fee,
                    user_lockscript,
                    deposit_request_args.pledge,
                    deposit_request_args.kind,
                    deposit_request_args.lot_size,
                )
                .unwrap()
        }
    };
    let rpc_tx = ckb_jsonrpc_types::TransactionView::from(unsigned_tx.clone());
    dbg!(rpc_tx);
    let tx = sign(unsigned_tx, &mut rpc_client, &from_privkey).unwrap();
    send_tx_sync(&mut rpc_client, tx, 60).map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
}

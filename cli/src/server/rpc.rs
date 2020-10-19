use crate::commands::contract::contract_tx_generator;
use crate::commands::types::{ContractSubCommand, ServerArgs};
use ckb_sdk::Address;
use ckb_types::packed::Script;
use jsonrpc_http_server::jsonrpc_core::*;
use jsonrpc_http_server::ServerBuilder;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

// #[rpc]
// pub trait Rpc {
//     /// Adds two numbers and returns a result
//     #[rpc(name = "add")]
//     fn contract(
//         &self,
//         from_lockscript: String,
//         tx_fee: String,
//         sub_cmd: ContractSubCommand,
//     ) -> Result<TransactionView>;
// }
//
// pub struct RpcImpl;
// impl Rpc for RpcImpl {
//     fn contract(
//         &self,
//         from_lockscript: String,
//         tx_fee: String,
//         sub_cmd: ContractSubCommand,
//     ) -> Result<TransactionView> {
//         todo!()
//     }
// }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonrpcContractArgs {
    from_lockscript_addr: String,
    tx_fee: String,
    sub_cmd: ContractSubCommand,
}

pub fn start(args: ServerArgs) {
    let threads_num = args.threads_num;
    let listen_url = args.listen_url.clone();
    let mut io = jsonrpc_core::IoHandler::new();
    // io.extend_with(RpcImpl.to_delegate());
    io.add_method("contract", move |params: Params| {
        dbg!(&params);
        let rpc_args: JsonrpcContractArgs = params
            .parse()
            .map_err(|_e| jsonrpc_core::Error::parse_error())?;
        let from_lockscript = Script::from(
            Address::from_str(&rpc_args.from_lockscript_addr)
                .map_err(|_e| jsonrpc_core::Error::parse_error())?
                .payload(),
        );
        let tx = contract_tx_generator(
            args.config_path.clone(),
            args.rpc_url.clone(),
            args.indexer_url.clone(),
            rpc_args.tx_fee.clone(),
            from_lockscript,
            rpc_args.sub_cmd.clone(),
        )
        .map_err(|_e| jsonrpc_core::Error::internal_error())?;
        let rpc_tx = ckb_jsonrpc_types::TransactionView::from(tx);
        Ok(serde_json::to_value(rpc_tx).unwrap())
    });
    let server = ServerBuilder::new(io)
        .threads(threads_num)
        .start_http(&listen_url.parse().unwrap())
        .unwrap();
    server.wait();
}

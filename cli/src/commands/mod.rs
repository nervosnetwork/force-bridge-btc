pub mod contract;
pub mod types;

use anyhow::Result;
use contract::contract_handler;
use types::{ContractSubCommand, Opts, SubCommand};

pub fn handler(opt: Opts) -> Result<()> {
    match opt.subcmd {
        SubCommand::Contract(contract_args) => contract_handler(contract_args),
        _ => todo!(),
    }
}

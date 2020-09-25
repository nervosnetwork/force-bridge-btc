pub mod types;
pub mod contract;

use anyhow::Result;
use types::{Opts, SubCommand, ContractSubCommand};
use contract::contract_handler;


pub fn handler(opt: Opts) -> Result<()> {
   match opt.subcmd {
     SubCommand::Contract(contract_args) => {
        contract_handler(contract_args)
     }
       _ => todo!()
   }
}

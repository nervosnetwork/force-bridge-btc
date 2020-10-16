pub mod commands;
pub mod server;

use crate::commands::handler;
use crate::commands::types::Opts;
use anyhow::Result;
use clap::Clap;

fn main() -> Result<()> {
    env_logger::init();
    let opts: Opts = Opts::parse();
    dbg!(&opts);
    handler(opts)
}

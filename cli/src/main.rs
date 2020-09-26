use anyhow::{anyhow, Result};
use clap::Clap;
use tockb_sdk::commands::handler;
use tockb_sdk::commands::types::Opts;

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    dbg!(&opts);
    handler(opts)
}

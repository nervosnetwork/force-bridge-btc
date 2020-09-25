use clap::Clap;
use tockb_sdk::commands::types::Opts;
use tockb_sdk::commands::handler;
use anyhow::{anyhow, Result};

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    dbg!(&opts);
    handler(opts)
}

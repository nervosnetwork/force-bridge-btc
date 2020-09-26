use clap::Clap;

/// toCKB sdk
#[derive(Clap, Clone, Debug)]
#[clap(version = "0.1", author = "Wenchao Hu <me@huwenchao.com>")]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap, Clone, Debug)]
pub enum SubCommand {
    Init(InitArgs),
    Utils(UtilsArgs),
    Contract(ContractArgs),
}

/// init tockb sdk config
#[derive(Clap, Clone, Debug)]
pub struct InitArgs {
    #[clap(short = 'f', long)]
    pub force: bool,
}

#[derive(Clap, Clone, Debug)]
pub struct UtilsArgs {}

#[derive(Clap, Clone, Debug)]
pub struct ContractArgs {
    #[clap(long, default_value = ".tockb-config.toml")]
    pub config_path: String,
    #[clap(long, default_value = "http://127.0.0.1:8114")]
    pub rpc_url: String,
    #[clap(long, default_value = "http://127.0.0.1:8116")]
    pub indexer_url: String,
    #[clap(long, default_value = "0.1")]
    pub tx_fee: String,
    #[clap(short, long)]
    pub private_key: String,
    #[clap(long)]
    pub wait_for_committed: bool,
    #[clap(subcommand)]
    pub subcmd: ContractSubCommand,
}

#[derive(Clap, Clone, Debug)]
pub enum ContractSubCommand {
    DepositRequest(DepositRequestArgs),
}

#[derive(Clap, Clone, Debug)]
pub struct DepositRequestArgs {
    #[clap(short, long)]
    pub user_lockscript_addr: String,
    #[clap(short, long)]
    pub pledge: u64,
    #[clap(short, long)]
    pub kind: u8,
    #[clap(short, long)]
    pub lot_size: u8,
}

use clap::Clap;
use serde::{Deserialize, Serialize};

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
    DevInit(DevInitArgs),
    Utils(UtilsArgs),
    Contract(ContractArgs),
    Sudt(SudtArgs),
    Server(ServerArgs),
}

#[derive(Clap, Clone, Debug)]
pub struct ServerArgs {
    #[clap(long, default_value = "/tmp/.tockb-cli/config.toml")]
    pub config_path: String,
    #[clap(long, default_value = "http://127.0.0.1:8114")]
    pub rpc_url: String,
    #[clap(long, default_value = "http://127.0.0.1:8116")]
    pub indexer_url: String,
    #[clap(short, long, default_value = "127.0.0.1:3030")]
    pub listen_url: String,
    #[clap(short, long, default_value = "3")]
    pub threads_num: usize,
}

/// init tockb sdk config
#[derive(Clap, Clone, Debug)]
pub struct InitArgs {
    #[clap(short = 'f', long)]
    pub force: bool,
    #[clap(long, default_value = "/tmp/.tockb-cli/config.toml")]
    pub config_path: String,
}

#[derive(Clap, Clone, Debug)]
pub struct DevInitArgs {
    #[clap(short = 'f', long)]
    pub force: bool,
    #[clap(long, default_value = "/tmp/.tockb-cli/config.toml")]
    pub config_path: String,
    #[clap(long, default_value = "http://127.0.0.1:8114")]
    pub rpc_url: String,
    #[clap(long, default_value = "http://127.0.0.1:8116")]
    pub indexer_url: String,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(long, default_value = "../build/release/toCKB-typescript")]
    pub typescript_path: String,
    #[clap(long, default_value = "../build/release/toCKB-lockscript")]
    pub lockscript_path: String,
    #[clap(long, default_value = "../tests/deps/simple_udt")]
    pub sudt_path: String,
    #[clap(short, long)]
    pub price: u128,
    #[clap(short = 'd', long)]
    pub btc_difficulty: u64,
}

#[derive(Clap, Clone, Debug)]
pub struct UtilsArgs {}

#[derive(Clap, Clone, Debug)]
pub struct ContractArgs {
    #[clap(long, default_value = "/tmp/.tockb-cli/config.toml")]
    pub config_path: String,
    #[clap(long, default_value = "http://127.0.0.1:8114")]
    pub rpc_url: String,
    #[clap(long, default_value = "http://127.0.0.1:8116")]
    pub indexer_url: String,
    #[clap(long, default_value = "0.1")]
    pub tx_fee: String,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(short, long)]
    pub wait_for_committed: bool,
    #[clap(subcommand)]
    pub subcmd: ContractSubCommand,
}

#[derive(Clap, Clone, Debug, Serialize, Deserialize)]
pub enum ContractSubCommand {
    DepositRequest(DepositRequestArgs),
    Bonding(BondingArgs),
    MintXt(MintXTArgs),
    PreTermRedeem(PreTermRedeemArgs),
    WithdrawCollateral(WithdrawCollateralArgs),
}

#[derive(Clap, Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clap, Clone, Debug, Serialize, Deserialize)]
pub struct BondingArgs {
    /// cell typescript hex
    #[clap(short, long)]
    pub cell: String,
    #[clap(short, long)]
    pub signer_lockscript_addr: String,
    #[clap(short, long)]
    pub lock_address: String,
}

#[derive(Clap, Clone, Debug, Serialize, Deserialize)]
pub struct MintXTArgs {
    #[clap(short, long)]
    pub cell: String,
    #[clap(short, long)]
    pub spv_proof: String,
}

#[derive(Clap, Clone, Debug, Serialize, Deserialize)]
pub struct PreTermRedeemArgs {
    #[clap(short, long)]
    pub cell: String,
    #[clap(short, long)]
    pub unlock_address: String,
    #[clap(short, long)]
    pub redeemer_lockscript_addr: String,
}

#[derive(Clap, Clone, Debug, Serialize, Deserialize)]
pub struct WithdrawCollateralArgs {
    #[clap(short, long)]
    pub cell: String,
    #[clap(short, long)]
    pub spv_proof: String,
}

#[derive(Clap, Clone, Debug)]
pub struct SudtArgs {
    #[clap(long, default_value = "/tmp/.tockb-cli/config.toml")]
    pub config_path: String,
    #[clap(long, default_value = "http://127.0.0.1:8114")]
    pub rpc_url: String,
    #[clap(long, default_value = "http://127.0.0.1:8116")]
    pub indexer_url: String,
    #[clap(long, default_value = "0.1")]
    pub tx_fee: String,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(short)]
    pub kind: u8,
    #[clap(subcommand)]
    pub subcmd: SudtSubCommand,
}

#[derive(Clap, Clone, Debug, Serialize, Deserialize)]
pub enum SudtSubCommand {
    Transfer(SudtTransferArgs),
    GetBalance(SudtGetBalanceArgs),
}

#[derive(Clap, Clone, Debug, Serialize, Deserialize)]
pub struct SudtTransferArgs {
    #[clap(short, long)]
    pub to_addr: String,
    #[clap(long)]
    pub sudt_amount: u128,
    #[clap(long, default_value = "200")]
    pub ckb_amount: String,
    #[clap(short, long)]
    pub wait_for_committed: bool,
}

#[derive(Clap, Clone, Debug, Serialize, Deserialize)]
pub struct SudtGetBalanceArgs {
    #[clap(short, long)]
    pub addr: String,
}

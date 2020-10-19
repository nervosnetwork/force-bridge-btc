use super::types::ServerArgs;
use crate::server::rpc::start;
use anyhow::Result;

pub fn server_handler(args: ServerArgs) -> Result<()> {
    start(args);
    Ok(())
}

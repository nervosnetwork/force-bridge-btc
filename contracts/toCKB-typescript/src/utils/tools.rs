use super::types::Error;
use ckb_std::ckb_types::{bytes::Bytes, prelude::Unpack};
use ckb_std::high_level::load_script;
use core::result::Result;
use int_enum::IntEnum;

#[repr(u8)]
#[derive(Clone, Copy, IntEnum)]
pub enum XChainKind {
    Btc = 1,
    Eth = 2,
}

pub fn get_xchain_kind() -> Result<XChainKind, Error> {
    let script_args: Bytes = load_script()?.args().unpack();
    if script_args.len() != 1 {
        return Err(Error::Encoding);
    }
    let mut buf = [0u8; 1];
    buf.copy_from_slice(script_args.as_ref());
    let kind = u8::from_le_bytes(buf);
    XChainKind::from_int(kind).map_err(|_| Error::Encoding)
}

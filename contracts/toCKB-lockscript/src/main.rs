#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![allow(non_snake_case)]

use ckb_std::high_level::{load_cell_type, load_script, QueryIter};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, prelude::*},
    default_alloc, entry,
    error::SysError,
};
use core::result::Result;
entry!(entry);
default_alloc!();

// total_size(4 byte) + offset(4 byte) * 3 + code_hash(32 byte) + hash_type(1 byte) + args_size(4 byte) + xchain_kind(1 byte) = 54 byte
const MOLECULE_TYPESCRIPT_SIZE: usize = 54;

/// Program entry
fn entry() -> i8 {
    // Call main function and return error code
    match main() {
        Ok(_) => 0,
        Err(err) => err as i8,
    }
}

/// Error
#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    // Add customized errors here...
    InvalidToCKBCell,
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}

fn main() -> Result<(), Error> {
    verify()
}

fn verify() -> Result<(), Error> {
    let args: Bytes = load_script()?.args().unpack();
    let count = QueryIter::new(load_cell_type, Source::GroupInput)
        .filter(|type_script_opt| {
            type_script_opt.is_none()
                || (type_script_opt.as_ref().unwrap().as_slice()[0..MOLECULE_TYPESCRIPT_SIZE]
                    != args[..])
        })
        .count();
    if 0 != count {
        return Err(Error::InvalidToCKBCell);
    }
    Ok(())
}

#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![allow(non_snake_case)]

mod utils;

use alloc::vec;
use ckb_std::high_level::load_script;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, prelude::*},
    debug, default_alloc, entry,
    error::SysError,
    high_level::{load_cell_lock_hash, load_cell_type_hash, load_script_hash},
};
use core::result::Result;
use hex;
use utils::error::Error;
entry!(entry);
default_alloc!();

/// Program entry
fn entry() -> i8 {
    // Call main function and return error code
    match main() {
        Ok(_) => 0,
        Err(err) => err as i8,
    }
}

fn main() -> Result<(), Error> {
    verify()
}

fn verify() -> Result<(), Error> {
    // load current lock_script hash
    let script_hash = load_script_hash().unwrap();
    let args: Bytes = load_script().unwrap().args().unpack();
    let cell_source = vec![Source::Input, Source::Output];

    for &source in cell_source.iter() {
        for i in 0.. {
            match verify_single_cell(i, source, script_hash, args.clone()) {
                Ok(()) => {}
                Err(Error::IndexOutOfBound) => break,
                Err(err) => return Err(err.into()),
            };
        }
    }
    Ok(())
}

fn verify_single_cell(
    index: usize,
    source: Source,
    script_hash: [u8; 32],
    args: Bytes,
) -> Result<(), Error> {
    match load_cell_type_hash(index, source) {
        Ok(type_hash_opt) => {
            if type_hash_opt.is_none() || type_hash_opt.unwrap()[..] != args[..] {
                return Ok(());
            }
        }
        Err(SysError::IndexOutOfBound) => return Err(Error::IndexOutOfBound),
        Err(err) => return Err(err.into()),
    };

    let lock_hash = load_cell_lock_hash(index, source)?;

    //the toCKBCell is valid when the toCKB cell lock_script hash equal current lock_script hash
    if lock_hash[..] != script_hash[..] {
        return Err(Error::InvalidToCKBCell);
    }
    Ok(())
}

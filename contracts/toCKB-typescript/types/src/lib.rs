#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![allow(non_snake_case)]

// TODO toCKBcell after molecule decode
pub struct ToCKBCell {}

// (input, output)
pub struct ToCKBCellTuple(ToCKBCell, ToCKBCell);

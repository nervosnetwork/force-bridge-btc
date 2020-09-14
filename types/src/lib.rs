#![cfg_attr(not(feature = "std"), no_std)]

pub mod config;
pub mod convert;
pub mod error;
pub mod generated;
pub mod tockb_cell;

pub use error::Error;
pub use generated::*;
pub use tockb_cell::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
    } else {
        extern crate alloc;
    }
}

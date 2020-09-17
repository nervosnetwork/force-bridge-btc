#[cfg(not(feature = "std"))]
use alloc::borrow::ToOwned;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use ckb_std::ckb_types::packed;
#[cfg(feature = "std")]
use ckb_types::packed;

use crate::generated::basic::{Byte32, Byte4, Bytes, Uint32, Uint32Reader, Uint64, Script, OutPoint};
use core::convert::TryFrom;
use molecule::{
    error::VerificationError,
    prelude::{Builder, Byte, Entity},
};

impl From<Vec<u8>> for Bytes {
    fn from(v: Vec<u8>) -> Self {
        Bytes::new_builder()
            .set(v.into_iter().map(Byte::new).collect())
            .build()
    }
}

impl TryFrom<Vec<u8>> for Byte4 {
    type Error = VerificationError;
    fn try_from(v: Vec<u8>) -> Result<Self, VerificationError> {
        if v.len() != 4 {
            return Err(VerificationError::TotalSizeNotMatch(
                "Byte4".to_owned(),
                4,
                v.len(),
            ));
        }
        let mut inner = [Byte::new(0); 4];
        let v = v.into_iter().map(Byte::new).collect::<Vec<_>>();
        inner.copy_from_slice(&v);
        Ok(Self::new_builder().set(inner).build())
    }
}

impl TryFrom<Vec<u8>> for Byte32 {
    type Error = VerificationError;
    fn try_from(v: Vec<u8>) -> Result<Self, VerificationError> {
        if v.len() != 32 {
            return Err(VerificationError::TotalSizeNotMatch(
                "Byte32".to_owned(),
                32,
                v.len(),
            ));
        }
        let mut inner = [Byte::new(0); 32];
        let v = v.into_iter().map(Byte::new).collect::<Vec<_>>();
        inner.copy_from_slice(&v);
        Ok(Self::new_builder().set(inner).build())
    }
}

impl From<u32> for Uint32 {
    fn from(v: u32) -> Self {
        let mut inner = [Byte::new(0); 4];
        let v = v
            .to_le_bytes()
            .to_vec()
            .into_iter()
            .map(Byte::new)
            .collect::<Vec<_>>();
        inner.copy_from_slice(&v);
        Self::new_builder().set(inner).build()
    }
}

impl From<Uint32> for u32 {
    fn from(v: Uint32) -> Self {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(v.raw_data().as_ref());
        u32::from_le_bytes(buf)
    }
}

impl From<Uint32Reader<'_>> for u32 {
    fn from(v: Uint32Reader<'_>) -> Self {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(v.raw_data());
        u32::from_le_bytes(buf)
    }
}

impl From<u64> for Uint64 {
    fn from(v: u64) -> Self {
        let mut inner = [Byte::new(0); 8];
        let v = v
            .to_le_bytes()
            .to_vec()
            .into_iter()
            .map(Byte::new)
            .collect::<Vec<_>>();
        inner.copy_from_slice(&v);
        Self::new_builder().set(inner).build()
    }
}

impl From<packed::Script> for Script {
    fn from(v: packed::Script) -> Self {
        Self::new_unchecked(v.as_bytes())
    }
}

impl From<packed::OutPoint> for OutPoint {
    fn from(v: packed::OutPoint) -> Self {
        Self::new_unchecked(v.as_bytes())
    }
}

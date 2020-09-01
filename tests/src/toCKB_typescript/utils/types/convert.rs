use super::generated::basic::{Byte32, Byte4, Bytes, Uint32, Uint64};
use molecule::prelude::{Builder, Byte, Entity};

impl From<Vec<u8>> for Bytes {
    fn from(v: Vec<u8>) -> Self {
        Bytes::new_builder()
            .set(v.into_iter().map(Byte::new).collect())
            .build()
    }
}

impl From<Vec<u8>> for Byte4 {
    fn from(v: Vec<u8>) -> Self {
        if v.len() != 4 {
            panic!("length for Byte4 should be 4")
        }
        let mut inner = [Byte::new(0); 4];
        let v = v.into_iter().map(Byte::new).collect::<Vec<_>>();
        inner.copy_from_slice(&v);
        Self::new_builder().set(inner).build()
    }
}

impl From<Vec<u8>> for Byte32 {
    fn from(v: Vec<u8>) -> Self {
        if v.len() != 32 {
            panic!("length for Byte32 should be 32")
        }
        let mut inner = [Byte::new(0); 32];
        let v = v.into_iter().map(Byte::new).collect::<Vec<_>>();
        inner.copy_from_slice(&v);
        Self::new_builder().set(inner).build()
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

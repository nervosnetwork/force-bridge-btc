pub const PLEDGE: u64 = 10000;

// since
pub const LOCK_TYPE_FLAG: u64 = 1 << 63;
pub const SINCE_TYPE_TIMESTAMP: u64 = 0x4000_0000_0000_0000;
pub const N4: u64 = 3 * 24 * 3600; // timeout
pub(crate) const SINCE_SIGNER_TIMEOUT: u64 = LOCK_TYPE_FLAG | SINCE_TYPE_TIMESTAMP | N4;

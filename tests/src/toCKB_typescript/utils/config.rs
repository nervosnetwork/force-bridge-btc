pub const PLEDGE: u64 = 10000;

// since
pub const LOCK_TYPE_FLAG: u64 = 1 << 63;
pub const SINCE_TYPE_TIMESTAMP: u64 = 0x4000_0000_0000_0000;

// 24 * 3600 means 1 day, the unit is second
pub const SINCE_SIGNER_TIMEOUT: u64 = LOCK_TYPE_FLAG | SINCE_TYPE_TIMESTAMP | 24 * 3600;
pub const SINCE_WITHDRAW_PLEDGE: u64 = LOCK_TYPE_FLAG | 100;

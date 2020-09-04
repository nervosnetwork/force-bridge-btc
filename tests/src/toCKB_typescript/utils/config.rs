pub const CKB_UNITS: u64 = 100_000_000;
pub const PLEDGE: u64 = 10000 * CKB_UNITS;
pub const XT_CELL_CAPACITY: u64 = 200 * CKB_UNITS;

// since
pub const LOCK_TYPE_FLAG: u64 = 1 << 63;
pub const SINCE_TYPE_TIMESTAMP: u64 = 0x4000_0000_0000_0000;

// 24 * 3600 means 1 day, the unit is second
pub const SINCE_SIGNER_TIMEOUT: u64 = LOCK_TYPE_FLAG | SINCE_TYPE_TIMESTAMP | 24 * 3600;

pub const PLEDGE: u64 = 10000;

// since
pub const LOCK_TYPE_FLAG: u64 = 1 << 63;
pub const METRIC_TYPE_FLAG_MASK: u64 = 0x6000_0000_0000_0000;
pub const VALUE_MASK: u64 = 0x00ff_ffff_ffff_ffff;
pub const REMAIN_FLAGS_BITS: u64 = 0x1f00_0000_0000_0000;
pub const SINCE_TYPE_TIMESTAMP: u64 = 0x4000_0000_0000_0000;

// n4 time
pub const N4: u64 = 3 * 24 * 3600;

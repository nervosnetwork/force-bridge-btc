pub const PLEDGE: u64 = 10000;
pub const XT_CELL_CAPACITY: u64 = 200;
pub const TX_PROOF_DIFFICULTY_FACTOR: u8 = 1;
// default value is 2/1000
pub const SIGNER_FEE_RATE: (u128, u128) = (2, 1000);
pub const SUDT_CODE_HASH: [u8; 32] = [
    225, 227, 84, 214, 214, 67, 173, 66, 114, 77, 64, 150, 126, 51, 73, 132, 83, 78, 3, 103, 64,
    92, 90, 228, 42, 157, 125, 99, 215, 125, 244, 25,
];

// since
pub const LOCK_TYPE_FLAG: u64 = 1 << 63;
pub const SINCE_TYPE_TIMESTAMP: u64 = 0x4000_0000_0000_0000;

// 24 * 3600 means 1 day, the unit is second
pub const SINCE_SIGNER_TIMEOUT: u64 = LOCK_TYPE_FLAG | SINCE_TYPE_TIMESTAMP | 24 * 3600;

// LIQUIDATION_COLLATERAL_PERCENT means min liquidation threshold of collateral/lot_amount
pub const LIQUIDATION_COLLATERAL_PERCENT: u8 = 115;
pub const CKB_DECIMAL: u128 = 10000_0000;

pub mod convert;
pub mod generated;
pub mod test_case;

#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    // Add customized errors here...
    XChainMismatch,
    TxInvalid,
    LotSizeInvalid,
    PledgeInvalid,
    // MintXT Error
    InvalidDataChange,
    InvalidWitness,
    DifficultyDataInvalid,
    SpvProofInvalid,
    InsufficientDifficulty,
    BadMerkleProof,
    NotAtCurrentOrPreviousDifficulty,
    WrongTxId,
    FundingNotEnough,
    UnsupportedFundingType,
    InvalidMintOutput,
    WrongFundingAddr,
    InvalidXTInInputOrOutput,
    InvalidXTMint,

    CapacityInvalid,
    InvariantDataMutated,
    InputSinceInvalid,
    UndercollateralInvalid,
    WitnessInvalid,
    XChainAddressInvalid,
    CollateralInvalid,
    XTBurnInvalid,
    InputSignerInvalid,

    // Faulty witness
    FaultyBtcWitnessInvalid,

    // Auction
    InvalidInputs,
    InvalidAuctionBidderCell,
    InvalidTriggerOrSignerCell,
    InvalidAuctionXTCell,
    XTAmountInvalid,
}

#[repr(u8)]
pub enum ToCKBStatus {
    Initial = 1,
    Bonded = 2,
    Warranty = 3,
    Redeeming = 4,
    SignerTimeout = 5,
    Undercollateral = 6,
    FaultyWhenWarranty = 7,
    FaultyWhenRedeeming = 8,
}

#[repr(u8)]
pub enum BtcLotSize {
    Quarter = 1,
    Half = 2,
    Single = 3,
}

#[repr(u8)]
pub enum EthLotSize {
    Quarter = 1,
    Half = 2,
    Single = 3,
    Two = 4,
    Three = 5,
    Four = 6,
}

#[repr(u8)]
pub enum XChainKind {
    Btc = 1,
    Eth = 2,
}

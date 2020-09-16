use int_enum::{IntEnum, IntEnumError};

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
    CellIDInvalid,
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

impl<T: IntEnum> From<IntEnumError<T>> for Error {
    fn from(_err: IntEnumError<T>) -> Self {
        Error::Encoding
    }
}

#[cfg(feature = "contract")]
impl From<ckb_std::error::SysError> for Error {
    fn from(err: ckb_std::error::SysError) -> Self {
        use ckb_std::error::SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}

#[cfg(feature = "contract")]
impl From<bitcoin_spv::types::SPVError> for Error {
    fn from(_err: bitcoin_spv::types::SPVError) -> Self {
        Error::SpvProofInvalid
    }
}

use super::error::Error;
use super::generated::toCKB_cell_data::ToCKBCellDataReader;
use ckb_std::ckb_types::bytes::Bytes;
use core::result::Result;
use int_enum::IntEnum;
use molecule::prelude::*;

const BTC_UNIT: u128 = 100_000_000;
const ETH_UNIT: u128 = 1_000_000_000_000_000_000;

pub struct ToCKBCellDataView {
    pub status: ToCKBStatus,
    lot_size: u8,
    pub user_lockscript: Bytes,
    pub x_lock_address: Bytes,
    pub signer_lockscript: Bytes,
    pub x_unlock_address: Bytes,
    pub redeemer_lockscript: Bytes,
    pub liquidation_trigger_lockscript: Bytes,
}

impl ToCKBCellDataView {
    pub fn from_slice(slice: &[u8]) -> Result<ToCKBCellDataView, Error> {
        ToCKBCellDataReader::verify(slice, false).map_err(|_| Error::Encoding)?;
        let data_reader = ToCKBCellDataReader::new_unchecked(slice);
        let status = ToCKBStatus::from_int(data_reader.status().to_entity().into())?;
        let lot_size = data_reader.lot_size().as_slice()[0];
        let user_lockscript = data_reader.user_lockscript().to_entity().as_bytes();
        let x_lock_address = data_reader.x_lock_address().to_entity().raw_data();
        let signer_lockscript = data_reader.signer_lockscript().to_entity().as_bytes();
        let x_unlock_address = data_reader.x_lock_address().to_entity().raw_data();
        let redeemer_lockscript = data_reader.redeemer_lockscript().to_entity().as_bytes();
        let liquidation_trigger_lockscript = data_reader
            .liquidation_trigger_lockscript()
            .to_entity()
            .as_bytes();
        Ok(ToCKBCellDataView {
            status,
            lot_size,
            user_lockscript,
            x_lock_address,
            signer_lockscript,
            x_unlock_address,
            redeemer_lockscript,
            liquidation_trigger_lockscript,
        })
    }

    pub fn get_raw_lot_size(&self) -> u8 {
        self.lot_size
    }

    // Caller should make sure xchain kind is Btc
    pub fn get_btc_lot_size(&self) -> Result<BtcLotSize, Error> {
        BtcLotSize::from_int(self.lot_size).map_err(|_e| Error::LotSizeInvalid)
    }

    // Caller should make sure xchain kind is Eth
    pub fn get_eth_lot_size(&self) -> Result<EthLotSize, Error> {
        EthLotSize::from_int(self.lot_size).map_err(|_e| Error::LotSizeInvalid)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, IntEnum, PartialEq, Eq)]
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
#[derive(Clone, Copy, IntEnum, PartialEq, Eq)]
pub enum BtcLotSize {
    Quarter = 1,
    Half = 2,
    Single = 3,
}

impl BtcLotSize {
    pub fn get_sudt_amount(&self) -> u128 {
        use BtcLotSize::*;
        match self {
            Quarter => BTC_UNIT / 4,
            Half => BTC_UNIT / 2,
            Single => BTC_UNIT,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, IntEnum, PartialEq, Eq)]
pub enum EthLotSize {
    Quarter = 1,
    Half = 2,
    Single = 3,
    Two = 4,
    Three = 5,
    Four = 6,
}

impl EthLotSize {
    pub fn get_sudt_amount(&self) -> u128 {
        use EthLotSize::*;
        match self {
            Quarter => ETH_UNIT / 4,
            Half => ETH_UNIT / 2,
            Single => ETH_UNIT,
            Two => ETH_UNIT * 2,
            Three => ETH_UNIT * 3,
            Four => ETH_UNIT * 4,
        }
    }
}

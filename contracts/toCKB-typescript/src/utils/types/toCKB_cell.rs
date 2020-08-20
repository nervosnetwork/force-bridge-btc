use super::error::Error;
use super::generated::toCKB_cell_data::ToCKBCellDataReader;
use ckb_std::ckb_types::bytes::Bytes;
use core::result::Result;
use int_enum::IntEnum;
use molecule::prelude::*;

pub struct ToCKBCellDataView {
    pub status: ToCKBStatus,
    pub kind: XChainKind,
    lot_size: u8,
    pub user_lockscript_hash: Bytes,
    pub x_lock_address: Bytes,
    pub signer_lockscript_hash: Bytes,
    pub x_unlock_address: Bytes,
    pub redeemer_lockscript_hash: Bytes,
    pub liquidation_trigger_lockscript_hash: Bytes,
}

impl ToCKBCellDataView {
    pub fn from_slice(slice: &[u8]) -> Result<ToCKBCellDataView, Error> {
        ToCKBCellDataReader::verify(slice, false).map_err(|_| Error::Encoding)?;
        let data_reader = ToCKBCellDataReader::new_unchecked(slice);
        let status = ToCKBStatus::from_int(data_reader.status().to_entity().into())?;
        let kind = XChainKind::from_int(data_reader.kind().to_entity().into())?;
        let lot_size = data_reader.lot_size().as_slice()[0];
        let user_lockscript_hash = data_reader.user_lockscript_hash().to_entity().as_bytes();
        let x_lock_address = data_reader.x_lock_address().to_entity().as_bytes();
        let signer_lockscript_hash = data_reader.signer_lockscript_hash().to_entity().as_bytes();
        let x_unlock_address = data_reader.x_lock_address().to_entity().as_bytes();
        let redeemer_lockscript_hash = data_reader
            .redeemer_lockscript_hash()
            .to_entity()
            .as_bytes();
        let liquidation_trigger_lockscript_hash = data_reader
            .liquidation_trigger_lockscript_hash()
            .to_entity()
            .as_bytes();
        Ok(ToCKBCellDataView {
            status,
            kind,
            lot_size,
            user_lockscript_hash,
            x_lock_address,
            signer_lockscript_hash,
            x_unlock_address,
            redeemer_lockscript_hash,
            liquidation_trigger_lockscript_hash,
        })
    }

    pub fn get_btc_lot_size(&self) -> Result<BtcLotSize, Error> {
        if let XChainKind::Btc = self.kind {
            BtcLotSize::from_int(self.lot_size).map_err(|_e| Error::Encoding)
        } else {
            Err(Error::XChainMismatch)
        }
    }

    pub fn get_eth_lot_size(&self) -> Result<EthLotSize, Error> {
        if let XChainKind::Eth = self.kind {
            EthLotSize::from_int(self.lot_size).map_err(|_e| Error::Encoding)
        } else {
            Err(Error::XChainMismatch)
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, IntEnum)]
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
#[derive(Clone, Copy, IntEnum)]
pub enum XChainKind {
    Btc = 1,
    Eth = 2,
}

#[repr(u8)]
#[derive(Clone, Copy, IntEnum)]
pub enum BtcLotSize {
    Quarter = 1,
    Half = 2,
    Single = 3,
}

#[repr(u8)]
#[derive(Clone, Copy, IntEnum)]
pub enum EthLotSize {
    Quarter = 1,
    Half = 2,
    Single = 3,
    Two = 4,
    Three = 5,
    Four = 6,
}

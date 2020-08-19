use super::error::Error;
use super::generated::toCKB_cell_data::ToCKBCellDataReader;
use ckb_std::ckb_types::bytes::Bytes;
use core::result::Result;
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
        let status = ToCKBStatus::from_byte(data_reader.status().to_entity())?;
        let kind = XChainKind::from_byte(data_reader.kind().to_entity())?;
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
            match self.lot_size {
                1 => Ok(BtcLotSize::Quarter),
                2 => Ok(BtcLotSize::Half),
                3 => Ok(BtcLotSize::Single),
                _ => Err(Error::Encoding),
            }
        } else {
            Err(Error::XChainMismatch)
        }
    }

    pub fn get_eth_lot_size(&self) -> Result<EthLotSize, Error> {
        if let XChainKind::Eth = self.kind {
            match self.lot_size {
                1 => Ok(EthLotSize::Quarter),
                2 => Ok(EthLotSize::Half),
                3 => Ok(EthLotSize::Single),
                4 => Ok(EthLotSize::Two),
                5 => Ok(EthLotSize::Three),
                6 => Ok(EthLotSize::Four),
                _ => Err(Error::Encoding),
            }
        } else {
            Err(Error::XChainMismatch)
        }
    }
}

#[repr(u8)]
pub enum ToCKBStatus {
    Initial = 1,
    Bonding,
    Warranty,
    Redeeming,
    LiquidationTimeout,
    LiquidationUndercollateral,
    LiquidationFaultyWhenWarranty,
    LiquidationFaultyWhenRedeeming,
}

impl ToCKBStatus {
    pub fn from_byte(b: Byte) -> Result<ToCKBStatus, Error> {
        let num = b.as_slice()[0];
        use ToCKBStatus::*;
        match num {
            1 => Ok(Initial),
            2 => Ok(Bonding),
            3 => Ok(Warranty),
            4 => Ok(Redeeming),
            5 => Ok(LiquidationTimeout),
            6 => Ok(LiquidationUndercollateral),
            7 => Ok(LiquidationFaultyWhenWarranty),
            8 => Ok(LiquidationFaultyWhenRedeeming),
            _ => Err(Error::Encoding),
        }
    }
}

#[repr(u8)]
pub enum XChainKind {
    Btc = 1,
    Eth,
}

impl XChainKind {
    pub fn from_byte(b: Byte) -> Result<XChainKind, Error> {
        let num = b.as_slice()[0];
        use XChainKind::*;
        match num {
            1 => Ok(Btc),
            2 => Ok(Eth),
            _ => Err(Error::Encoding),
        }
    }
}

#[repr(u8)]
pub enum BtcLotSize {
    Quarter = 1,
    Half,
    Single,
}

#[repr(u8)]
pub enum EthLotSize {
    Quarter = 1,
    Half,
    Single,
    Two,
    Three,
    Four,
}

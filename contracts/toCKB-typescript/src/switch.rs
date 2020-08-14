#![allow(non_snake_case)]

use ckb_std::{
    entry,
    default_alloc,
    debug,
    high_level::{load_script, load_tx_hash},
    error::SysError,
    ckb_types::{bytes::Bytes, prelude::*},
};

use types::{ToCKBCell, ToCKBCellTuple};

use crate::{internal_verify, external_verify};

use crate::Error;

enum TxType {
    DepositRequest,
    Bonding,
    WithdrawPledge,
    WithdrawPledgeAndCollateral,
    MintXT,
    PreTermRedeem,
    AtTermRedeem,
    WithdrawCollateral,
    LiquidationSignerTimeout,
    LiquidationUnderCollateral,
    LiquidationFaultyWhenWarranty,
    LiquidationFaultyWhenRedeeming,
    AuctionSignerTimeout,
    AuctionUnderCollateral,
    AuctionFaultyWhenWarranty,
    AuctionFaultyWhenRedeeming
}

// // TODO toCKBcell after molecule decode
// pub struct ToCKBCell {}
//
// // (input, output)
// pub struct ToCKBCellTuple(ToCKBCell, ToCKBCell);

pub fn verify() -> Result<(), Error> {
    let toCKB_cells = get_toCKB_cells()?;
    let tx_type = get_tx_type(&toCKB_cells)?;
    full_verify(&tx_type, &toCKB_cells)?;
    Ok(())
}

fn get_toCKB_cells() -> Result<ToCKBCellTuple, Error> {
    unimplemented!()
}

fn get_tx_type(toCKB_cells: &ToCKBCellTuple) -> Result<TxType, Error> {
    unimplemented!()
}

fn full_verify(tx_type: &TxType, toCKB_cells: &ToCKBCellTuple) -> Result<(), Error> {
    match tx_type {
        TxType::DepositRequest => {
            internal_verify::deposit_request(toCKB_cells)?;
            external_verify::deposit_request(toCKB_cells)?;
        },
        TxType::Bonding => {},
        TxType::WithdrawPledge => {},
        TxType::WithdrawPledgeAndCollateral => {},
        TxType::MintXT => {},
        TxType::PreTermRedeem => {},
        TxType::AtTermRedeem => {},
        TxType::WithdrawCollateral => {},
        TxType::LiquidationSignerTimeout => {},
        TxType::LiquidationUnderCollateral => {},
        TxType::LiquidationFaultyWhenWarranty => {},
        TxType::LiquidationFaultyWhenRedeeming => {},
        TxType::AuctionSignerTimeout => {},
        TxType::AuctionUnderCollateral => {},
        TxType::AuctionFaultyWhenWarranty => {},
        TxType::AuctionFaultyWhenRedeeming => {},
    }
    Ok(())
}

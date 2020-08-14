#![allow(non_snake_case)]

use core::result::Result;

use crate::Error;
use types::ToCKBCellTuple;

pub fn deposit_request(toCKB_cells: &ToCKBCellTuple) -> Result<(), Error> {
    Ok(())
}

pub fn bonding(toCKB_cells: &ToCKBCellTuple) -> Result<(), Error> {
    Ok(())
}

pub fn withdraw_pledge(toCKB_cells: &ToCKBCellTuple) -> Result<(), Error> {
    Ok(())
}

pub fn withdraw_pledge_collateral(toCKB_cells: &ToCKBCellTuple) -> Result<(), Error> {
    Ok(())
}

pub fn mint_xt(toCKB_cells: &ToCKBCellTuple) -> Result<(), Error> {
    Ok(())
}

pub fn preterm_redeem(toCKB_cells: &ToCKBCellTuple) -> Result<(), Error> {
    Ok(())
}

pub fn atterme_redeem(toCKB_cells: &ToCKBCellTuple) -> Result<(), Error> {
    Ok(())
}

pub fn withdraw_collateral(toCKB_cells: &ToCKBCellTuple) -> Result<(), Error> {
    Ok(())
}

pub fn liquidation_signer_timeout(toCKB_cells: &ToCKBCellTuple) -> Result<(), Error> {
    Ok(())
}

pub fn liquidation_undercollateral(toCKB_cells: &ToCKBCellTuple) -> Result<(), Error> {
    Ok(())
}

pub fn liquidation_faulty_warranty(toCKB_cells: &ToCKBCellTuple) -> Result<(), Error> {
    Ok(())
}

pub fn liquidation_faulty_redeeming(toCKB_cells: &ToCKBCellTuple) -> Result<(), Error> {
    Ok(())
}

pub fn auction_signer_timeout(toCKB_cells: &ToCKBCellTuple) -> Result<(), Error> {
    Ok(())
}

pub fn auction_undercollateral(toCKB_cells: &ToCKBCellTuple) -> Result<(), Error> {
    Ok(())
}

pub fn auction_faulty_warranty(toCKB_cells: &ToCKBCellTuple) -> Result<(), Error> {
    Ok(())
}

pub fn auction_faulty_redeeming(toCKB_cells: &ToCKBCellTuple) -> Result<(), Error> {
    Ok(())
}

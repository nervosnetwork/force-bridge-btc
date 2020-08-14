#![allow(non_snake_case)]

mod deposit_request;
mod bonding;
mod withdraw_pledge;
mod withdraw_pledge_collateral;
mod mint_xt;
mod preterm_redeem;
mod atterm_redeem;
mod withdraw_collateral;
mod liquidation_signertimeout;
mod liquidation_undercollateral;
mod liquidation_faulty_warranty;
mod liquidation_faulty_redeeming;
mod auction_signertimeout;
mod auction_undercollateral;
mod auction_faulty_warranty;
mod auction_faulty_redeeming;

use crate::Error;

// TODO toCKBcell after molecule decode
pub struct ToCKBCell {}

// (input, output)
pub struct ToCKBCellTuple(Option<ToCKBCell>, Option<ToCKBCell>);

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
    AuctionFaultyWhenRedeeming,
}

pub fn verify() -> Result<(), Error> {
    let toCKB_cells = get_toCKB_cells()?;
    let tx_type = get_tx_type(&toCKB_cells)?;
    switch(&tx_type, &toCKB_cells)?;
    Ok(())
}

fn get_toCKB_cells() -> Result<ToCKBCellTuple, Error> {
    unimplemented!()
}

fn get_tx_type(toCKB_cells: &ToCKBCellTuple) -> Result<TxType, Error> {
    unimplemented!()
}

fn switch(tx_type: &TxType, toCKB_cells: &ToCKBCellTuple) -> Result<(), Error> {
    use TxType::*;
    match tx_type {
        DepositRequest => { deposit_request::verify(toCKB_cells)?; }
        Bonding => { bonding::verify(toCKB_cells)?; }
        WithdrawPledge => { withdraw_pledge::verify(toCKB_cells)?; }
        WithdrawPledgeAndCollateral => { withdraw_pledge_collateral::verify(toCKB_cells)?; }
        MintXT => { mint_xt::verify(toCKB_cells)?; }
        PreTermRedeem => { preterm_redeem::verify(toCKB_cells)?; }
        AtTermRedeem => { atterm_redeem::verify(toCKB_cells)?; }
        WithdrawCollateral => { withdraw_collateral::verify(toCKB_cells)?; }
        LiquidationSignerTimeout => { liquidation_signertimeout::verify(toCKB_cells)?; }
        LiquidationUnderCollateral => { liquidation_undercollateral::verify(toCKB_cells)?; }
        LiquidationFaultyWhenWarranty => { liquidation_faulty_warranty::verify(toCKB_cells)?; }
        LiquidationFaultyWhenRedeeming => { liquidation_faulty_redeeming::verify(toCKB_cells)?; }
        AuctionSignerTimeout => { auction_signertimeout::verify(toCKB_cells)?; }
        AuctionUnderCollateral => { auction_undercollateral::verify(toCKB_cells)?; }
        AuctionFaultyWhenWarranty => { auction_faulty_warranty::verify(toCKB_cells)?; }
        AuctionFaultyWhenRedeeming => { auction_faulty_redeeming::verify(toCKB_cells)?; }
    }
    Ok(())
}

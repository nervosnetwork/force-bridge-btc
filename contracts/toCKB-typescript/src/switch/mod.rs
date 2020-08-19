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

use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell_data, QueryIter},
};
use alloc::vec::Vec;
use crate::utils::types::{ToCKBCellDataView, ToCKBStatus, Error};

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

pub struct ToCKBCellDataTuple(Option<ToCKBCellDataView>, Option<ToCKBCellDataView>);

pub fn verify() -> Result<(), Error> {
    let toCKB_data_tuple = get_toCKB_data_tuple()?;
    let tx_type = get_tx_type(&toCKB_data_tuple)?;
    switch(&tx_type, &toCKB_data_tuple)?;
    Ok(())
}

fn get_toCKB_data_tuple() -> Result<ToCKBCellDataTuple, Error> {
    let input_toCKB_data = get_toCKB_data(Source::GroupInput)?;
    let output_toCKB_data = get_toCKB_data(Source::GroupOutput)?;
    let tuple = ToCKBCellDataTuple(input_toCKB_data, output_toCKB_data);
    Ok(tuple)
}

fn get_toCKB_data(source: Source) -> Result<Option<ToCKBCellDataView>, Error> {
    let toCKB_data_list = QueryIter::new(load_cell_data, source).collect::<Vec<Vec<u8>>>();
    match toCKB_data_list.len() {
        0 => Ok(None),
        1 => Ok(Some(ToCKBCellDataView::from_slice(toCKB_data_list[0].as_slice())?)),
        _ => Err(Error::TxInvalid)
    }
}

fn get_tx_type(data_tuple: &ToCKBCellDataTuple) -> Result<TxType, Error> {
    match data_tuple {
        ToCKBCellDataTuple(None, Some(output_data)) => get_generation_tx_type(output_data),
        ToCKBCellDataTuple(Some(input_data), Some(output_data)) => get_transformation_tx_type(input_data, output_data),
        ToCKBCellDataTuple(Some(input_data), None) => get_deletion_tx_type(input_data),
        _ => Err(Error::TxInvalid)
    }
}

fn get_generation_tx_type(data: &ToCKBCellDataView) -> Result<TxType, Error> {
    if let ToCKBStatus::Initial = data.status {
        Ok(TxType::DepositRequest)
    } else {
        Err(Error::TxInvalid)
    }
}

fn get_transformation_tx_type(_input_data: &ToCKBCellDataView, _output_data: &ToCKBCellDataView) -> Result<TxType, Error> {
    unimplemented!()
}

fn get_deletion_tx_type(_data: &ToCKBCellDataView) -> Result<TxType, Error> {
    unimplemented!()
}

fn switch(tx_type: &TxType, toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    use TxType::*;
    match tx_type {
        DepositRequest => { deposit_request::verify(toCKB_data_tuple)?; }
        Bonding => { bonding::verify(toCKB_data_tuple)?; }
        WithdrawPledge => { withdraw_pledge::verify(toCKB_data_tuple)?; }
        WithdrawPledgeAndCollateral => { withdraw_pledge_collateral::verify(toCKB_data_tuple)?; }
        MintXT => { mint_xt::verify(toCKB_data_tuple)?; }
        PreTermRedeem => { preterm_redeem::verify(toCKB_data_tuple)?; }
        AtTermRedeem => { atterm_redeem::verify(toCKB_data_tuple)?; }
        WithdrawCollateral => { withdraw_collateral::verify(toCKB_data_tuple)?; }
        LiquidationSignerTimeout => { liquidation_signertimeout::verify(toCKB_data_tuple)?; }
        LiquidationUnderCollateral => { liquidation_undercollateral::verify(toCKB_data_tuple)?; }
        LiquidationFaultyWhenWarranty => { liquidation_faulty_warranty::verify(toCKB_data_tuple)?; }
        LiquidationFaultyWhenRedeeming => { liquidation_faulty_redeeming::verify(toCKB_data_tuple)?; }
        AuctionSignerTimeout => { auction_signertimeout::verify(toCKB_data_tuple)?; }
        AuctionUnderCollateral => { auction_undercollateral::verify(toCKB_data_tuple)?; }
        AuctionFaultyWhenWarranty => { auction_faulty_warranty::verify(toCKB_data_tuple)?; }
        AuctionFaultyWhenRedeeming => { auction_faulty_redeeming::verify(toCKB_data_tuple)?; }
    }
    Ok(())
}

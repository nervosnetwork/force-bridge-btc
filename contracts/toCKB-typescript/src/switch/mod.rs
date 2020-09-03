mod atterm_redeem;
mod auction_faulty_redeeming;
mod auction_faulty_warranty;
mod auction_signertimeout;
mod auction_undercollateral;
mod bonding;
mod deposit_request;
mod liquidation_faulty_redeeming;
mod liquidation_faulty_warranty;
mod liquidation_signertimeout;
mod liquidation_undercollateral;
mod mint_xt;
mod pre_undercollateral_redeem;
mod preterm_redeem;
mod withdraw_collateral;
mod withdraw_pledge;
mod withdraw_pledge_collateral;

use crate::utils::{
    config::SUDT_CODE_HASH,
    types::{Error, ToCKBCellDataView, ToCKBStatus},
};
use alloc::vec::Vec;
use ckb_std::{
    ckb_constants::Source,
    debug,
    high_level::{load_cell_data, load_cell_type, load_input_since, QueryIter},
};

#[derive(Debug)]
enum TxType {
    DepositRequest,
    Bonding,
    WithdrawPledge,
    WithdrawPledgeAndCollateral,
    MintXT,
    PretermRedeem,
    AttermRedeem,
    PreUndercollateralRedeem,
    WithdrawCollateral,
    LiquidationSignerTimeout,
    LiquidationUndercollateral,
    LiquidationFaultyWhenWarranty,
    LiquidationFaultyWhenRedeeming,
    AuctionSignerTimeout,
    AuctionUnderCollateral,
    AuctionFaultyWhenWarranty,
    AuctionFaultyWhenRedeeming,
}

#[derive(Debug)]
pub struct ToCKBCellDataTuple(Option<ToCKBCellDataView>, Option<ToCKBCellDataView>);

pub fn verify() -> Result<(), Error> {
    let toCKB_data_tuple = get_toCKB_data_tuple()?;
    debug!("toCKB_data_tuple: {:?}", toCKB_data_tuple);
    let tx_type = get_tx_type(&toCKB_data_tuple)?;
    verify_xt(&tx_type)?;
    debug!("tx_type: {:?}", tx_type);
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
        1 => Ok(Some(ToCKBCellDataView::from_slice(
            toCKB_data_list[0].as_slice(),
        )?)),
        _ => Err(Error::TxInvalid),
    }
}

fn get_tx_type(data_tuple: &ToCKBCellDataTuple) -> Result<TxType, Error> {
    match data_tuple {
        ToCKBCellDataTuple(None, Some(output_data)) => get_generation_tx_type(output_data),
        ToCKBCellDataTuple(Some(input_data), Some(output_data)) => {
            get_transformation_tx_type(input_data, output_data)
        }
        ToCKBCellDataTuple(Some(input_data), None) => get_deletion_tx_type(input_data),
        _ => Err(Error::TxInvalid),
    }
}

fn get_generation_tx_type(data: &ToCKBCellDataView) -> Result<TxType, Error> {
    if let ToCKBStatus::Initial = data.status {
        Ok(TxType::DepositRequest)
    } else {
        Err(Error::TxInvalid)
    }
}

fn get_transformation_tx_type(
    input_data: &ToCKBCellDataView,
    output_data: &ToCKBCellDataView,
) -> Result<TxType, Error> {
    use ToCKBStatus::*;
    use TxType::*;
    let status_transformation = (input_data.status, output_data.status);
    match status_transformation {
        (Initial, Bonded) => Ok(Bonding),
        (Bonded, Warranty) => Ok(MintXT),
        (Warranty, Redeeming) => {
            if let 0 = load_input_since(0, Source::GroupInput)? {
                Ok(PretermRedeem)
            } else {
                Ok(AttermRedeem)
            }
        }
        (Redeeming, SignerTimeout) => Ok(LiquidationSignerTimeout),
        (Warranty, Undercollateral) => Ok(LiquidationUndercollateral),
        (Warranty, FaultyWhenWarranty) => Ok(LiquidationFaultyWhenWarranty),
        (Redeeming, FaultyWhenRedeeming) => Ok(LiquidationFaultyWhenRedeeming),
        _ => Err(Error::TxInvalid),
    }
}

fn get_deletion_tx_type(data: &ToCKBCellDataView) -> Result<TxType, Error> {
    use ToCKBStatus::*;
    use TxType::*;
    match data.status {
        Initial => Ok(WithdrawPledge),
        Bonded => Ok(WithdrawPledgeAndCollateral),
        Warranty => Ok(PreUndercollateralRedeem),
        Redeeming => Ok(WithdrawCollateral),
        SignerTimeout => Ok(AuctionSignerTimeout),
        Undercollateral => Ok(AuctionUnderCollateral),
        FaultyWhenWarranty => Ok(AuctionFaultyWhenWarranty),
        FaultyWhenRedeeming => Ok(AuctionFaultyWhenRedeeming),
    }
}

fn verify_xt(tx_type: &TxType) -> Result<(), Error> {
    use TxType::*;
    match tx_type {
        MintXT
        | PretermRedeem
        | AttermRedeem
        | PreUndercollateralRedeem
        | AuctionSignerTimeout
        | AuctionUnderCollateral
        | AuctionFaultyWhenWarranty
        | AuctionFaultyWhenRedeeming => Ok(()),
        _ => forbid_mint_xt(),
    }
}

fn forbid_mint_xt() -> Result<(), Error> {
    let sudt_cell_count = QueryIter::new(load_cell_type, Source::Output)
        .filter(|type_opt| type_opt.is_some())
        .map(|type_opt| type_opt.unwrap())
        .filter(|script| script.code_hash().raw_data().as_ref() == SUDT_CODE_HASH.as_ref())
        .count();
    if 0 != sudt_cell_count {
        return Err(Error::TxInvalid);
    }
    Ok(())
}

fn switch(tx_type: &TxType, toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    use TxType::*;
    match tx_type {
        DepositRequest => {
            deposit_request::verify(toCKB_data_tuple)?;
        }
        Bonding => {
            bonding::verify(toCKB_data_tuple)?;
        }
        WithdrawPledge => {
            withdraw_pledge::verify(toCKB_data_tuple)?;
        }
        WithdrawPledgeAndCollateral => {
            withdraw_pledge_collateral::verify(toCKB_data_tuple)?;
        }
        MintXT => {
            mint_xt::verify(toCKB_data_tuple)?;
        }
        PretermRedeem => {
            preterm_redeem::verify(toCKB_data_tuple)?;
        }
        AttermRedeem => {
            atterm_redeem::verify(toCKB_data_tuple)?;
        }
        PreUndercollateralRedeem => {
            pre_undercollateral_redeem::verify(toCKB_data_tuple)?;
        }
        WithdrawCollateral => {
            withdraw_collateral::verify(toCKB_data_tuple)?;
        }
        LiquidationSignerTimeout => {
            liquidation_signertimeout::verify(toCKB_data_tuple)?;
        }
        LiquidationUndercollateral => {
            liquidation_undercollateral::verify(toCKB_data_tuple)?;
        }
        LiquidationFaultyWhenWarranty => {
            liquidation_faulty_warranty::verify(toCKB_data_tuple)?;
        }
        LiquidationFaultyWhenRedeeming => {
            liquidation_faulty_redeeming::verify(toCKB_data_tuple)?;
        }
        AuctionSignerTimeout => {
            auction_signertimeout::verify(toCKB_data_tuple)?;
        }
        AuctionUnderCollateral => {
            auction_undercollateral::verify(toCKB_data_tuple)?;
        }
        AuctionFaultyWhenWarranty => {
            auction_faulty_warranty::verify(toCKB_data_tuple)?;
        }
        AuctionFaultyWhenRedeeming => {
            auction_faulty_redeeming::verify(toCKB_data_tuple)?;
        }
    }
    Ok(())
}

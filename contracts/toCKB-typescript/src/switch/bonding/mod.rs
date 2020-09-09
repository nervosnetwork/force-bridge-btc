use crate::switch::ToCKBCellDataTuple;
use crate::utils::config::*;
use crate::utils::tools::{get_price, XChainKind};
use crate::utils::types::{Error, ToCKBCellDataView};
use bech32::{self, FromBase32};
use ckb_std::ckb_constants::Source;
use ckb_std::debug;
use ckb_std::high_level::load_cell_capacity;
use core::result::Result;
use molecule::prelude::Vec;

fn verify_data(
    input_toCKB_data: &ToCKBCellDataView,
    out_toCKB_data: &ToCKBCellDataView,
) -> Result<u128, Error> {
    let amount: u128 = match input_toCKB_data.get_xchain_kind() {
        XChainKind::Btc => {
            let (hrp, data) = bech32::decode(
                core::str::from_utf8(out_toCKB_data.x_lock_address.as_ref()).map_err(|_|Error::XChainAddressInvalid)?,
            )
            .map_err(|_| Error::XChainAddressInvalid)?;
            if hrp != "bc" {
                return Err(Error::XChainAddressInvalid);
            }
            let raw_data = Vec::<u8>::from_base32(&data).unwrap();
            if raw_data.len() != 22 {
                return Err(Error::XChainAddressInvalid);
            }
            if &raw_data[..2] != &[0x00, 0x14] {
                return Err(Error::XChainAddressInvalid);
            }
            let btc_lot_size = out_toCKB_data.get_btc_lot_size()?;
            btc_lot_size.get_sudt_amount()
        }
        XChainKind::Eth => {
            if out_toCKB_data.x_lock_address.as_ref().len() != 20 {
                return Err(Error::XChainAddressInvalid);
            }
            let eth_lot_size = out_toCKB_data.get_eth_lot_size()?;
            eth_lot_size.get_sudt_amount()
        }
    };
    if input_toCKB_data.user_lockscript.as_ref() != out_toCKB_data.user_lockscript.as_ref()
        || input_toCKB_data.get_raw_lot_size() != out_toCKB_data.get_raw_lot_size()
        || input_toCKB_data.x_extra != out_toCKB_data.x_extra
    {
        return Err(Error::InvariantDataMutated);
    }
    Ok(amount)
}

fn verify_collateral(lot_amount: u128) -> Result<(), Error> {
    let input_capacity = load_cell_capacity(0, Source::GroupInput)?;
    let output_capacity = load_cell_capacity(0, Source::GroupOutput)?;
    debug!(
        "output_capacity {:?}, input_capacity {:?}, lot_amount {:?}",
        output_capacity, input_capacity, lot_amount
    );
    if input_capacity > output_capacity {
        return Err(Error::CollateralInvalid);
    }

    let diff_capacity = output_capacity - input_capacity;

    let price = get_price()?;

    let collateral: u128 = lot_amount * (COLLATERAL_PERCENT as u128)
        + (2 * XT_CELL_CAPACITY * 100 / CKB_UNITS) as u128 * price;
    let diff: u128 = (diff_capacity * 100 / CKB_UNITS) as u128 * price;
    if collateral != diff {
        return Err(Error::CollateralInvalid);
    }
    Ok(())
}

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    debug!("start bonding");

    let input_toCKB_data = toCKB_data_tuple
        .0
        .as_ref()
        .expect("inputs contain toCKB cell");
    let output_toCKB_data = toCKB_data_tuple
        .1
        .as_ref()
        .expect("outputs contain toCKB cell");
    let amount = verify_data(input_toCKB_data, output_toCKB_data)?;
    debug!("amount {:?}", amount);

    verify_collateral(amount)
}

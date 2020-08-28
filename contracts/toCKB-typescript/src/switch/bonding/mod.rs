use crate::switch::ToCKBCellDataTuple;
use crate::utils::config::COLLATERAL_PERCENT;
use crate::utils::tools::{get_xchain_kind, XChainKind};
use crate::utils::types::{Error, ToCKBCellDataView};
use bech32::{self, FromBase32};
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::{bytes::Bytes, prelude::*};
use ckb_std::high_level::{load_cell_capacity, load_witness_args};
use core::result::Result;
use molecule::prelude::Vec;

pub fn verify_data(
    input_toCKB_data: &ToCKBCellDataView,
    out_toCKB_data: &ToCKBCellDataView,
) -> Result<u128, Error> {
    let coin_kind = get_xchain_kind()?;
    let amount: u128 = match coin_kind {
        XChainKind::Btc => {
            if out_toCKB_data.get_btc_lot_size()? != input_toCKB_data.get_btc_lot_size()? {
                return Err(Error::InvariantDataMutated);
            }
            if bech32::decode(core::str::from_utf8(out_toCKB_data.x_lock_address.as_ref()).unwrap())
                .is_err()
            {
                return Err(Error::XChainAddressInvalid);
            }
            let (hrp, data) = bech32::decode(
                core::str::from_utf8(out_toCKB_data.x_lock_address.as_ref()).unwrap(),
            )
            .unwrap();
            if hrp != "bc" {
                return Err(Error::XChainAddressInvalid);
            }
            let raw_data = Vec::<u8>::from_base32(&data).unwrap();
            if &raw_data[..2] != &[0x00, 0x14] {
                return Err(Error::XChainAddressInvalid);
            }
            if raw_data.len() != 22 {
                return Err(Error::XChainAddressInvalid);
            }
            let btc_lot_size = out_toCKB_data.get_btc_lot_size()?;
            btc_lot_size.get_sudt_amount()
        }
        XChainKind::Eth => {
            if out_toCKB_data.get_eth_lot_size()? != input_toCKB_data.get_eth_lot_size()? {
                return Err(Error::InvariantDataMutated);
            }
            if out_toCKB_data.x_lock_address.as_ref().len() != 20 {
                return Err(Error::XChainAddressInvalid);
            }
            let eth_lot_size = out_toCKB_data.get_eth_lot_size()?;
            eth_lot_size.get_sudt_amount()
        }
    };
    if input_toCKB_data.user_lockscript.as_ref() != out_toCKB_data.user_lockscript.as_ref() {
        return Err(Error::InvariantDataMutated);
    }
    Ok(amount)
}

pub fn verify_collateral(lot_amount: u128) -> Result<(), Error> {
    let witness_args = load_witness_args(0, Source::GroupInput)?.input_type();
    if witness_args.is_none() {
        return Err(Error::InvalidWitness);
    }
    let witness_bytes: Bytes = witness_args.to_opt().unwrap().unpack();
    let price: u8 = witness_bytes[0];

    let input_capacity = load_cell_capacity(0, Source::GroupInput)?;
    let output_capacity = load_cell_capacity(0, Source::GroupOutput)?;
    if input_capacity > output_capacity {
        return Err(Error::CollateralInvalid);
    }
    let diff_capacity = output_capacity - input_capacity;
    let collateral: u128 = lot_amount * (COLLATERAL_PERCENT as u128) * (price as u128);

    if collateral != diff_capacity as u128 * 100 as u128 {
        return Err(Error::CollateralInvalid);
    }
    Ok(())
}

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    let input_toCKB_data = toCKB_data_tuple
        .0
        .as_ref()
        .expect("inputs contain toCKB cell");
    let output_toCKB_data = toCKB_data_tuple
        .1
        .as_ref()
        .expect("outputs contain toCKB cell");
    let amount = verify_data(input_toCKB_data, output_toCKB_data)?;

    verify_collateral(amount)
}

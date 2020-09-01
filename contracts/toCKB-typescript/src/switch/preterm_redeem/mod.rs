use crate::switch::ToCKBCellDataTuple;
use crate::utils::config::{SIGNER_FEE_RATE, SUDT_CODE_HASH};
use crate::utils::tools::{get_xchain_kind, XChainKind};
use crate::utils::types::{Error, ToCKBCellDataView};
use bech32::{self, FromBase32};
use ckb_std::ckb_constants::Source;
use ckb_std::error::SysError;
use ckb_std::high_level::{load_cell_data, load_cell_lock, load_cell_type};
use core::result::Result;
use molecule::prelude::*;

pub fn verify_data(
    input_toCKB_data: &ToCKBCellDataView,
    out_toCKB_data: &ToCKBCellDataView,
) -> Result<u128, Error> {
    let kind = get_xchain_kind()?;
    let lot_size = match kind {
        XChainKind::Btc => {
            if out_toCKB_data.get_btc_lot_size()? != input_toCKB_data.get_btc_lot_size()? {
                return Err(Error::InvariantDataMutated);
            }
            let (hrp, data) = bech32::decode(
                core::str::from_utf8(out_toCKB_data.x_unlock_address.as_ref()).unwrap(),
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

            out_toCKB_data.get_btc_lot_size()?.get_sudt_amount()
        }
        XChainKind::Eth => {
            if out_toCKB_data.get_eth_lot_size()? != input_toCKB_data.get_eth_lot_size()? {
                return Err(Error::InvariantDataMutated);
            }
            if out_toCKB_data.x_unlock_address.as_ref().len() != 20 {
                return Err(Error::XChainAddressInvalid);
            }
            out_toCKB_data.get_eth_lot_size()?.get_sudt_amount()
        }
    };
    if input_toCKB_data.user_lockscript.as_ref() != out_toCKB_data.user_lockscript.as_ref()
        || input_toCKB_data.x_lock_address.as_ref() != out_toCKB_data.x_lock_address.as_ref()
        || input_toCKB_data.signer_lockscript.as_ref() != out_toCKB_data.signer_lockscript.as_ref()
    {
        return Err(Error::InvariantDataMutated);
    }
    Ok(lot_size)
}

pub fn verify_burn(lot_size: u128, out_toCKB_data: &ToCKBCellDataView) -> Result<(), Error> {
    let mut deposit_requestor = false;
    let mut input_sudt_sum: u128 = 0;
    let mut output_sudt_sum: u128 = 0;
    let mut input_index = 0;
    loop {
        let cell_type = load_cell_type(input_index, Source::Input);
        match cell_type {
            Err(SysError::IndexOutOfBound) => break,
            Err(_err) => panic!("iter input return an error"),
            Ok(cell_type) => {
                if !(cell_type.is_some()
                    && cell_type.unwrap().code_hash().raw_data().as_ref()
                        == SUDT_CODE_HASH.as_ref())
                {
                    input_index += 1;
                    continue;
                }

                let lock = load_cell_lock(input_index, Source::Input)?;

                if lock.as_slice() == out_toCKB_data.redeemer_lockscript.as_ref() {
                    deposit_requestor = true;
                }

                let data = load_cell_data(input_index, Source::Input)?;
                let mut buf = [0u8; 16];
                if data.len() == 16 {
                    buf.copy_from_slice(&data);
                    input_sudt_sum += u128::from_le_bytes(buf)
                }
                input_index += 1;
            }
        }
    }

    let mut output_index = 0;
    loop {
        let cell_type = load_cell_type(output_index, Source::Output);

        match cell_type {
            Err(SysError::IndexOutOfBound) => break,
            Err(_err) => panic!("iter output return an error"),
            Ok(cell_type) => {
                if !(cell_type.is_some()
                    && cell_type.unwrap().code_hash().raw_data().as_ref()
                        == SUDT_CODE_HASH.as_ref())
                {
                    output_index += 1;
                    continue;
                }
                let data = load_cell_data(output_index, Source::Output)?;
                let mut buf = [0u8; 16];
                if data.len() == 16 {
                    buf.copy_from_slice(&data);
                    output_sudt_sum += u128::from_le_bytes(buf)
                }
                output_index += 1;
            }
        }
    }

    if deposit_requestor && output_sudt_sum != 0 {
        return Err(Error::XTBurnInvalid);
    }
    if !deposit_requestor {
        let signer_fee: u128 = lot_size * SIGNER_FEE_RATE.0 / SIGNER_FEE_RATE.1;
        if (input_sudt_sum != lot_size + signer_fee) || (output_sudt_sum != signer_fee) {
            return Err(Error::XTBurnInvalid);
        }
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
    let lot_size = verify_data(input_toCKB_data, output_toCKB_data)?;
    verify_burn(lot_size, output_toCKB_data)
}

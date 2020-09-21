use crate::switch::ToCKBCellDataTuple;
use crate::utils::config::SINCE_AT_TERM_REDEEM;
use crate::utils::tools::{check_capacity, is_XT_typescript, verify_btc_address, XChainKind};
use crate::utils::types::{Error, ToCKBCellDataView};
use ckb_std::ckb_constants::Source;
use ckb_std::error::SysError;
use ckb_std::high_level::{load_cell_data, load_cell_lock_hash, load_cell_type, load_input_since};
use core::result::Result;

fn verify_data(
    input_toCKB_data: &ToCKBCellDataView,
    out_toCKB_data: &ToCKBCellDataView,
) -> Result<u128, Error> {
    let lot_size = match input_toCKB_data.get_xchain_kind() {
        XChainKind::Btc => {
            if out_toCKB_data.get_btc_lot_size()? != input_toCKB_data.get_btc_lot_size()? {
                return Err(Error::InvariantDataMutated);
            }
            verify_btc_address(out_toCKB_data.x_unlock_address.as_ref())?;
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
    if input_toCKB_data.user_lockscript != out_toCKB_data.user_lockscript
        || input_toCKB_data.x_lock_address != out_toCKB_data.x_lock_address
        || input_toCKB_data.signer_lockscript != out_toCKB_data.signer_lockscript
        || input_toCKB_data.x_extra != out_toCKB_data.x_extra
    {
        return Err(Error::InvariantDataMutated);
    }
    Ok(lot_size)
}

fn verify_since() -> Result<(), Error> {
    let since = load_input_since(0, Source::GroupInput)?;
    if since != SINCE_AT_TERM_REDEEM {
        return Err(Error::InputSinceInvalid);
    }
    Ok(())
}

fn verify_burn(lot_size: u128) -> Result<(), Error> {
    let lock_hash = load_cell_lock_hash(0, Source::GroupInput)?;

    let mut input_sudt_sum: u128 = 0;
    let mut input_index = 0;
    loop {
        let cell_type = load_cell_type(input_index, Source::Input);
        match cell_type {
            Err(SysError::IndexOutOfBound) => break,
            Err(_err) => panic!("iter input return an error"),
            Ok(cell_type) => {
                if !(cell_type.is_some()
                    && is_XT_typescript(&cell_type.unwrap(), lock_hash.as_ref()))
                {
                    input_index += 1;
                    continue;
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

    let mut output_sudt_num = 0;
    let mut output_index = 0;
    loop {
        let cell_type = load_cell_type(output_index, Source::Output);
        match cell_type {
            Err(SysError::IndexOutOfBound) => break,
            Err(_err) => panic!("iter output return an error"),
            Ok(cell_type) => {
                if !(cell_type.is_some()
                    && is_XT_typescript(&cell_type.unwrap(), lock_hash.as_ref()))
                {
                    output_index += 1;
                    continue;
                }

                let data = load_cell_data(output_index, Source::Output)?;
                let mut buf = [0u8; 16];
                if data.len() == 16 {
                    buf.copy_from_slice(&data);
                    output_sudt_num += u128::from_le_bytes(buf)
                }
                output_index += 1;
            }
        }
    }

    if input_sudt_sum - output_sudt_num != lot_size {
        return Err(Error::XTBurnInvalid);
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
    check_capacity()?;
    let lot_size = verify_data(input_toCKB_data, output_toCKB_data)?;
    verify_burn(lot_size)?;
    verify_since()
}

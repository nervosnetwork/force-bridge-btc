use crate::switch::ToCKBCellDataTuple;
use crate::utils::tools::{get_xchain_kind, XChainKind};
use crate::utils::types::{Error, ToCKBCellDataView};
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::{bytes::Bytes, prelude::*};
use ckb_std::high_level::{load_cell_capacity, load_cell_data, load_witness_args};
use core::result::Result;

const UDT_LEN: usize = 16;

pub fn verify_data(
    input_toCKB_data: &ToCKBCellDataView,
    out_toCKB_data: &ToCKBCellDataView,
) -> Result<(), Error> {
    let kind = get_xchain_kind()?;
    match kind {
        XChainKind::Btc => {
            if out_toCKB_data.get_btc_lot_size()? != input_toCKB_data.get_btc_lot_size()? {
                return Err(Error::InvariantDataMutated);
            }
            if bech32::decode(core::str::from_utf8(out_toCKB_data.x_unlock_address.as_ref()).unwrap())
                .is_err()
            {
                return Err(Error::XChainAddressInvalid);
            }
        }
        XChainKind::Eth => {
            if out_toCKB_data.get_eth_lot_size()? != input_toCKB_data.get_eth_lot_size()? {
                return Err(Error::InvariantDataMutated);
            }
            if out_toCKB_data.x_unlock_address.as_ref().len() != 20 {
                return Err(Error::XChainAddressInvalid);
            }
        }
    };
    if input_toCKB_data.user_lockscript.as_ref() != out_toCKB_data.user_lockscript.as_ref()
        || input_toCKB_data.x_lock_address.as_ref() != out_toCKB_data.x_lock_address.as_ref()
        || input_toCKB_data.signer_lockscript.as_ref() != out_toCKB_data.signer_lockscript.as_ref()
    {
        return Err(Error::InvariantDataMutated);
    }
    Ok(())
}

pub fn verify_burn() -> Result<(), Error> {
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
    verify_data(input_toCKB_data, output_toCKB_data)?;
    verify_burn()
}

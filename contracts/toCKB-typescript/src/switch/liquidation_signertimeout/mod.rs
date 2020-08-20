use crate::switch::ToCKBCellDataTuple;
use crate::utils::types::{
    Error, XChainKind
};
use core::result::Result;
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell_capacity}
};
use ckb_std::high_level::load_input_since;

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    if toCKB_data_tuple.0.is_none() {
        return Err(Error::InputDataInvalid)
    }
    if toCKB_data_tuple.1.is_none() {
        return Err(Error::OutputDataInvalid)
    }
    let data = toCKB_data_tuple;
    let input_data = data.0.as_ref().unwrap();
    let output_data = data.1.as_ref().unwrap();

    // `get_toCKB_data_tuple` has checked cell nums == 1 in both inputs and outputs
    // Todo: check since n4
    let _since = load_input_since(0, Source::GroupInput);

    // check if capacity and data[2..7] are not changed
    let cap_input = load_cell_capacity(0, Source::GroupInput)
        .map_err(|_| Error::CapacityInvalid)?;
    let cap_output = load_cell_capacity(0, Source::GroupOutput)
        .map_err(|_| Error::CapacityInvalid)?;
    if cap_input != cap_output {
        return Err(Error::CapacityInvalid);
    }

    if input_data.kind != output_data.kind {
        return Err(Error::KindInvalid)
    }

    match input_data.kind {
        XChainKind::Btc => {
            if input_data.get_btc_lot_size()? != output_data.get_btc_lot_size()? {
                return Err(Error::BTCLotSizeInvalid)
            }
        }
        XChainKind::Eth => {
            if input_data.get_eth_lot_size()? != output_data.get_eth_lot_size()? {
                return Err(Error::ETHLotSizeInvalid)
            }
        }
    };
    
    if input_data.user_lockscript_hash.to_vec() != output_data.user_lockscript_hash.to_vec() {
        return Err(Error::UserLockInvalid);
    }

    if input_data.x_lock_address.to_vec() != output_data.x_lock_address.to_vec() {
        return Err(Error::XLockAddressInvalid);
    }

    if input_data.signer_lockscript_hash.to_vec() != output_data.signer_lockscript_hash.to_vec() {
        return Err(Error::SignerLockInvalid);
    }

    if input_data.x_unlock_address.to_vec() != output_data.x_unlock_address.to_vec() {
        return Err(Error::XUnlockAddressInvalid);
    }

    if input_data.redeemer_lockscript_hash.to_vec() != output_data.redeemer_lockscript_hash.to_vec() {
        return Err(Error::RedeemerLockInvalid);
    }
    Ok(())
}

use crate::switch::ToCKBCellDataTuple;
use crate::utils::types::{
    Error, XChainKind,
};
use core::result::Result;
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell_capacity, load_input_since},
};

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    let input_data = toCKB_data_tuple.0.as_ref().expect("inputs should contain toCKB cell");
    let output_data = toCKB_data_tuple.1.as_ref().expect("outputs should contain toCKB cell");

    // `get_toCKB_data_tuple` has checked cell nums == 1 in both inputs and outputs
    // Todo: check since n4
    let since = load_input_since(0, Source::GroupInput).expect("since should exist");
    verify_since(since)?;

    // check if capacity and data[2..7] are not changed
    let cap_input = load_cell_capacity(0, Source::GroupInput).expect("get input capacity");
    let cap_output = load_cell_capacity(0, Source::GroupOutput).expect("get output capacity");
    if cap_input != cap_output {
        return Err(Error::CapacityInvalid);
    }

    if input_data.kind != output_data.kind
        || input_data.user_lockscript_hash.as_ref() != output_data.user_lockscript_hash.as_ref()
        || input_data.x_lock_address.as_ref() != output_data.x_lock_address.as_ref()
        || input_data.signer_lockscript_hash.as_ref() != output_data.signer_lockscript_hash.as_ref()
        || input_data.x_unlock_address.as_ref() != output_data.x_unlock_address.as_ref()
        || input_data.redeemer_lockscript_hash.as_ref() != output_data.redeemer_lockscript_hash.as_ref()
    {
        return Err(Error::InvariantDataMutated);
    }

    match input_data.kind {
        XChainKind::Btc => {
            if input_data.get_btc_lot_size()? != output_data.get_btc_lot_size()? {
                return Err(Error::InvariantDataMutated);
            }
        }
        XChainKind::Eth => {
            if input_data.get_eth_lot_size()? != output_data.get_eth_lot_size()? {
                return Err(Error::InvariantDataMutated);
            }
        }
    };

    Ok(())
}

fn verify_since(_since: u64) -> Result<(), Error> {
    Ok(())
}
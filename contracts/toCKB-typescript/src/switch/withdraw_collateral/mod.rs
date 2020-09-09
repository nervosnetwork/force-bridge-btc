use crate::switch::ToCKBCellDataTuple;
use crate::utils::{
    tools::{verify_btc_witness, XChainKind},
    types::{mint_xt_witness::MintXTWitnessReader, Error, ToCKBCellDataView},
};

use ckb_std::ckb_types::prelude::*;
use ckb_std::{
    ckb_constants::Source,
    debug,
    high_level::{load_cell, load_cell_capacity, load_witness_args, QueryIter},
};
use core::result::Result;
use molecule::prelude::*;

/// ensure transfer happen on XChain by verifying the spv proof
fn verify_witness(data: &ToCKBCellDataView) -> Result<(), Error> {
    let witness_args = load_witness_args(0, Source::GroupInput)?.input_type();
    debug!("witness_args: {:?}", &witness_args);
    if witness_args.is_none() {
        return Err(Error::InvalidWitness);
    }
    let witness_args = witness_args.to_opt().unwrap().raw_data();
    debug!("witness_args parsed: {:?}", &witness_args);
    if MintXTWitnessReader::verify(&witness_args, false).is_err() {
        return Err(Error::InvalidWitness);
    }
    let witness = MintXTWitnessReader::new_unchecked(&witness_args);
    debug!("witness: {:?}", witness);
    let proof = witness.spv_proof().raw_data();
    let cell_dep_index_list = witness.cell_dep_index_list().raw_data();
    match data.get_xchain_kind() {
        XChainKind::Btc => {
            let _btc_extra = verify_btc_witness(
                data,
                proof,
                cell_dep_index_list,
                data.x_unlock_address.as_ref(),
                data.get_btc_lot_size()?.get_sudt_amount(),
            )?;
            Ok(())
        }
        XChainKind::Eth => todo!(),
    }
}

fn verify_capacity(input_data: &ToCKBCellDataView) -> Result<(), Error> {
    let signer_xt_cell_cap = QueryIter::new(load_cell, Source::Output)
        .filter(|cell| cell.lock().as_bytes().as_ref() == input_data.signer_lockscript.as_ref())
        .map(|cell| cell.capacity().unpack())
        .collect::<Vec<u64>>()
        .into_iter()
        .sum::<u64>();
    let ckb_cell_cap = load_cell_capacity(0, Source::GroupInput)?;
    if signer_xt_cell_cap != ckb_cell_cap {
        return Err(Error::CapacityInvalid);
    }
    Ok(())
}

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    debug!("start withdraw collateral");
    let input_data = toCKB_data_tuple.0.as_ref().expect("should not happen");
    verify_capacity(input_data)?;
    debug!("verify capacity finish");
    verify_witness(input_data)?;
    debug!("verify witness finish");
    Ok(())
}

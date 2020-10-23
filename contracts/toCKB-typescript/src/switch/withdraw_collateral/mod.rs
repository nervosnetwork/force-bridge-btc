use crate::switch::ToCKBCellDataTuple;
use crate::utils::{
    tools::{verify_btc_witness, XChainKind},
    types::{mint_xt_witness::MintXTWitnessReader, Error, ToCKBCellDataView, XExtraView},
};

use crate::utils::common::verify_capacity_with_value;
use ckb_std::ckb_types::prelude::*;
use ckb_std::{
    ckb_constants::Source,
    debug,
    high_level::{load_cell_capacity, load_witness_args},
};
use core::result::Result;

/// ensure transfer happen on XChain by verifying the spv proof
fn verify_witness(data: &ToCKBCellDataView) -> Result<XExtraView, Error> {
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
            let btc_extra = verify_btc_witness(
                data,
                proof,
                cell_dep_index_list,
                data.x_unlock_address.as_ref(),
                data.get_btc_lot_size()?.get_sudt_amount(),
                true,
            )?;
            debug!("extra {:?}", btc_extra);
            Ok(XExtraView::Btc(btc_extra))
        }
        XChainKind::Eth => todo!(),
    }
}

fn verify_extra(data: &ToCKBCellDataView, x_extra: &XExtraView) -> Result<(), Error> {
    if &data.x_extra != x_extra {
        return Err(Error::FaultyBtcWitnessInvalid);
    }
    Ok(())
}

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    debug!("start withdraw collateral");
    let input_data = toCKB_data_tuple.0.as_ref().expect("should not happen");
    // verify_capacity(input_data)?;
    let ckb_cell_cap = load_cell_capacity(0, Source::GroupInput)?;
    verify_capacity_with_value(input_data, ckb_cell_cap)?;
    debug!("verify capacity finish");
    let extra = verify_witness(input_data)?;
    debug!("verify witness finish");
    verify_extra(input_data, &extra)?;
    debug!("verify extra finish");
    Ok(())
}

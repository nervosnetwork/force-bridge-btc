use crate::switch::ToCKBCellDataTuple;
use crate::utils::common::verify_capacity;
use crate::utils::{
    tools::{verify_btc_faulty_witness, XChainKind},
    types::{mint_xt_witness::MintXTWitnessReader, Error, ToCKBCellDataView},
};
use ckb_std::{ckb_constants::Source, debug, high_level::load_witness_args};
use core::result::Result;
use molecule::prelude::Reader;

pub fn verify(toCKB_data_tuple: &ToCKBCellDataTuple) -> Result<(), Error> {
    let input_data = toCKB_data_tuple
        .0
        .as_ref()
        .expect("inputs should contain toCKB cell");
    let output_data = toCKB_data_tuple
        .1
        .as_ref()
        .expect("outputs should contain toCKB cell");

    verify_capacity()?;
    verify_data(input_data, output_data)?;
    verify_witness(input_data)
}

fn verify_data(
    input_data: &ToCKBCellDataView,
    output_data: &ToCKBCellDataView,
) -> Result<(), Error> {
    if input_data.get_raw_lot_size() != output_data.get_raw_lot_size()
        || input_data.user_lockscript != output_data.user_lockscript
        || input_data.x_lock_address != output_data.x_lock_address
        || input_data.signer_lockscript != output_data.signer_lockscript
        || input_data.x_extra != output_data.x_extra
    {
        return Err(Error::InvariantDataMutated);
    }

    Ok(())
}

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
        XChainKind::Btc => verify_btc_faulty_witness(data, proof, cell_dep_index_list, false),
        XChainKind::Eth => todo!(),
    }
}

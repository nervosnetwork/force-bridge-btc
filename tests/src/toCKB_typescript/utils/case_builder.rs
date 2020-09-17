use crate::toCKB_typescript::utils::types::generated::{
    basic, mint_xt_witness, btc_difficulty,
    tockb_cell_data::{BtcExtra, EthExtra, ToCKBCellData, ToCKBTypeArgs, XExtra, XExtraUnion},
};
pub use crate::toCKB_typescript::utils::types::tockb_cell::XExtraView;
use anyhow::Result;
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_tool::ckb_types::{bytes::Bytes, packed::*, prelude::*};
use molecule::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::vec::Vec;

pub const USER_LOCKSCRIPT_OUTPOINT_KEY: &str = "user_lockscript_outpoint_key";
pub const TOCKB_TYPESCRIPT_OUTPOINT_KEY: &str = "toCKB_typescript_outpoint_key";
pub const TOCKB_LOCKSCRIPT_OUTPOINT_KEY: &str = "toCKB_lockscript_outpoint_key";
pub const SUDT_TYPESCRIPT_OUTPOINT_KEY: &str = "sudt_typescript_key";
pub const FIRST_INPUT_OUTPOINT_KEY: &str = "toCKB_cell_id_outpoint_key";
pub const ALWAYS_SUCCESS_OUTPOINT_KEY: &str = "always_success_outpoint_key";

pub type OutpointsContext = HashMap<&'static str, OutPoint>;

pub trait BuildCell {
    fn build_input_cell(
        &self,
        context: &mut Context,
        outpoints: &OutpointsContext,
    ) -> (OutPoint, CellInput) {
        let (cell_data, cell) = self.build_output_cell(context, outpoints);
        let input_out_point = context.create_cell(cell, cell_data);
        let input_cell = CellInput::new_builder()
            .previous_output(input_out_point.clone())
            .build();
        (input_out_point, input_cell)
    }

    fn build_output_cell(
    &self,
    context: &mut Context,
    outpoints: &OutpointsContext,
    ) -> (Bytes, CellOutput);

    fn get_index(&self) -> usize;
}

pub struct TestCase {
    pub cell_deps: Vec<CellDepView>,
    pub toCKB_cells: ToCKBCells,
    pub sudt_cells: SudtCells,
    pub capacity_cells: CapacityCells,
    pub witnesses: Vec<Witness>,
    pub expect_return_code: i8,
}

pub enum CellDepView {
    DifficultyOracle(DifficultyOracle),
    PriceOracle(u128),
}

impl CellDepView {
    pub fn build_cell_dep(&self, context: &mut Context) -> CellDep {
        match self {
            CellDepView::DifficultyOracle(difficulty) => {
                let difficulty = btc_difficulty::BTCDifficulty::new_builder()
                    .previous(difficulty.previous.to_le_bytes().to_vec().into())
                    .current(difficulty.current.to_le_bytes().to_vec().into())
                    .build();
                let difficulty_outpoint = context.deploy_cell(difficulty.as_bytes());
                CellDep::new_builder().out_point(difficulty_outpoint).build()
            },
            CellDepView::PriceOracle(price) => {
                let price_outpoint = context.deploy_cell(price.to_le_bytes().to_vec().into());
                CellDep::new_builder().out_point(price_outpoint).build()
            }
        }
    }
}

pub struct DifficultyOracle {
    pub previous: u64,
    pub current: u64,
}

pub struct ToCKBCells {
    pub inputs: Vec<ToCKBCell>,
    pub outputs: Vec<ToCKBCell>,
}

pub struct ToCKBCell {
    pub capacity: u64,
    pub data: ToCKBCellDataView,
    pub type_args: ToCKBTypeArgsView,
    pub index: usize,
}

impl ToCKBCell {
    fn build_typescript(&self, context: &mut Context, outpoints: &OutpointsContext) -> Script {
        let args = self.type_args.as_molecule_bytes(outpoints);
        context
            .build_script(&outpoints[TOCKB_TYPESCRIPT_OUTPOINT_KEY], args)
            .expect("build toCKB typescript succ")
    }

    fn build_lockscript(&self, context: &mut Context, outpoints: &OutpointsContext) -> Script {
        context
            .build_script(
                &outpoints[TOCKB_LOCKSCRIPT_OUTPOINT_KEY],
                Default::default(),
            )
            .expect("build toCKB lockscript succ")
    }
}

impl BuildCell for ToCKBCell {
    fn build_output_cell(
        &self,
        context: &mut Context,
        outpoints: &OutpointsContext,
    ) -> (Bytes, CellOutput) {
        let output_cell = CellOutput::new_builder()
            .capacity(self.capacity.pack())
            .type_(Some(self.build_typescript(context, outpoints)).pack())
            .lock(self.build_lockscript(context, outpoints))
            .build();
        let output_data = self.data.as_molecule_bytes(context, outpoints);
        (output_data, output_cell)
    }

    fn get_index(&self) -> usize {
        self.index
    }
}

pub struct ToCKBCellDataView {
    pub status: u8,
    pub lot_size: u8,
    pub user_lockscript: ScriptView,
    pub x_lock_address: String,
    pub signer_lockscript: ScriptView,
    pub x_unlock_address: String,
    pub redeemer_lockscript: ScriptView,
    pub liquidation_trigger_lockscript: ScriptView,
    pub x_extra: XExtraView,
}

#[derive(Default)]
pub struct ScriptView {
    pub outpoint_key: &'static str,
    pub args: Bytes,
}

impl ScriptView {
    pub fn build_script(&self, context: &mut Context, outpoints: &OutpointsContext) -> Script {
        context
            .build_script(&outpoints[self.outpoint_key], self.args.clone())
            .expect("build script succ")
    }

    pub fn build_basic_script(
        &self,
        context: &mut Context,
        outpoints: &OutpointsContext,
    ) -> basic::Script {
        context
            .build_script(&outpoints[self.outpoint_key], self.args.clone())
            .expect("build script succ")
            .into()
    }
}

impl ToCKBCellDataView {
    pub fn as_molecule_bytes(&self, context: &mut Context, outpoints: &OutpointsContext) -> Bytes {
        let toCKB_data = ToCKBCellData::new_builder()
            .status(Byte::new(self.status))
            .lot_size(Byte::new(self.lot_size))
            .user_lockscript(self.user_lockscript.build_basic_script(context, outpoints))
            .x_lock_address(str_to_molecule_bytes(self.x_lock_address.as_str()))
            .signer_lockscript(
                self.signer_lockscript
                    .build_basic_script(context, outpoints),
            )
            .x_unlock_address(str_to_molecule_bytes(self.x_unlock_address.as_str()))
            .redeemer_lockscript(
                self.redeemer_lockscript
                    .build_basic_script(context, outpoints),
            )
            .liquidation_trigger_lockscript(
                self.liquidation_trigger_lockscript
                    .build_basic_script(context, outpoints),
            )
            .x_extra(to_molecule_xextra(&self.x_extra))
            .build();
        toCKB_data.as_bytes()
    }
}

pub struct ToCKBTypeArgsView {
    pub xchain_kind: u8,
    pub cell_id: Option<Bytes>,
}

impl ToCKBTypeArgsView {
    pub fn as_molecule_bytes(&self, outpoints: &OutpointsContext) -> Bytes {
        if let Some(cell_id) = self.cell_id.as_ref() {
            let toCKB_type_args = ToCKBTypeArgs::new_builder()
                .xchain_kind(Byte::new(self.xchain_kind))
                .cell_id(basic::OutPoint::new_unchecked(cell_id.clone()))
                .build();
            toCKB_type_args.as_bytes()
        } else {
            let cell_id = &outpoints[FIRST_INPUT_OUTPOINT_KEY];
            let toCKB_type_args = ToCKBTypeArgs::new_builder()
                .xchain_kind(Byte::new(self.xchain_kind))
                .cell_id(cell_id.clone().into())
                .build();
            toCKB_type_args.as_bytes()
        }
    }
}

#[derive(Default)]
pub struct SudtCells {
    pub inputs: Vec<SudtCell>,
    pub outputs: Vec<SudtCell>,
}

#[derive(Default)]
pub struct SudtCell {
    pub capacity: u64,
    pub amount: u128,
    pub lockscript: ScriptView,
    pub index: usize,
}

impl BuildCell for SudtCell {
    fn build_output_cell(
        &self,
        context: &mut Context,
        outpoints: &OutpointsContext,
    ) -> (Bytes, CellOutput) {
        let output_cell = CellOutput::new_builder()
            .capacity(self.capacity.pack())
            .lock(self.lockscript.build_script(context, outpoints))
            .build();
        let output_data = self.amount.to_le_bytes().to_vec().into();
        (output_data, output_cell)
    }

    fn get_index(&self) -> usize {
        self.index
    }
}

pub struct CapacityCells {
    pub inputs: Vec<CapacityCell>,
    pub outputs: Vec<CapacityCell>,
}

pub struct CapacityCell {
    pub capacity: u64,
    pub lockscript: ScriptView,
    pub index: usize,
}

impl BuildCell for CapacityCell {
    fn build_output_cell(
        &self,
        context: &mut Context,
        outpoints: &OutpointsContext,
    ) -> (Bytes, CellOutput) {
        let output_cell = CellOutput::new_builder()
            .capacity(self.capacity.pack())
            .lock(self.lockscript.build_script(context, outpoints))
            .build();
        (Default::default(), output_cell)
    }

    fn get_index(&self) -> usize {
        self.index
    }
}

pub enum Witness {
    Btc(BtcWitness),
}

impl Witness {
    pub fn as_bytes(&self) -> Bytes {
        match self {
            Witness::Btc(btc_witness) => btc_witness.as_bytes()
        }
    }
}

pub struct BtcWitness {
    pub cell_dep_index_list: Vec<u8>,
    pub spv_proof: BTCSPVProofJson,
}

impl BtcWitness {
    pub fn as_bytes(&self) -> Bytes {
        let spv_proof: mint_xt_witness::BTCSPVProof = self.spv_proof.clone().try_into().expect("try into mint_xt_witness::BTCSPVProof succ");
        let spv_proof = spv_proof.as_slice().to_vec();
        let witness_data = mint_xt_witness::MintXTWitness::new_builder()
            .spv_proof(spv_proof.into())
            .cell_dep_index_list(self.cell_dep_index_list.clone().into())
            .build();
        let witness = WitnessArgs::new_builder()
            .input_type(Some(witness_data.as_bytes()).pack())
            .build();
        witness.as_bytes()
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct BTCSPVProofJson {
    pub version: u32,
    pub vin: String,
    pub vout: String,
    pub locktime: u32,
    pub tx_id: String,
    pub index: u64,
    pub headers: String,
    pub intermediate_nodes: String,
    pub funding_output_index: u32,
    pub funding_input_index: u32,
}

impl TryFrom<BTCSPVProofJson> for mint_xt_witness::BTCSPVProof {
    type Error = anyhow::Error;

    fn try_from(proof: BTCSPVProofJson) -> Result<Self> {
        Ok(mint_xt_witness::BTCSPVProof::new_builder()
            .version(proof.version.into())
            .vin(hex::decode(clear_0x(&proof.vin))?.into())
            .vout(hex::decode(clear_0x(&proof.vout))?.into())
            .locktime(proof.locktime.into())
            .tx_id(hex::decode(clear_0x(&proof.tx_id))?.into())
            .index(proof.index.into())
            .headers(hex::decode(clear_0x(&proof.headers))?.into())
            .intermediate_nodes(hex::decode(clear_0x(&proof.intermediate_nodes))?.into())
            .funding_output_index(proof.funding_output_index.into())
            .funding_input_index(proof.funding_input_index.into())
            .build())
    }
}

pub fn clear_0x(s: &str) -> &str {
    if &s[..2] == "0x" || &s[..2] == "0X" {
        &s[2..]
    } else {
        s
    }
}

pub fn json_to_btc_proof(proof: &str) -> Result<mint_xt_witness::BTCSPVProof> {
    let proof: BTCSPVProofJson = serde_json::from_str(proof)?;
    proof.try_into()
}

fn str_to_molecule_bytes(s: &str) -> basic::Bytes {
    basic::Bytes::new_builder()
        .set(
            s.as_bytes()
                .iter()
                .map(|c| Byte::new(*c))
                .collect::<Vec<_>>()
                .into(),
        )
        .build()
}

fn to_molecule_xextra(extra_view: &XExtraView) -> XExtra {
    match extra_view {
        XExtraView::Btc(btc_extra) => {
            let lock_tx_hash = basic::Byte32::new_unchecked(btc_extra.lock_tx_hash.clone());
            let lock_vout_index = Vec::<u8>::from(&btc_extra.lock_vout_index.to_le_bytes()[..]);
            let lock_vout_index = basic::Uint32::new_unchecked(Bytes::from(lock_vout_index));
            let btc_extra = BtcExtra::new_builder()
                .lock_tx_hash(lock_tx_hash)
                .lock_vout_index(lock_vout_index)
                .build();
            let x_extra = XExtraUnion::BtcExtra(btc_extra);
            XExtra::new_builder().set(x_extra).build()
        }
        XExtraView::Eth(eth_extra) => {
            let eth_extra = EthExtra::new_builder()
                .dummy(basic::Bytes::new_unchecked(eth_extra.dummy.clone()))
                .build();
            let x_extra = XExtraUnion::EthExtra(eth_extra);
            XExtra::new_builder().set(x_extra).build()
        }
    }
}

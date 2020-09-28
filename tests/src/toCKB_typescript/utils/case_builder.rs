use crate::toCKB_typescript::utils::types::generated::{
    basic,
    basic::Bytes2,
    btc_difficulty,
    eth_header_cell_data::{Chain, EthCellData, HeaderInfo},
    mint_xt_witness,
    tockb_cell_data::{BtcExtra, EthExtra, ToCKBCellData, ToCKBTypeArgs, XExtra, XExtraUnion},
};
use anyhow::Result;
use ckb_testtool::context::Context;
pub use ckb_tool::ckb_types::bytes::Bytes;
use ckb_tool::ckb_types::{packed::*, prelude::*};
use eth_spv_lib::eth_types::BlockHeader;
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

pub trait CellBuilder {
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
    HeadersOracle(HeadersOracle),
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
                CellDep::new_builder()
                    .out_point(difficulty_outpoint)
                    .build()
            }
            CellDepView::PriceOracle(price) => {
                let price_outpoint = context.deploy_cell(price.to_le_bytes().to_vec().into());
                CellDep::new_builder().out_point(price_outpoint).build()
            }
            CellDepView::HeadersOracle(headersOracle) => {
                let headers_data = &headersOracle.headers;
                let mut headers: Vec<basic::Bytes> = vec![];
                for header_str in headers_data {
                    let header: BlockHeader = rlp::decode(
                        hex::decode(&header_str)
                            .expect("invalid header rlp string.")
                            .as_slice(),
                    )
                    .expect("invalid header rlp string.");
                    let header_info = HeaderInfo::new_builder()
                        .header(
                            hex::decode(&header_str)
                                .expect("invalid header rlp string.")
                                .as_slice()
                                .to_vec()
                                .into(),
                        )
                        // .total_difficulty(header.difficulty.0.as_u64().into())
                        .hash(
                            basic::Byte32::from_slice(
                                header.hash.expect("invalid hash.").0.as_bytes(),
                            )
                            .expect("invalid hash."),
                        )
                        .build();
                    headers.push(header_info.as_slice().to_vec().into());
                }
                let eth_cell_data = EthCellData::new_builder()
                    .headers(
                        Chain::new_builder()
                            .main(Bytes2::new_builder().set(headers).build())
                            .build(),
                    )
                    .build();
                let headers_outpoint = context.deploy_cell(eth_cell_data.as_bytes());
                CellDep::new_builder().out_point(headers_outpoint).build()
            }
        }
    }
}

pub struct DifficultyOracle {
    pub previous: u64,
    pub current: u64,
}

pub struct HeadersOracle {
    pub headers: Vec<String>,
}

pub struct ToCKBCells {
    pub inputs: Vec<ToCKBCell>,
    pub outputs: Vec<ToCKBCell>,
}

pub struct ToCKBCell {
    pub capacity: u64,
    pub data: ToCKBCellDataView,
    pub type_args: ToCKBTypeArgsView,
    pub since: u64,
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

impl CellBuilder for ToCKBCell {
    fn build_input_cell(
        &self,
        context: &mut Context,
        outpoints: &OutpointsContext,
    ) -> (OutPoint, CellInput) {
        let (cell_data, cell) = self.build_output_cell(context, outpoints);
        let input_out_point = context.create_cell(cell, cell_data);
        let input_cell = CellInput::new_builder()
            .previous_output(input_out_point.clone())
            .since(self.since.pack())
            .build();
        (input_out_point, input_cell)
    }

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
            .x_extra(self.x_extra.as_xextra())
            .build();
        toCKB_data.as_bytes()
    }
}

pub struct ScriptView {
    pub outpoint_key: &'static str,
    pub args: Bytes,
}

impl Default for ScriptView {
    fn default() -> Self {
        Self {
            outpoint_key: ALWAYS_SUCCESS_OUTPOINT_KEY,
            args: Default::default(),
        }
    }
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

#[derive(Debug)]
pub enum XExtraView {
    Btc(BtcExtraView),
    Eth(EthExtraView),
}

impl Default for XExtraView {
    fn default() -> Self {
        Self::Btc(Default::default())
    }
}

impl XExtraView {
    pub fn as_xextra(&self) -> XExtra {
        match self {
            XExtraView::Btc(btc_extra) => {
                let lock_tx_hash = hex::decode(btc_extra.lock_tx_hash.as_str())
                    .expect("decode lock_tx_hash hex")
                    .iter()
                    .map(|v| Byte::new(*v))
                    .collect::<Vec<_>>();
                let lock_tx_hash = if lock_tx_hash.is_empty() {
                    basic::Byte32::new_builder().build()
                } else {
                    let lock_tx_hash: Box<[Byte; 32]> = lock_tx_hash
                        .into_boxed_slice()
                        .try_into()
                        .expect("convert lock_tx_hash");
                    basic::Byte32::new_builder().set(*lock_tx_hash).build()
                };
                let lock_vout_index = Vec::<u8>::from(&btc_extra.lock_vout_index.to_le_bytes()[..]);
                let lock_vout_index: Box<[Byte; 4]> = lock_vout_index
                    .iter()
                    .map(|v| Byte::new(*v))
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
                    .try_into()
                    .expect("convert lock_vout_index");
                let lock_vout_index = basic::Uint32::new_builder().set(*lock_vout_index).build();
                let btc_extra = BtcExtra::new_builder()
                    .lock_tx_hash(lock_tx_hash)
                    .lock_vout_index(lock_vout_index)
                    .build();
                let x_extra = XExtraUnion::BtcExtra(btc_extra);
                XExtra::new_builder().set(x_extra).build()
            }
            XExtraView::Eth(_) => {
                let eth_extra = EthExtra::new_builder()
                    // .dummy(basic::Bytes::new_unchecked(eth_extra.dummy.clone()))
                    .build();
                let x_extra = XExtraUnion::EthExtra(eth_extra);
                XExtra::new_builder().set(x_extra).build()
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct BtcExtraView {
    pub lock_tx_hash: String,
    pub lock_vout_index: u32,
}

#[derive(Debug)]
pub struct EthExtraView {
    pub dummy: Bytes,
}

impl Default for EthExtraView {
    fn default() -> Self {
        Self {
            dummy: basic::Bytes::new_builder().build().as_bytes(),
        }
    }
}

pub struct ToCKBTypeArgsView {
    pub xchain_kind: u8,
    pub cell_id: Option<Bytes>,
}

impl ToCKBTypeArgsView {
    pub fn default_cell_id() -> Option<Bytes> {
        Some(basic::OutPoint::new_builder().build().as_bytes())
    }
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
    pub owner_script: ScriptView,
    pub index: usize,
}

impl SudtCell {
    pub fn build_typescript(&self, context: &mut Context, outpoints: &OutpointsContext) -> Script {
        let owner_script = context
            .build_script(
                &outpoints[self.owner_script.outpoint_key],
                self.owner_script.args.clone(),
            )
            .expect("build owner script");
        let args: [u8; 32] = owner_script.calc_script_hash().unpack();
        let args: Bytes = args.to_vec().into();
        context
            .build_script(&outpoints[SUDT_TYPESCRIPT_OUTPOINT_KEY], args)
            .expect("build sudt typescript succ")
    }
}

impl CellBuilder for SudtCell {
    fn build_output_cell(
        &self,
        context: &mut Context,
        outpoints: &OutpointsContext,
    ) -> (Bytes, CellOutput) {
        let output_cell = CellOutput::new_builder()
            .capacity(self.capacity.pack())
            .type_(Some(self.build_typescript(context, outpoints)).pack())
            .lock(self.lockscript.build_script(context, outpoints))
            .build();
        let output_data = self.amount.to_le_bytes().to_vec().into();
        (output_data, output_cell)
    }

    fn get_index(&self) -> usize {
        self.index
    }
}

#[derive(Default)]
pub struct CapacityCells {
    pub inputs: Vec<CapacityCell>,
    pub outputs: Vec<CapacityCell>,
}

#[derive(Default)]
pub struct CapacityCell {
    pub capacity: u64,
    pub lockscript: ScriptView,
    pub index: usize,
}

impl CellBuilder for CapacityCell {
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

#[derive(Clone)]
pub enum Witness {
    Btc(BtcWitness),
    Eth(EthWitness),
}

impl Witness {
    pub fn as_bytes(&self) -> Bytes {
        match self {
            Witness::Btc(btc_witness) => btc_witness.as_bytes(),
            Witness::Eth(eth_witness) => eth_witness.as_bytes(),
        }
    }
}

#[derive(Clone)]
pub struct BtcWitness {
    pub cell_dep_index_list: Vec<u8>,
    pub spv_proof: BTCSPVProofJson,
}

#[derive(Clone)]
pub struct EthWitness {
    pub cell_dep_index_list: Vec<u8>,
    pub spv_proof: ETHSPVProofJson,
}

impl BtcWitness {
    pub fn as_bytes(&self) -> Bytes {
        let spv_proof: mint_xt_witness::BTCSPVProof = self
            .spv_proof
            .clone()
            .try_into()
            .expect("try into mint_xt_witness::BTCSPVProof succ");
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
            .tx_id(hex::decode(clear_0x(&proof.tx_id))?.try_into()?)
            .index(proof.index.into())
            .headers(hex::decode(clear_0x(&proof.headers))?.into())
            .intermediate_nodes(hex::decode(clear_0x(&proof.intermediate_nodes))?.into())
            .funding_output_index(proof.funding_output_index.into())
            .funding_input_index(proof.funding_input_index.into())
            .build())
    }
}

impl EthWitness {
    pub fn as_bytes(&self) -> Bytes {
        let spv_proof: mint_xt_witness::ETHSPVProof = self
            .spv_proof
            .clone()
            .try_into()
            .expect("try into mint_xt_witness::ETHSPVProof success");
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
pub struct ETHSPVProofJson {
    pub log_index: u64,
    pub log_entry_data: String,
    pub receipt_index: u64,
    pub receipt_data: String,
    pub header_data: String,
    pub proof: Vec<Vec<u8>>,
}

impl TryFrom<ETHSPVProofJson> for mint_xt_witness::ETHSPVProof {
    type Error = anyhow::Error;
    fn try_from(proof: ETHSPVProofJson) -> Result<Self> {
        let mut proofVec: Vec<basic::Bytes> = vec![];
        for i in 0..proof.proof.len() {
            // proofVec.push(hex::decode(clear_0x(&proof.proof[i]))?.into())
            proofVec.push(proof.proof[i].to_vec().into())
        }
        Ok(mint_xt_witness::ETHSPVProof::new_builder()
            .log_index(proof.log_index.into())
            .log_entry_data(hex::decode(clear_0x(&proof.log_entry_data))?.into())
            .receipt_index(proof.receipt_index.into())
            .receipt_data(hex::decode(clear_0x(&proof.receipt_data))?.into())
            .header_data(hex::decode(clear_0x(&proof.header_data))?.into())
            .proof(Bytes2::new_builder().set(proofVec).build())
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

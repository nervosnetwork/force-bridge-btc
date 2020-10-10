use crate::indexer::{Cell, IndexerRpcClient, Order, Pagination, ScriptType, SearchKey};
use crate::util::{get_live_cell_with_cache, is_mature};
use anyhow::{anyhow, Result};
use ckb_hash::blake2b_256;
use ckb_jsonrpc_types::{
    JsonBytes, Script as JsonScript, ScriptHashType as JsonScriptHashType, Uint32,
};
use ckb_sdk::{
    calc_max_mature_number,
    constants::{CELLBASE_MATURITY, MIN_SECP_CELL_CAPACITY, ONE_CKB},
    rpc::AlertMessage,
    wallet::{KeyStore, ScryptType},
    Address, AddressPayload, CodeHashIndex, GenesisInfo, HttpRpcClient, NetworkType, SignerFn,
    SECP256K1,
};
use ckb_types::{
    bytes::Bytes,
    core::{service::Request, BlockView, Capacity, EpochNumberWithFraction, TransactionView},
    h256,
    packed::{self, CellOutput, OutPoint, Script},
    prelude::*,
    H160, H256,
};
use secp256k1::SecretKey;
use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::TryFrom;
use tockb_types::config::UDT_LEN;

/// get live cell by typescript
/// it assumes there is at most 1 cell found
pub fn get_live_cell_by_typescript(
    indexer_client: &mut IndexerRpcClient,
    typescript: Script,
) -> Result<Option<Cell>, String> {
    let search_key = SearchKey {
        script: typescript.into(),
        script_type: ScriptType::Type,
        args_len: None,
    };
    let cells = get_live_cells(indexer_client, search_key, |_, _| (true, true))?;
    let len = cells.len();
    if len > 1 {
        return Err(format!("expected zero or one cell"));
    }
    if len == 1 {
        Ok(Some(cells[0].clone()))
    } else {
        Ok(None)
    }
}

/// get cells to supply capacity
/// if max_mature_number is None, skip mature check
pub fn get_live_cells_by_lock_and_capacity(
    indexer_client: &mut IndexerRpcClient,
    lockscript: Script,
    capacity: u64,
    max_mature_number: Option<u64>,
) -> Result<Vec<Cell>, String> {
    let mut accumulated_capacity = 0;
    let terminator = |_, cell: &Cell| {
        if accumulated_capacity >= capacity {
            (true, false)
        } else if cell.output.type_.is_none()
            && cell.output_data.len() == 0
            && max_mature_number
                .map(|n| is_mature(cell, n))
                .unwrap_or(true)
        {
            accumulated_capacity += cell.output.capacity.value();
            (accumulated_capacity > capacity, true)
        } else {
            (false, false)
        }
    };
    let search_key = SearchKey {
        script: lockscript.into(),
        script_type: ScriptType::Lock,
        args_len: None,
    };
    get_live_cells(indexer_client, search_key, terminator)
}

pub fn get_live_cells<F: FnMut(usize, &Cell) -> (bool, bool)>(
    indexer_client: &mut IndexerRpcClient,
    search_key: SearchKey,
    mut terminator: F,
) -> Result<Vec<Cell>, String> {
    let limit = Uint32::from(100u32);
    let mut infos = Vec::new();
    let mut cursor = None;
    loop {
        let live_cells: Pagination<Cell> =
            indexer_client.get_cells(search_key.clone(), Order::Asc, limit, cursor)?;
        if live_cells.objects.len() == 0 {
            break;
        }
        cursor = Some(live_cells.last_cursor);
        for (index, cell) in live_cells.objects.into_iter().enumerate() {
            let (stop, push_info) = terminator(index, &cell);
            if push_info {
                infos.push(cell);
            }
            if stop {
                return Ok(infos);
            }
        }
    }

    Ok(infos)
}

pub fn collect_sudt_cells_by_amout(
    indexer_client: &mut IndexerRpcClient,
    lockscript: Script,
    sudt_typescript: Script,
    need_sudt_amount: u128,
) -> Result<(u128, Vec<Cell>), String> {
    let mut collected_amount = 0u128;
    let terminator = |_, cell: &Cell| {
        if collected_amount >= need_sudt_amount {
            (true, false)
        } else if cell.output.type_.is_some()
            && packed::Script::from(cell.output.type_.clone().unwrap()) == sudt_typescript
            && cell.output_data.len() >= UDT_LEN
        {
            collected_amount += {
                let mut buf = [0u8; UDT_LEN];
                buf.copy_from_slice(cell.output_data.as_bytes());
                u128::from_le_bytes(buf)
            };
            dbg!(collected_amount);
            (collected_amount > need_sudt_amount, true)
        } else {
            (false, false)
        }
    };
    let search_key = SearchKey {
        script: lockscript.into(),
        script_type: ScriptType::Lock,
        args_len: None,
    };

    let cells = get_live_cells(indexer_client, search_key, terminator)?;
    Ok((collected_amount, cells))
}

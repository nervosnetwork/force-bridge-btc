use crate::utils::{
    config::{SUDT_CODE_HASH, UDT_LEN},
    types::Error,
};

use ckb_std::{
    ckb_constants::Source,
    ckb_types::packed::Script,
    debug,
    high_level::{load_cell_data, load_cell_type, load_script},
};
use core::result::Result;
use tockb_types::generated::basic::OutPoint;
pub use tockb_types::tockb_cell::{ToCKBTypeArgsView, XChainKind};

pub fn get_toCKB_type_args() -> Result<ToCKBTypeArgsView, Error> {
    let toCKB_type_args = load_script()?.args().raw_data();
    debug!("before molecule decode toCKB type args");
    let toCKB_type_args = ToCKBTypeArgsView::from_slice(toCKB_type_args.as_ref())?;
    debug!("molecule decode toCKB type args succ");
    Ok(toCKB_type_args)
}

pub fn get_xchain_kind() -> Result<XChainKind, Error> {
    Ok(get_toCKB_type_args()?.xchain_kind)
}

pub fn get_cell_id() -> Result<OutPoint, Error> {
    Ok(get_toCKB_type_args()?.cell_id)
}

pub fn get_price() -> Result<u128, Error> {
    let price_cell_data = load_cell_data(0, Source::CellDep)?;
    if price_cell_data.len() != 16 {
        return Err(Error::Encoding);
    }
    let mut buf = [0u8; 16];
    buf.copy_from_slice(&price_cell_data);
    let price: u128 = u128::from_le_bytes(buf);
    Ok(price)
}

pub fn is_XT_typescript(script: &Option<Script>, toCKB_lock_hash: &[u8]) -> bool {
    match script {
        Some(script) => {
            if script.code_hash().raw_data().as_ref() == SUDT_CODE_HASH.as_ref()
                && script.args().raw_data().as_ref() == toCKB_lock_hash
                && script.hash_type() == 0u8.into()
            {
                return true;
            }
            false
        }
        None => false,
    }
}

pub fn get_sum_sudt_amount(
    start_index: usize,
    source: Source,
    toCKB_lock_hash: &[u8],
) -> Result<u128, Error> {
    let mut index = start_index;
    let mut sum_amount = 0;
    loop {
        let res = load_cell_type(index, source);
        if res.is_err() {
            break;
        }
        let script = res.unwrap();
        if !is_XT_typescript(&script, toCKB_lock_hash) {
            index += 1;
            continue;
        }

        let cell_data = load_cell_data(index, source)?;
        let mut data = [0u8; UDT_LEN];
        data.copy_from_slice(&cell_data);
        let amount = u128::from_le_bytes(data);
        sum_amount += amount;
        index += 1;
    }

    Ok(sum_amount)
}

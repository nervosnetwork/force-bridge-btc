use core::result::Result;
use crate::Error;
use crate::switch::ToCKBCellTuple;

#[repr(i8)]
enum BTCLotSize {
    Quarter = 1,
    Half,
    Single
}

#[repr(i8)]
enum ETHLotSize {
    Half = 1,
    Single,
    Two,
    Three
}

pub fn verify(toCKB_cells: &ToCKBCellTuple) -> Result<(), Error> {
    Ok(())
}

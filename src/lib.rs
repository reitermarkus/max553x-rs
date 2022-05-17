//! https://datasheets.maximintegrated.com/en/ds/MAX5532-MAX5535.pdf

use core::marker::PhantomData;

pub struct Shutdown;
pub struct Standby;
pub struct Normal;

pub enum Command {
    LoadInputAFromShiftRegA = 0b0001,
    LoadShiftRegB = 0b0010,
    LoadDacRegsFromInputRegs = 0b1000,
}

pub struct Max5532<W, MODE> {
    writer: W,
    _mode: PhantomData<MODE>,
}

impl<W, MODE> Max5532<W, MODE> {
    pub fn new(writer: W) -> Max5532<W, Shutdown> {
        Max5532 { writer, _mode: PhantomData }
    }
    
    
}

//! Instr Unreachble

use crate::WResult;
use crate::bitcode::blocks::function_block::utils::{FunctionInstrData, FunctionInstrTrait};
use crate::bitcode::blocks::typeblock::TypeBlockUnit;

#[derive(Debug)]
pub(in crate::bitcode) struct InstrUnreachble;

impl FunctionInstrTrait for InstrUnreachble {
    fn res(&self) -> Option<&TypeBlockUnit> {
        None
    }

    fn new<'a>(_data: FunctionInstrData<'a>) -> WResult<Self> {
        Ok(Self)
    }
}

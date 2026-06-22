//! Instr Resume

use crate::WResult;
use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue,
};
use crate::bitcode::blocks::typeblock::TypeBlockUnit;

#[derive(Debug)]
pub(in crate::bitcode) struct InstrResume {
    pub(in crate::bitcode) op: GlobalValue,
}

impl FunctionInstrTrait for InstrResume {
    fn res(&self) -> Option<&TypeBlockUnit> {
        None
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let op = data.next_value("Resume op", None)?;
        Ok(Self { op })
    }
}

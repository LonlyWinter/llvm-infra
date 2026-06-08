//! Instr Store

use crate::WResult;
use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue,
};
use crate::bitcode::blocks::typeblock::TypeBlockUnit;

#[derive(Debug)]
pub(in crate::bitcode) struct InstrStore {
    pub(in crate::bitcode) ptr: GlobalValue,
    pub(in crate::bitcode) val: GlobalValue,
    pub(in crate::bitcode) align: u16,
}

impl FunctionInstrTrait for InstrStore {
    fn res(&self) -> Option<&TypeBlockUnit> {
        None
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let ptr = data.next_value("parse store ptr", None)?;

        let val = data.next_value("parse store val", None)?;

        let align = data.data.next_alignment()?;

        let r = Self { ptr, val, align };
        Ok(r)
    }
}

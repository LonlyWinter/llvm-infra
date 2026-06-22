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
    pub(in crate::bitcode) is_volatile: bool,
    pub(in crate::bitcode) align: u16,
}

impl FunctionInstrTrait for InstrStore {
    fn res(&self) -> Option<&TypeBlockUnit> {
        None
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let ptr = data.next_value("store ptr", None)?;

        let val = data.next_value("store val", None)?;

        let align = data.data.next_alignment("store align")?;
        let is_volatile = data.data.next_bool("store volatile")?;

        let r = Self { ptr, val, is_volatile, align };
        Ok(r)
    }
}

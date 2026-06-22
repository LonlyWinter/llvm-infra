//! Instr Br

use std::ops::Deref;

use crate::WResult;
use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue, TY_BOOL,
};
use crate::bitcode::blocks::typeblock::TypeBlockUnit;

#[derive(Debug)]
pub(in crate::bitcode) enum InstrBr {
    /// label
    DirectBr(usize),
    /// label1, label2, cond
    CondBr(usize, usize, GlobalValue),
}

impl FunctionInstrTrait for InstrBr {
    fn res(&self) -> Option<&TypeBlockUnit> {
        None
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let label1 = data.data.next("br label1")?;
        let r = if data.data.len() == 2 {
            let label2 = data.data.next("br label2")?;

            let binding = TY_BOOL;
            let ty_bool = binding.deref().clone();

            let cond = data.next_value("br cond", Some(ty_bool))?;

            Self::CondBr(label1, label2, cond)
        } else {
            Self::DirectBr(label1)
        };
        Ok(r)
    }
}

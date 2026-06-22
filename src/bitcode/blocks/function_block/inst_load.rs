//! Instr Load

use crate::WResult;
use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue,
};
use crate::bitcode::blocks::typeblock::TypeBlockUnit;

#[derive(Debug)]
pub(in crate::bitcode) struct InstrLoad {
    pub(in crate::bitcode) op: GlobalValue,
    pub(in crate::bitcode) ty: TypeBlockUnit,
    pub(in crate::bitcode) align: u16,
}

impl FunctionInstrTrait for InstrLoad {
    fn res(&self) -> Option<&TypeBlockUnit> {
        Some(&self.ty)
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let op = data.next_value("load op", None)?;

        let ty = data.next_ty("load ty")?;

        let align = data.data.next_alignment("load align")?;

        let r = Self { op, ty, align };
        Ok(r)
    }
}

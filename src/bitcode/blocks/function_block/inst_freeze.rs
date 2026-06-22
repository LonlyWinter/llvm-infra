//! Instr Freeze

use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue,
};
use crate::bitcode::blocks::typeblock::TypeBlockUnit;
use crate::{WError, WResult};

#[derive(Debug)]
pub(in crate::bitcode) struct InstrFreeze {
    pub(in crate::bitcode) op: GlobalValue,
    pub(in crate::bitcode) ty: TypeBlockUnit,
}

impl FunctionInstrTrait for InstrFreeze {
    fn res(&self) -> Option<&TypeBlockUnit> {
        Some(&self.ty)
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let op = data.next_value("Freeze op", None)?;
        let ty = op.get_ty("Freeze op ty")?;
        if data.data.len() != 0 {
            return Err(WError::DataNotAllowed(
                "Freeze data",
                format!("{:?}", data.data),
            ));
        }

        let r = Self { op, ty };
        Ok(r)
    }
}

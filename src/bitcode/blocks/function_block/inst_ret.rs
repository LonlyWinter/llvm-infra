//! Instr Ret

use std::rc::Rc;

use crate::WResult;
use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue,
};
use crate::bitcode::blocks::typeblock::{TypeBlockData, TypeBlockUnit};

#[derive(Debug)]
pub(in crate::bitcode) struct InstrRet {
    pub(in crate::bitcode) val: Option<GlobalValue>,
    pub(in crate::bitcode) ty: TypeBlockUnit,
}

impl FunctionInstrTrait for InstrRet {
    fn res(&self) -> Option<&TypeBlockUnit> {
        Some(&self.ty)
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        if data.data.len() == 0 {
            return Ok(Self {
                val: None,
                ty: Rc::new(TypeBlockData::Void),
            });
        }

        let val = data.next_value("parse ret val", None)?;
        let ty = val.get_ty("parse ret ty")?;

        let r = Self { val: Some(val), ty };
        Ok(r)
    }
}

//! Instr Ret

use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue,
};
use crate::bitcode::blocks::typeblock::TypeBlockUnit;
use crate::{WError, WResult};

#[derive(Debug)]
pub(in crate::bitcode) struct InstrRet {
    pub(in crate::bitcode) val: Option<GlobalValue>,
}

impl FunctionInstrTrait for InstrRet {
    fn res(&self) -> Option<&TypeBlockUnit> {
        None
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        if data.data.len() == 0 {
            return Ok(Self { val: None });
        }

        let val = data.next_value("ret val", None)?;
        let ty = val.get_ty("ret ty")?;

        if val.get_ty("instr ret")? != ty {
            return Err(WError::DataNotAllowed(
                "ret val ty",
                format!("val: {:?}, ty: {:?}", val, ty),
            ));
        }

        let r = Self { val: Some(val) };
        Ok(r)
    }
}

//! Instr GEP

use crate::WResult;
use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue,
};
use crate::bitcode::blocks::typeblock::TypeBlockUnit;

#[derive(Debug)]
pub(in crate::bitcode) struct InstrGEPNoWrapFlags {
    pub(in crate::bitcode) in_bounds: bool,
    pub(in crate::bitcode) nusw: bool,
    pub(in crate::bitcode) nuw: bool,
}

#[derive(Debug)]
pub(in crate::bitcode) struct InstrGEP {
    pub(in crate::bitcode) no_wrap_flags: InstrGEPNoWrapFlags,
    pub(in crate::bitcode) ty: TypeBlockUnit,
    pub(in crate::bitcode) base: GlobalValue,
    pub(in crate::bitcode) idx: GlobalValue,
}

impl FunctionInstrTrait for InstrGEP {
    fn res(&self) -> Option<&TypeBlockUnit> {
        Some(&self.ty)
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let no_wrap_flags = {
            let flags = data.data.next("parse gep no_wrap_flags")?;
            let in_bounds = (flags & 1) != 0;
            let nusw = ((flags >> 1) & 1) != 0;
            let nuw = ((flags >> 2) & 1) != 0;
            InstrGEPNoWrapFlags {
                in_bounds,
                nusw,
                nuw,
            }
        };

        let ty = data.next_ty("parse gep ty")?;

        let base = data.next_value("parse gep base", None)?;

        let idx = data.next_value("parse gep idx", None)?;

        let r = Self {
            no_wrap_flags,
            ty,
            base,
            idx,
        };
        Ok(r)
    }
}

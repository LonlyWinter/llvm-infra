//! Instr GEP

use std::rc::Rc;

use crate::WResult;
use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue,
};
use crate::bitcode::blocks::typeblock::{TypeBlockData, TypeBlockPointer, TypeBlockUnit};

#[derive(Debug, PartialEq, Clone)]
pub(in crate::bitcode) struct InstrGEPNoWrapFlags {
    pub(in crate::bitcode) in_bounds: bool,
    pub(in crate::bitcode) nusw: bool,
    pub(in crate::bitcode) nuw: bool,
}

impl From<usize> for InstrGEPNoWrapFlags {
    fn from(flags: usize) -> Self {
        let in_bounds = (flags & 1) != 0;
        let nusw = (flags & 2) != 0;
        let nuw = (flags & 4) != 0;
        Self {
            in_bounds,
            nusw,
            nuw,
        }
    }
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
            let flags = data.data.next("gep no_wrap_flags")?;
            InstrGEPNoWrapFlags::from(flags)
        };

        let ty = Rc::new(TypeBlockData::Pointer(TypeBlockPointer {
            ty: data.next_ty("gep ty")?,
            space: 0,
        }));

        let base = data.next_value("gep base", None)?;

        let idx = data.next_value("gep idx", None)?;

        let r = Self {
            no_wrap_flags,
            ty,
            base,
            idx,
        };
        Ok(r)
    }
}

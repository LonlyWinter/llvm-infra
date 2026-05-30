//! Instr VSelect

use std::ops::Deref;

use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue, InstrDataFastMathFlags, TY_BOOL,
};
use crate::bitcode::blocks::typeblock::{TypeBlockData, TypeBlockUnit};
use crate::{WError, WResult};

#[derive(Debug)]
pub(in crate::bitcode) struct InstrVSelect {
    pub(in crate::bitcode) val_true: GlobalValue,
    pub(in crate::bitcode) val_false: GlobalValue,
    pub(in crate::bitcode) cond: GlobalValue,
    pub(in crate::bitcode) ty: TypeBlockUnit,
    pub(in crate::bitcode) fmf: Option<InstrDataFastMathFlags>,
}

impl FunctionInstrTrait for InstrVSelect {
    fn res(&self) -> Option<&TypeBlockUnit> {
        Some(&self.ty)
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let val_true = data.next_value("parse vselect true", None)?;

        let ty = val_true.get_ty("parse vselect true ty")?;
        let val_false = data.next_value("parse vselect false", Some(ty.clone()))?;

        let cond = data.next_value("parse vselect cond", None)?;

        let ty_cond = cond.get_ty("parse vselect cond ty")?;
        let binding = TY_BOOL;
        let ty_bool = binding.deref();
        match ty_cond.as_ref() {
            TypeBlockData::Array(v) if &v.ty == ty_bool => {}
            TypeBlockData::Vector(v) if &v.ty == ty_bool => {}
            v if v == ty_bool.as_ref() => {}
            _ => {
                return Err(WError::DataNotAllowed(
                    "check vsekect ty",
                    format!("{:?}", ty_cond),
                ));
            }
        }

        // TODO: FMF

        let r = Self {
            val_true,
            val_false,
            cond,
            ty,
            fmf: None,
        };
        Ok(r)
    }
}

//! Instr InsertElt

use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue,
};
use crate::bitcode::blocks::typeblock::{TypeBlockData, TypeBlockUnit};
use crate::{WError, WResult};

#[derive(Debug)]
pub(in crate::bitcode) struct InstrInsertElt {
    pub(in crate::bitcode) src: GlobalValue,
    pub(in crate::bitcode) elt: GlobalValue,
    pub(in crate::bitcode) idx: GlobalValue,
    pub(in crate::bitcode) ty: TypeBlockUnit,
}

impl FunctionInstrTrait for InstrInsertElt {
    fn res(&self) -> Option<&TypeBlockUnit> {
        Some(&self.ty)
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let src = data.next_value("insert_elt src", None)?;

        let ty = src.get_ty("insert_elt src ty")?;
        let ty_unit = match ty.as_ref() {
            TypeBlockData::Vector(v) | TypeBlockData::Array(v) => v.ty.clone(),
            _ => {
                return Err(WError::DataNotAllowed(
                    "check insert_elt src ty",
                    format!("{:?}", ty),
                ));
            }
        };

        let elt = data.next_value("insert_elt elt", Some(ty_unit))?;

        let idx = data.next_value("insert_elt idx", None)?;

        let r = Self { src, elt, idx, ty };
        Ok(r)
    }
}

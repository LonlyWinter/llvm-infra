//! Instr ExtractElt

use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue,
};
use crate::bitcode::blocks::typeblock::{TypeBlockData, TypeBlockUnit};
use crate::{WError, WResult};

#[derive(Debug)]
pub(in crate::bitcode) struct InstrExtractElt {
    pub(in crate::bitcode) src: GlobalValue,
    pub(in crate::bitcode) idx: GlobalValue,
    pub(in crate::bitcode) ty: TypeBlockUnit,
}

impl FunctionInstrTrait for InstrExtractElt {
    fn res(&self) -> Option<&TypeBlockUnit> {
        Some(&self.ty)
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let src = data.next_value("parse extract_elt src", None)?;

        let ty = src.get_ty("parse extract_elt vec ty")?;
        let ty = if let TypeBlockData::Vector(v) | TypeBlockData::Array(v) = ty.as_ref() {
            v.ty.clone()
        } else {
            return Err(WError::DataNotAllowed(
                "check insert_elt vec ty",
                format!("{:?}", ty),
            ));
        };

        let idx = data.next_value("parse extract_elt idx", None)?;

        let r = Self { src, idx, ty };
        Ok(r)
    }
}

//! Instr ExtractVal

use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue,
};
use crate::bitcode::blocks::typeblock::{TypeBlockData, TypeBlockUnit};
use crate::{WError, WResult};

#[derive(Debug)]
pub(in crate::bitcode) struct InstrExtractVal {
    pub(in crate::bitcode) src: GlobalValue,
    pub(in crate::bitcode) idxs: Vec<usize>,
    pub(in crate::bitcode) ty: TypeBlockUnit,
}

impl FunctionInstrTrait for InstrExtractVal {
    fn res(&self) -> Option<&TypeBlockUnit> {
        Some(&self.ty)
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let src = data.next_value("extract_val src", None)?;

        let mut ty = src.get_ty("extract_val src ty")?;
        let mut idxs = Vec::with_capacity(4);
        for idx in data.data.into_iter() {
            idxs.push(idx);
            ty = match ty.as_ref() {
                TypeBlockData::StructAnon(v) if idx < v.tys.len() => v.tys[idx].clone(),
                TypeBlockData::StructNamed(v) if idx < v.tys.len() => v.tys[idx].clone(),
                TypeBlockData::Array(v) if idx < v.num => v.ty.clone(),
                _ => {
                    return Err(WError::DataNotAllowed(
                        "check extract_val ty",
                        format!("{} {:?}", idx, ty),
                    ));
                }
            };
        }
        if idxs.is_empty() {
            return Err(WError::DataNotFound("extract_val idx"));
        }
        idxs.shrink_to_fit();
        let r = Self { src, idxs, ty };
        Ok(r)
    }
}

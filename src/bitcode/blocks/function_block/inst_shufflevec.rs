//! Instr ShuffleVec

use std::rc::Rc;

use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue,
};
use crate::bitcode::blocks::typeblock::{TypeBlockArray, TypeBlockData, TypeBlockUnit};
use crate::{WError, WResult};

#[derive(Debug)]
pub(in crate::bitcode) struct InstrShuffleVec {
    pub(in crate::bitcode) vec1: GlobalValue,
    pub(in crate::bitcode) vec2: GlobalValue,
    pub(in crate::bitcode) mask: GlobalValue,
    pub(in crate::bitcode) ty: TypeBlockUnit,
}

impl FunctionInstrTrait for InstrShuffleVec {
    fn res(&self) -> Option<&TypeBlockUnit> {
        Some(&self.ty)
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let vec1 = data.next_value("parse shuffle_vec vec1", None)?;

        let ty_vec1 = vec1.get_ty("shuffle_vec vec1 ty")?;
        let vec2 = data.next_value("parse shuffle_vec vec2", Some(ty_vec1.clone()))?;

        let mask = data.next_value("parse shuffle_vec mask", None)?;

        let ty_mask = mask.get_ty("shuffle_vec mask ty")?;
        let ty = if let TypeBlockData::Vector(v1) = ty_vec1.as_ref()
            && let TypeBlockData::Vector(v2) = ty_mask.as_ref()
        {
            Rc::new(TypeBlockData::Vector(TypeBlockArray {
                num: v2.num,
                ty: v1.ty.clone(),
            }))
        } else {
            return Err(WError::DataNotAllowed(
                "parse shuffle_vec ty",
                format!("{:?} {:?}", ty_vec1, ty_mask),
            ));
        };

        let r = Self {
            vec1,
            vec2,
            mask,
            ty,
        };
        Ok(r)
    }
}

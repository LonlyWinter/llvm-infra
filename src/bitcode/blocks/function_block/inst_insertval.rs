//! Instr Insertval

use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue,
};
use crate::bitcode::blocks::typeblock::{TypeBlockData, TypeBlockUnit};
use crate::{WError, WResult};

#[derive(Debug)]
pub(in crate::bitcode) struct InstrInsertVal {
    pub(in crate::bitcode) src: GlobalValue,
    pub(in crate::bitcode) val: GlobalValue,
    pub(in crate::bitcode) idxs: Vec<usize>,
    pub(in crate::bitcode) ty: TypeBlockUnit,
}

impl FunctionInstrTrait for InstrInsertVal {
    fn res(&self) -> Option<&TypeBlockUnit> {
        Some(&self.ty)
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        // eprintln!("insertval: data {:?}", data.data);
        let src = data.next_value("insert_val src", None)?;
        let val = data.next_value("insert_val val", None)?;

        // eprintln!("insertval: {:?} {:?}", src, val);

        let mut ty = src.get_ty("insert_val src ty")?;
        let ty_res = ty.clone();
        let mut idxs = Vec::with_capacity(4);
        for idx in data.data.into_iter() {
            idxs.push(idx);
            ty = match ty.as_ref() {
                TypeBlockData::StructAnon(v) if idx < v.tys.len() => v.tys[idx].clone(),
                TypeBlockData::StructNamed(v) if idx < v.tys.len() => v.tys[idx].clone(),
                TypeBlockData::Array(v) if idx < v.num => v.ty.clone(),
                _ => {
                    return Err(WError::DataNotAllowed(
                        "check insert_val ty",
                        format!("{} {:?}", idx, ty),
                    ));
                }
            };
        }
        if idxs.is_empty() {
            return Err(WError::DataNotFound("insert_val idx"));
        }
        idxs.shrink_to_fit();

        let ty_val = val.get_ty("insert_val val ty")?;
        if ty_val != ty && !matches!(ty.as_ref(), TypeBlockData::OpaquePointer(_)) {
            return Err(WError::DataNotAllowed(
                "check insert_val ty",
                format!("{:?} {:?}", ty_val, ty),
            ));
        }

        let r = Self {
            src,
            val,
            idxs,
            ty: ty_res,
        };
        Ok(r)
    }
}

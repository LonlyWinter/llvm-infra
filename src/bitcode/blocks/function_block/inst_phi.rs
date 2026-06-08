//! Instr Phi

use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue,
};
use crate::bitcode::blocks::typeblock::TypeBlockUnit;
use crate::filebit::decoded_signed_vbrs_u32;
use crate::{WError, WResult};

#[derive(Debug)]
pub(in crate::bitcode) struct InstrPhi {
    pub(in crate::bitcode) ty: TypeBlockUnit,
    pub(in crate::bitcode) srcs: Vec<(GlobalValue, usize)>,
}

impl FunctionInstrTrait for InstrPhi {
    fn res(&self) -> Option<&TypeBlockUnit> {
        Some(&self.ty)
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let ty = data.next_ty("parse phi ty")?;
        // eprintln!("Phi ty: {:?}", ty);
        let num = data.data.len() / 2;
        let srcs = (0..num)
            .map(|_| {
                let idx_val = data.data.next("parse phi val idx")? as u32;
                let idx_val = decoded_signed_vbrs_u32(idx_val) as usize;
                let (val, idx_val) = data.other.get_value(
                    idx_val,
                    Some(ty.clone()),
                    "parsh phi val", || Err(WError::DataNotFound("parse phi val ty"))
                )?;
                let idx_block = data.data.next("parse phi block")?;
                Ok((
                    GlobalValue {
                        data: val,
                        idx: idx_val,
                    },
                    idx_block,
                ))
            })
            .collect::<WResult<Vec<_>>>()?;
        let r = Self { ty, srcs };
        Ok(r)
    }
}

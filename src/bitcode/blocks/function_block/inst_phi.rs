//! Instr Phi

use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue,
};
use crate::bitcode::blocks::typeblock::TypeBlockUnit;
use crate::filebit::decoded_signed_vbrs_u64;
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
        let ty = data.next_ty("phi ty")?;
        // eprintln!("Phi ty: {:?}", ty);
        let num = data.data.len() / 2;
        let srcs = (0..num)
            .map(|_| {
                let idx_val = data.data.next("phi val idx")?;
                let idx_val = if data.other.data_basic.use_relative_ids {
                    decoded_signed_vbrs_u64(idx_val as u64) as usize
                } else {
                    idx_val
                };
                let val = data.other.get_value(
                    idx_val,
                    Some(ty.clone()),
                    "parsh phi val",
                    || {
                        Err(WError::DataNotFound("phi val ty"))
                    })?;
                let idx_block = data.data.next("phi block")?;
                Ok((
                    val,
                    idx_block,
                ))
            })
            .collect::<WResult<Vec<_>>>()?;
        let r = Self { ty, srcs };
        Ok(r)
    }
}

//! Instr Unop

use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue,
};
use crate::bitcode::blocks::typeblock::TypeBlockUnit;
use crate::meta::FastMathFlags;
use crate::{WError, WResult};

enum_parse! {
    pub(in crate::bitcode) InstrUnOpType, (usize, bool);
    Debug,;
    ;
    FNeg = (Option<FastMathFlags>; None; (0, true)),
}

impl InstrUnOpType {
    pub(in crate::bitcode) fn update(&mut self, info: usize) -> WResult<()> {
        #[allow(clippy::single_match)]
        match self {
            InstrUnOpType::FNeg(v) => {
                // fast math flags
                v.replace(FastMathFlags::parse(info)?);
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(in crate::bitcode) struct InstrUnOp {
    pub(in crate::bitcode) val: GlobalValue,
    pub(in crate::bitcode) op: InstrUnOpType,
    pub(in crate::bitcode) ty: TypeBlockUnit,
}

impl FunctionInstrTrait for InstrUnOp {
    fn res(&self) -> Option<&TypeBlockUnit> {
        Some(&self.ty)
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let val = data.next_value("Unop val", None)?;

        let ty = val.get_ty("Unop ty")?;

        let mut op = {
            let data = data.data.next("Unop op")?;
            let fp = ty.is_float() || ty.is_float_vec();
            InstrUnOpType::try_from((data, fp))?
        };

        if let Ok(info) = data.data.next("binop info") {
            op.update(info)?;
        }

        let r = Self { ty, val, op };
        Ok(r)
    }
}

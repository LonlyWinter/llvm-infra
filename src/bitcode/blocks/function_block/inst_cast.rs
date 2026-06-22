//! Instr Cast

use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue,
};
use crate::bitcode::blocks::typeblock::TypeBlockUnit;
use crate::meta::FastMathFlags;
use crate::{WError, WResult};

enum_parse! {
    pub(in crate::bitcode) InstrCastType, usize;
    Debug, PartialEq, Clone, ;
    SExt = 2,
    FpToUi = 3,
    FpToSi = 4,
    SiToFp = 6,
    PtrToInt = 9,
    IntToPtr = 10,
    BitCast = 11,
    AddrSpaceCast = 12,
    PtrToAddr = 13,
    ;
    ZExt = (bool; false; 1),
    UiToFp = (bool; false; 5),
    Trunc = ((bool, bool); (false, false); 0),
    FpTrunc = (Option<FastMathFlags>; None; 7),
    FpExt = (Option<FastMathFlags>; None; 8),

}

#[derive(Debug)]
pub(in crate::bitcode) struct InstrCast {
    pub(in crate::bitcode) op: InstrCastType,
    pub(in crate::bitcode) ty: TypeBlockUnit,
    pub(in crate::bitcode) val: GlobalValue,
}

impl FunctionInstrTrait for InstrCast {
    fn res(&self) -> Option<&TypeBlockUnit> {
        Some(&self.ty)
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let val = data.next_value("cast val", None)?;

        let ty = data.next_ty("cast ty")?;

        let mut op = {
            let op = data.data.next("Cast op")?;
            InstrCastType::try_from(op)?
        };

        if let Ok(info) = data.data.next("Cast info") {
            match &mut op {
                InstrCastType::Trunc(v) => {
                    // no signed wrap
                    v.0 = (info & 2) != 0;
                    // no unsigned wrap
                    v.1 = (info & 1) != 0;
                }
                InstrCastType::ZExt(v) | InstrCastType::UiToFp(v) => {
                    // non neg
                    *v = (info & 1) != 0;
                }
                InstrCastType::FpTrunc(v) | InstrCastType::FpExt(v) => {
                    // fast math flags
                    v.replace(FastMathFlags::parse(info)?);
                }
                _ => {}
            }
        }

        Ok(Self { op, ty, val })
    }
}

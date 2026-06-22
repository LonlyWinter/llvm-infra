//! Instr BinOP

use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue,
};
use crate::bitcode::blocks::typeblock::TypeBlockUnit;
use crate::meta::FastMathFlags;
use crate::{WError, WResult};

enum_parse! {
    pub(in crate::bitcode) InstrBinOPType, (usize, bool);
    Debug, PartialEq, Clone, ;
    FAdd = (0, true),
    FSub = (1, true),
    FMul = (2, true),
    Urem = (5, false),
    Srem = (6, false),
    And = (10, false),
    Xor = (12, false),
    ;
    // no_signed_wrap, no_unsigned_wrap
    Add = ((bool, bool); (false, false); (0, false)),
    Sub = ((bool, bool); (false, false); (1, false)),
    Mul = ((bool, bool); (false, false); (2, false)),
    Shl = ((bool, bool); (false, false); (7, false)),

    // is_exact
    Sdiv = (bool; false; (4, false)),
    Udiv = (bool; false; (3, false)),
    Lshr = (bool; false; (8, false)),
    Ashr = (bool; false; (9, false)),

    // is_disjoint
    Or = (bool; false; (11, false)),

    Fdiv = (Option<FastMathFlags>; None; (4, true)),
    Frem = (Option<FastMathFlags>; None; (6, true)),
}

impl InstrBinOPType {
    pub(in crate::bitcode) fn update(&mut self, info: usize) -> WResult<()> {
        match self {
            InstrBinOPType::Add(v)
            | InstrBinOPType::Sub(v)
            | InstrBinOPType::Mul(v)
            | InstrBinOPType::Shl(v) => {
                // no signed wrap
                v.0 = (info & 2) != 0;
                // no unsigned wrap
                v.1 = (info & 1) != 0;
            }
            InstrBinOPType::Sdiv(v)
            | InstrBinOPType::Udiv(v)
            | InstrBinOPType::Lshr(v)
            | InstrBinOPType::Ashr(v) => {
                // is exact
                *v = (info & 1) != 0;
            }
            InstrBinOPType::Or(v) => {
                // is disjoint
                *v = (info & 1) != 0;
            }
            InstrBinOPType::Fdiv(v) | InstrBinOPType::Frem(v) => {
                // fast math flags
                v.replace(FastMathFlags::parse(info)?);
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(in crate::bitcode) struct InstrBinOP {
    pub(in crate::bitcode) op: InstrBinOPType,
    pub(in crate::bitcode) ty: TypeBlockUnit,
    pub(in crate::bitcode) left: GlobalValue,
    pub(in crate::bitcode) right: GlobalValue,
}

impl FunctionInstrTrait for InstrBinOP {
    fn res(&self) -> Option<&TypeBlockUnit> {
        Some(&self.ty)
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let left = data.next_value("binop left", None)?;

        let ty = left.get_ty("binop left ty")?;
        let fp = ty.is_float() || ty.is_float_vec();

        let right = data.next_value("binop right", Some(ty.clone()))?;

        // eprintln!("binop: {:?} {:?}", left, right);

        let mut op = {
            let op = data.data.next("binop op")?;
            InstrBinOPType::try_from((op, fp))?
        };

        if let Ok(info) = data.data.next("binop info") {
            op.update(info)?;
        }

        Ok(Self {
            op,
            ty,
            left,
            right,
        })
    }
}

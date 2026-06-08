//! Instr Cmp

use std::ops::Deref;
use std::rc::Rc;

use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue, InstrDataFastMathFlags, TY_BOOL,
};
use crate::bitcode::blocks::typeblock::{TypeBlockArray, TypeBlockData, TypeBlockUnit};
use crate::{WError, WResult};

enum_parse! {
    pub(in crate::bitcode) InstrCmpFloatType, usize;
    Debug,;
    // Always false (always folded)
    False = 0,
    // True if ordered and equal
    Oeq = 1,
    // True if ordered and greater than
    Ogt = 2,
    // True if ordered and greater than or equal
    Oge = 3,
    // True if ordered and less than
    Olt = 4,
    // True if ordered and less than or equal
    Ole = 5,
    // True if ordered and operands are unequal
    One = 6,
    // True if ordered (no nans)
    Ord = 7,
    // True if unordered: isnan(X) | isnan(Y)
    Uno = 8,
    // True if unordered or equal
    Ueq = 9,
    // True if unordered or greater than
    Ugt = 10,
    // True if unordered, greater than, or equal
    Uge = 11,
    // True if unordered or less than
    Ult = 12,
    // True if unordered, less than, or equal
    Ule = 13,
    // True if unordered or not equal
    Une = 14,
    // Always true (always folded)
    True = 15,
}

enum_parse! {
    pub(in crate::bitcode) InstrCmpIntType, usize;
    Debug,;
    // equal
    Eq = 32,
    // not equal
    Ne = 33,
    // unsigned greater than
    Ugt = 34,
    // unsigned greater or equal
    Uge = 35,
    // unsigned less than
    Ult = 36,
    // unsigned less or equal
    Ule = 37,
    // signed greater than
    Sgt = 38,
    // signed greater or equal
    Sge = 39,
    // signed less than
    Slt = 40,
    // signed less or equal
    Sle = 41,
}

#[derive(Debug)]
pub(in crate::bitcode) enum InstrCmpType {
    Float(InstrCmpFloatType, InstrDataFastMathFlags),
    Int(InstrCmpIntType, bool),
}

#[derive(Debug)]
pub(in crate::bitcode) struct InstrCmp {
    pub(in crate::bitcode) op: InstrCmpType,
    pub(in crate::bitcode) ty: TypeBlockUnit,
    pub(in crate::bitcode) left: GlobalValue,
    pub(in crate::bitcode) right: GlobalValue,
}

impl FunctionInstrTrait for InstrCmp {
    fn res(&self) -> Option<&TypeBlockUnit> {
        Some(&self.ty)
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let left = data.next_value("parse cmp left val", None)?;

        let ty = left.get_ty("cmp left ty")?;

        let right = data.next_value("parse cmp right value", Some(ty.clone()))?;

        // eprintln!("cmp: {:?} {:?}", left, right);

        let op = {
            let op = data.data.next("parse cmp op")?;
            match (
                InstrCmpFloatType::try_from(op),
                InstrCmpIntType::try_from(op),
            ) {
                (Ok(v), _) => {
                    let fmf = data.data.next("parse cmp float fmf")?;
                    let fmf = InstrDataFastMathFlags::parse(fmf)?;
                    InstrCmpType::Float(v, fmf)
                }
                (_, Ok(v)) => {
                    let same_sign = data
                        .data
                        .next("parse cmp int same sign")
                        .ok()
                        .map(|v| (v & 1) != 0)
                        .unwrap_or(false);
                    InstrCmpType::Int(v, same_sign)
                }
                _ => return Err(WError::DataNotAllowed("parse cmp op", format!("code {op}"))),
            }
        };

        let binding = TY_BOOL;
        let ty_bool = binding.deref().clone();
        let ty = match ty.as_ref() {
            TypeBlockData::Array(v) => Rc::new(TypeBlockData::Array(TypeBlockArray {
                num: v.num,
                ty: ty_bool,
            })),
            TypeBlockData::Vector(v) => Rc::new(TypeBlockData::Vector(TypeBlockArray {
                num: v.num,
                ty: ty_bool,
            })),
            _ => ty_bool,
        };

        Ok(Self {
            op,
            ty,
            left,
            right,
        })
    }
}

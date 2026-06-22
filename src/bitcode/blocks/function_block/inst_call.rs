//! Instr Call

use std::collections::HashMap;

use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue, InstrCallFunc,
};
use crate::bitcode::blocks::function_record::FunctionCallingConv;
use crate::bitcode::blocks::param_attr_group::ParamAttrUnit;
use crate::bitcode::blocks::typeblock::{TypeBlockData, TypeBlockUnit};
use crate::meta::FastMathFlags;
use crate::{WError, WResult};

#[derive(Debug)]
pub(in crate::bitcode) enum InstrCallTailTy {
    Tail,
    Must,
    No,
    None,
}

#[derive(Debug)]
pub(in crate::bitcode) struct InstrCall {
    pub(in crate::bitcode) attr_list: ParamAttrUnit,
    pub(in crate::bitcode) call_tail: InstrCallTailTy,
    pub(in crate::bitcode) call_conv: FunctionCallingConv,
    pub(in crate::bitcode) fmf: Option<FastMathFlags>,
    pub(in crate::bitcode) func_name: InstrCallFunc,
    pub(in crate::bitcode) res_ty: TypeBlockUnit,
    pub(in crate::bitcode) args: Vec<GlobalValue>,
}

const BIT_CALL_TAIL: u8 = 0;
const BIT_CALL_CCONV: u8 = 1;
const BIT_CALL_MUSTTAIL: u8 = 14;
const BIT_CALL_EXPLICIT_TYPE: u8 = 15;
const BIT_CALL_NOTAIL: u8 = 16;
const BIT_CALL_FMF: u8 = 17;

impl FunctionInstrTrait for InstrCall {
    fn res(&self) -> Option<&TypeBlockUnit> {
        if matches!(self.res_ty.as_ref(), TypeBlockData::Void) {
            return None;
        }
        Some(&self.res_ty)
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let idx_attr_list = data.data.next("call attr_list")?;
        let attr_list = if idx_attr_list == 0 {
            HashMap::with_capacity(0)
        } else {
            data.other
                .param_attrs
                .get(idx_attr_list - 1)
                .cloned()
                .unwrap_or_default()
        };
        let data_cc_info = data.data.next("call cc_info")?;

        let call_tail = if ((data_cc_info >> BIT_CALL_TAIL) & 1) != 0 {
            InstrCallTailTy::Tail
        } else if ((data_cc_info >> BIT_CALL_MUSTTAIL) & 1) != 0 {
            InstrCallTailTy::Must
        } else if ((data_cc_info >> BIT_CALL_NOTAIL) & 1) != 0 {
            InstrCallTailTy::No
        } else {
            InstrCallTailTy::None
        };
        let call_conv = FunctionCallingConv::try_from(
            (0x7ff & (data_cc_info as u64)) >> BIT_CALL_CCONV
        )?;

        let fmf = if ((data_cc_info >> BIT_CALL_FMF) & 1) != 0 {
            // eprintln!("fast math flags");
            let data_now = data.data.next("call fast_math_flags")?;
            Some(FastMathFlags::parse(data_now)?)
        } else {
            None
        };
        let mut func_ty = if ((data_cc_info >> BIT_CALL_EXPLICIT_TYPE) & 1) != 0 {
            let idx_explicit_ty = data.data.next("call explicit_ty")?;
            let r = data.other.data_types.get(idx_explicit_ty).cloned().ok_or(
                WError::DataNotAllowed(
                    "call func ty",
                    format!("{}/{}", idx_explicit_ty, data.other.data_types.len()),
                ),
            )?;

            let r = match r.as_ref() {
                TypeBlockData::Function(v) => v,
                TypeBlockData::FunctionOld(v) => &v.func,
                v => {
                    return Err(WError::DataNotAllowed("call func ty", format!("{:?}", v)));
                }
            };
            Some(r.clone())
        } else {
            None
        };

        let func_name = data.next_func_name(&mut func_ty, "call func name")?;
        // eprintln!("Func name: {}", func_name);
        let func = func_ty.ok_or(WError::DataNotFound("call func ty"))?;

        // eprintln!("Call: {:?} {:?} {:?}", func_name, func, data.data);
        let mut args = func.paramty.clone().into_iter()
            .map(| ty | data.next_value(
                "call arg value",
                Some(ty)
            ))
            .collect::<WResult<Vec<_>>>()?;
        if func.vararg {
            while let Ok(arg) = data.next_value(
                "call arg value var",
                None
            ) {
                args.push(arg);
            }
        }
        if data.data.len() != 0 {
            return Err(WError::DataNotAllowed(
                "call args",
                format!("{} {:?}", data.data.len(), func)
            ));
        }
        args.shrink_to_fit();
        // eprintln!("Call: {:?} {:?}", func_name, args);
        Ok(InstrCall {
            attr_list,
            call_tail,
            call_conv,
            fmf,
            func_name,
            res_ty: func.retty.clone(),
            args,
        })
    }
}

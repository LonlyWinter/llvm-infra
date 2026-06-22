//! Instr Invoke

use std::collections::HashMap;

use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue, InstrCallFunc,
};
use crate::bitcode::blocks::function_record::FunctionCallingConv;
use crate::bitcode::blocks::param_attr_group::ParamAttrUnit;
use crate::bitcode::blocks::typeblock::{TypeBlockData, TypeBlockUnit};
use crate::{WError, WResult};

#[derive(Debug)]
pub(in crate::bitcode) struct InstrInvoke {
    pub(in crate::bitcode) attr_list: ParamAttrUnit,
    pub(in crate::bitcode) call_conv: FunctionCallingConv,
    pub(in crate::bitcode) func_name: InstrCallFunc,
    pub(in crate::bitcode) block_normal: usize,
    pub(in crate::bitcode) block_unwind: usize,
    pub(in crate::bitcode) res_ty: TypeBlockUnit,
    pub(in crate::bitcode) args: Vec<GlobalValue>,
}

const BIT_CALL_EXPLICIT_TYPE: u8 = 13;

impl FunctionInstrTrait for InstrInvoke {
    fn res(&self) -> Option<&TypeBlockUnit> {
        if matches!(self.res_ty.as_ref(), TypeBlockData::Void) {
            return None;
        }
        Some(&self.res_ty)
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let idx_attr_list = data.data.next("invoke attr_list")?;
        let attr_list = if idx_attr_list == 0 {
            HashMap::with_capacity(0)
        } else {
            data.other
                .param_attrs
                .get(idx_attr_list - 1)
                .cloned()
                .unwrap_or_default()
        };
        let data_cc_info = data.data.next("invoke cc_info")?;

        let block_normal = data.data.next("invoke block_normal")?;
        let block_unwind = data.data.next("invoke block_unwind")?;

        let call_conv = FunctionCallingConv::try_from(0x3ff & (data_cc_info as u64))?;

        let mut func_ty = if ((data_cc_info >> BIT_CALL_EXPLICIT_TYPE) & 1) != 0 {
            let idx_explicit_ty = data.data.next("invoke explicit_ty")?;
            let r = data.other.data_types.get(idx_explicit_ty).cloned().ok_or(
                WError::DataNotAllowed(
                    "invoke func ty",
                    format!("{}/{}", idx_explicit_ty, data.other.data_types.len()),
                ),
            )?;

            let r = match r.as_ref() {
                TypeBlockData::Function(v) => v,
                TypeBlockData::FunctionOld(v) => &v.func,
                v => {
                    return Err(WError::DataNotAllowed("invoke func ty", format!("{:?}", v)));
                }
            };
            Some(r.clone())
        } else {
            None
        };

        let func_name = data.next_func_name(&mut func_ty, "invoke func name")?;
        // eprintln!("Func name: {}", func_name);
        let func = func_ty.ok_or(WError::DataNotFound("invoke func ty"))?;

        // eprintln!("Call: {:?} {:?}", func_name, func);
        let mut args = func.paramty.clone().into_iter()
            .map(| ty | data.next_value(
                "invoke arg value",
                Some(ty)
            ))
            .collect::<WResult<Vec<_>>>()?;
        if func.vararg {
            while let Ok(arg) = data.next_value(
                "invoke arg value var",
                None
            ) {
                args.push(arg);
            }
        }
        if data.data.len() != 0 {
            return Err(WError::DataNotAllowed(
                "invoke args",
                format!("{} {:?}", data.data.len(), func)
            ));
        }
        args.shrink_to_fit();
        Ok(InstrInvoke {
            attr_list,
            block_normal,
            block_unwind,
            call_conv,
            func_name,
            res_ty: func.retty.clone(),
            args,
        })
    }
}

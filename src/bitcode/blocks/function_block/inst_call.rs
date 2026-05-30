//! Instr Call

use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue, InstrDataFastMathFlags,
};
use crate::bitcode::blocks::moduleblock::GlobalValueUnit;
use crate::bitcode::blocks::param_attr_group::ParamAttrList;
use crate::bitcode::blocks::typeblock::{TypeBlockData, TypeBlockUnit};
use crate::{WError, WResult};

#[derive(Debug)]
pub(in crate::bitcode) struct InstrCall {
    pub(in crate::bitcode) attr_list: Vec<ParamAttrList>,
    pub(in crate::bitcode) fast_math_flats: InstrDataFastMathFlags,
    pub(in crate::bitcode) func_name: String,
    pub(in crate::bitcode) res_ty: TypeBlockUnit,
    pub(in crate::bitcode) args: Vec<GlobalValue>,
}

const BIT_CALL_FMF: u8 = 17;
const BIT_CALL_EXPLICIT_TYPE: u8 = 15;

impl FunctionInstrTrait for InstrCall {
    fn res(&self) -> Option<&TypeBlockUnit> {
        if matches!(self.res_ty.as_ref(), TypeBlockData::Void) {
            return None;
        }
        Some(&self.res_ty)
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let idx_attr_list = data.data.next("parse call attr_list")?;
        let attr_list = data
            .other
            .param_attrs
            .get(idx_attr_list)
            .cloned()
            .unwrap_or_default();
        let data_cc_info = data.data.next("parse call cc_info")?;

        let fast_math_flats = if ((data_cc_info >> BIT_CALL_FMF) & 1) != 0 {
            // eprintln!("fast math flags");
            let data_now = data.data.next("parse call fast_math_flats")?;
            InstrDataFastMathFlags::parse(data_now)?
        } else {
            InstrDataFastMathFlags::default()
        };
        let mut func_ty = if ((data_cc_info >> BIT_CALL_EXPLICIT_TYPE) & 1) != 0 {
            let idx_explicit_ty = data.data.next("parse call explicit_ty")?;
            let r = data.other.data_types.get(idx_explicit_ty).cloned().ok_or(
                WError::DataNotAllowed(
                    "parse call func ty",
                    format!("{}/{}", idx_explicit_ty, data.other.data_types.len()),
                ),
            )?;

            let r = match r.as_ref() {
                TypeBlockData::Function(v) => v,
                TypeBlockData::FunctionOld(v) => &v.func,
                v => {
                    return Err(WError::DataNotAllowed(
                        "parse call func ty",
                        format!("{:?}", v),
                    ));
                }
            };
            Some(r.clone())
        } else {
            None
        };

        let func_name = {
            let func_name = data.next_value("parse call func name", None)?;
            match func_name.data {
                GlobalValueUnit::Data(v) => v.value_inner().try_into(),
                GlobalValueUnit::Function(f) => {
                    if func_ty.is_none() {
                        func_ty.replace(f.ty.clone());
                    }
                    Ok(f.name.clone())
                }
                _ => Err(WError::DataNotAllowed(
                    "parse call func name",
                    format!("{:?}", func_name),
                )),
            }
        }?;
        // eprintln!("Func name: {}", func_name);
        let func = func_ty.ok_or(WError::DataNotFound("parse call func ty"))?;

        let len = func.paramty.len();
        let len_all = data.data.len();
        if len_all < len {
            return Err(WError::DataNotFound("parse call args"));
        }
        let param_tys = if func.vararg {
            func.paramty
                .iter()
                .cloned()
                .map(Some)
                .chain(vec![None; len_all - len])
                .collect::<Vec<_>>()
        } else {
            func.paramty.iter().cloned().map(Some).collect::<Vec<_>>()
        };
        // eprintln!("param tys: {:?}", param_tys);
        let args = data
            .data
            .into_iter()
            .zip(param_tys)
            .map(|(idx, ty)| {
                data.other
                    .get_value(idx, ty, "parse arg value", || {
                        Err(WError::DataNotFound("parse arg ty future"))
                    })
                    .map(|(data, idx)| GlobalValue { data, idx })
            })
            .collect::<WResult<Vec<_>>>()?
            .into_iter()
            .collect();
        Ok(InstrCall {
            attr_list,
            fast_math_flats,
            func_name,
            res_ty: func.retty.clone(),
            args,
        })
    }
}

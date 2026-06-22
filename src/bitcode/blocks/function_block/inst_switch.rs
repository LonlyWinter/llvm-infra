//! Instr Switch

use crate::bitcode::blocks::constants::parse_constants_integer128;
use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue,
};
use crate::bitcode::blocks::moduleblock::GlobalValueUnit;
use crate::bitcode::blocks::typeblock::{TypeBlockData, TypeBlockUnit};
use crate::meta::InstrTermSwitchValue;
use crate::{WError, WResult};


#[derive(Debug)]
pub(in crate::bitcode) struct InstrSwitch {
    pub(in crate::bitcode) cond: GlobalValue,
    pub(in crate::bitcode) default: usize,
    /// block, val
    pub(in crate::bitcode) cases: Vec<(usize, InstrTermSwitchValue)>,
}

const SWITCH_MAGIC: usize = 1205;


impl InstrTermSwitchValue {
    fn parse(data: i128, bit_width: u16) -> WResult<Self> {
        let r = match bit_width {
            1..=8 => {
                InstrTermSwitchValue::Int8(bit_width, data as i8)
            },
            9..=16 => {
                InstrTermSwitchValue::Int16(bit_width, data as i16)
            },
            17..=32 => {
                InstrTermSwitchValue::Int32(bit_width, data as i32)
            },
            33..=64 => {
                InstrTermSwitchValue::Int64(bit_width, data as i64)
            },
            65..=128 => {
                InstrTermSwitchValue::Int128(bit_width, data)
            },
            _ => return Err(WError::DataNotAllowed(
                "Switch case val",
                format!("{} {}", data, bit_width),
            ))
        };
        Ok(r)
    }

    fn parse_arr(low: i128, high: i128, bit_width: u16) -> WResult<Vec<Self>> {
        (low..=high)
            .map(| v | Self::parse(v, bit_width))
            .collect()
    }
}

impl InstrSwitch {
    fn new_new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let ty_op = data.next_ty("Switch new op ty")?;
        let bit_width = if let TypeBlockData::Integer(n) = ty_op.as_ref()
            && *n <= 128
        {
            *n
        } else {
            return Err(WError::DataNotAllowed(
                "Switch new op ty",
                format!("{:?}", ty_op),
            ));
        };

        let cond = data.next_value("Switch new cond", Some(ty_op.clone()))?;

        let default = data.data.next("Switch new default")?;
        let num_cases = data.data.next("Switch new num cases")?;

        let cases = (0..num_cases)
            .map(|_| {
                let num_items = data.data.next("Switch new case num_items")?;
                let vals = (0..num_items)
                    .map(|_| {
                        let is_single_num = data.data.next_bool(
                            "Switch new case item is_single"
                        )?;
                        let mut low = data.data.next("Switch new case item low")? as i128;
                        if bit_width > 64 {
                            let low2 = data.data.next("Switch new case item low2")?;
                            low = parse_constants_integer128(vec![low as u64, low2 as u64])?;
                        }
                        let r = if is_single_num {
                            let low = InstrTermSwitchValue::parse(low, bit_width)?;
                            vec![low]
                        } else {
                            let mut high = data.data.next("Switch new case item high")? as i128;
                            if bit_width > 64 {
                                let high2 = data.data.next("Switch new case item high2")?;
                                high = parse_constants_integer128(vec![high as u64, high2 as u64])?;
                            }
                            InstrTermSwitchValue::parse_arr(low, high, bit_width)?
                        };
                        Ok(r)
                    })
                    .collect::<WResult<Vec<_>>>()?;
                let block = data.data.next("Switch new case block")?;
                Ok((vals, block))
            })
            .collect::<WResult<Vec<_>>>()?
            .into_iter()
            .flat_map(|(vals, block)| {
                vals.into_iter()
                    .flat_map(move |range| range.into_iter().map(move |val| (block, val)))
            })
            .collect::<Vec<_>>();
        Ok(Self {
            cond,
            default,
            cases,
        })
    }

    fn new_old<'a>(idx_ty: usize, mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let ty_op = data
            .other
            .data_types
            .get(idx_ty)
            .cloned()
            .ok_or(WError::DataNotFound("Switch old op ty"))?;
        let bit_width = if let TypeBlockData::Integer(n) = ty_op.as_ref()
            && *n <= 128
        {
            *n
        } else {
            return Err(WError::DataNotAllowed(
                "Switch old op ty",
                format!("{:?}", ty_op),
            ));
        };

        let cond = data.next_value("Switch old cond", Some(ty_op.clone()))?;

        let default = data.data.next("Switch old default")?;
        let num_cases = data.data.len() >> 1;

        let mut cases = Vec::with_capacity(num_cases);
        for _ in 0..num_cases {
            let val = {
                let idx = data.data.next("Switch old case val idx")?;
                let val = data.other.values.get(idx);
                if let Some(GlobalValueUnit::Data(v)) = val {
                    let data_temp = v.convert_i128()?;
                    // eprintln!("Case {} {:?}", data_temp, v);
                    InstrTermSwitchValue::parse(data_temp, bit_width)?
                } else {
                    return Err(WError::DataNotAllowed(
                        "Switch old case val",
                        format!("{} {:?}", idx, val),
                    ));
                }
            };
            let block = data.data.next("Switch old case block")?;
            cases.push((block, val));
        }
        Ok(Self {
            cond,
            default,
            cases,
        })
    }
}

impl FunctionInstrTrait for InstrSwitch {
    fn res(&self) -> Option<&TypeBlockUnit> {
        None
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let magic = data.data.next("Switch magic")?;
        if (magic >> 16) == SWITCH_MAGIC {
            Self::new_new(data)
        } else {
            Self::new_old(magic, data)
        }
    }
}

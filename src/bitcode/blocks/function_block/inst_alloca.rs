//! Instr Alloca

use std::rc::Rc;

use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait,
};
use crate::bitcode::blocks::moduleblock::GlobalValueUnit;
use crate::bitcode::blocks::typeblock::{TypeBlockData, TypeBlockPointer, TypeBlockUnit};
use crate::bitcode::entry::parse_alignment;
use crate::{WError, WResult};

#[derive(Debug)]
pub(in crate::bitcode) struct InstrAlloca {
    pub(in crate::bitcode) ty_inst: TypeBlockUnit,
    pub(in crate::bitcode) ty_op: TypeBlockUnit,
    pub(in crate::bitcode) size: usize,
    pub(in crate::bitcode) align: u16,
    pub(in crate::bitcode) used_within_alloca: bool,
    pub(in crate::bitcode) explicit_type: bool,
    pub(in crate::bitcode) swift_error: bool,
    pub(in crate::bitcode) address_space: u16,
}

impl FunctionInstrTrait for InstrAlloca {
    fn res(&self) -> Option<&TypeBlockUnit> {
        Some(&self.ty_inst)
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        if data.data.len() < 4 {
            return Err(WError::DataNotFound("alloca"));
        }
        let ty_inst = Rc::new(TypeBlockData::Pointer(TypeBlockPointer {
            ty: data.next_ty("alloca inst ty")?,
            space: 0,
        }));
        let ty_op = data.next_ty("alloca op ty")?;
        let idx_size = data.data.next("alloca size idx")?;
        let size = data
            .other
            .values
            .get(idx_size)
            .cloned()
            .and_then(|v| {
                if let GlobalValueUnit::Data(v) = v {
                    Some(v.convert_i128().map(| v | v as usize))
                } else {
                    None
                }
            })
            .ok_or(WError::DataNotFound("alloca size"))??;
        let idx_align_flags = data.data.next("alloca align flags")?;
        let align = {
            let low5bit = (idx_align_flags as u8) & 31;
            let high3bit = ((idx_align_flags >> 8) as u8) & 7;
            let t = (high3bit << 5) | low5bit;
            parse_alignment(t - 1)
        };
        let used_within_alloca = (idx_align_flags & 32) != 0;
        let explicit_type = (idx_align_flags & 63) != 0;
        let swift_error = (idx_align_flags & 128) != 0;
        let address_space = if let Ok(v) = data.data.next("alloca inst ty idx") {
            v as u16
        } else {
            data.other.data_basic.data_layout.address_space_stack_alloca
        };
        Ok(Self {
            ty_inst,
            ty_op,
            size,
            align,
            used_within_alloca,
            explicit_type,
            swift_error,
            address_space,
        })
    }
}

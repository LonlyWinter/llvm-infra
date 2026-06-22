//! Instr AtomicRMW

use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue, InstrAtomicOrdering,
};
use crate::bitcode::blocks::sync_scope_names::SyncScopeName;
use crate::bitcode::blocks::typeblock::{TypeBlockData, TypeBlockUnit};
use crate::{WError, WResult};

enum_parse! {
    pub(in crate::bitcode) InstrAtomicRMWOp, usize;
    Debug,;
    Xchg = 0,
    Add = 1,
    Sub = 2,
    And = 3,
    Nand = 4,
    Or = 5,
    Xor = 6,
    Max = 7,
    Min = 8,
    Umax = 9,
    Umin = 10,
    Fadd = 11,
    Fsub = 12,
    Fmax = 13,
    Fmin = 14,
    UincWrap = 15,
    UdecWrap = 16,
    UsubCond = 17,
    UsubSat = 18,
    Fmaximum = 19,
    Fminimum = 20,
}

#[derive(Debug)]
pub(in crate::bitcode) struct InstrAtomicRMW {
    pub(in crate::bitcode) ty: TypeBlockUnit,
    pub(in crate::bitcode) ptr: GlobalValue,
    pub(in crate::bitcode) val: GlobalValue,
    pub(in crate::bitcode) op: InstrAtomicRMWOp,
    pub(in crate::bitcode) ordering: InstrAtomicOrdering,
    pub(in crate::bitcode) ssid: SyncScopeName,
    pub(in crate::bitcode) is_volatile: bool,
    pub(in crate::bitcode) align: u16,
}

impl FunctionInstrTrait for InstrAtomicRMW {
    fn res(&self) -> Option<&TypeBlockUnit> {
        Some(&self.ty)
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let ptr = data.next_value("AtomicRMW ptr", None)?;
        let ty_ptr = ptr.get_ty("AtomicRMW ptr ty")?;
        if !matches!(
            ty_ptr.as_ref(),
            TypeBlockData::Pointer(_) | TypeBlockData::OpaquePointer(_)
        ) {
            return Err(WError::DataNotAllowed(
                "check AtomicRMW ptr ty",
                format!("{:?}", ty_ptr),
            ));
        }

        let val = data.next_value("AtomicRMW val", None)?;

        let ty = val.get_ty("AtomicRMW ty")?;

        let op = {
            let data = data.data.next("AtomicRMW op")?;
            InstrAtomicRMWOp::try_from(data)?
        };

        let is_volatile = data.data.next_bool("AtomicRMW is_volatile")?;

        let ordering = {
            let data = data.data.next("AtomicRMW ordering")?;
            InstrAtomicOrdering::try_from(data)?
        };
        if matches!(
            ordering,
            InstrAtomicOrdering::NotAtomic | InstrAtomicOrdering::Unordered
        ) {
            return Err(WError::DataNotAllowed(
                "check AtomicRMW ordering",
                format!("{:?}", ordering),
            ));
        }

        let ssid = {
            let idx = data.data.next("AtomicRMW ssid")?;
            data.other.sync_scope_names.parse(idx)?
        };

        let align = data.next_alignment();

        let r = Self {
            ty,
            ptr,
            val,
            op,
            ordering,
            ssid,
            align,
            is_volatile,
        };
        Ok(r)
    }
}

//! Instr Cmpxchg

use std::ops::Deref;
use std::rc::Rc;

use crate::bitcode::blocks::function_block::utils::InstrAtomicOrdering;
use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue, TY_BOOL,
};
use crate::bitcode::blocks::sync_scope_names::SyncScopeName;
use crate::bitcode::blocks::typeblock::{TypeBlockData, TypeBlockStructAnon, TypeBlockUnit};
use crate::{WError, WResult};

#[derive(Debug)]
pub(in crate::bitcode) struct InstrCmpxchg {
    pub(in crate::bitcode) ptr: GlobalValue,
    pub(in crate::bitcode) cmp: GlobalValue,
    pub(in crate::bitcode) val: GlobalValue,
    pub(in crate::bitcode) ty: TypeBlockUnit,
    pub(in crate::bitcode) success_ordering: InstrAtomicOrdering,
    pub(in crate::bitcode) failure_ordering: InstrAtomicOrdering,
    pub(in crate::bitcode) ssid: SyncScopeName,
    pub(in crate::bitcode) is_volatile: bool,
    pub(in crate::bitcode) is_weak: bool,
    pub(in crate::bitcode) align: u16,
}

impl FunctionInstrTrait for InstrCmpxchg {
    fn res(&self) -> Option<&TypeBlockUnit> {
        Some(&self.ty)
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let ptr = data.next_value("Cmpxchg ptr", None)?;

        let cmp = data.next_value("Cmpxchg cmp", None)?;
        let ty = cmp.get_ty("get Cmpxchg cmp ty")?;
        let val = data.next_value("Cmpxchg val", Some(ty.clone()))?;
        let ty = Rc::new(TypeBlockData::StructAnon(TypeBlockStructAnon {
            is_packed: false,
            tys: vec![ty, TY_BOOL.deref().clone()],
        }));

        let is_volatile = data.data.next_bool("Cmpxchg is_volatile")?;

        let success_ordering = {
            let data = data.data.next("Cmpxchg success_ordering")?;
            InstrAtomicOrdering::try_from(data)?
        };
        if matches!(
            success_ordering,
            InstrAtomicOrdering::NotAtomic | InstrAtomicOrdering::Unordered
        ) {
            return Err(WError::DataNotAllowed(
                "check Cmpxchg success_ordering",
                format!("{:?}", success_ordering),
            ));
        }

        let ssid = {
            let idx = data.data.next("Cmpxchg ssid")?;
            data.other.sync_scope_names.parse(idx)?
        };

        let failure_ordering = {
            let data = data.data.next("Cmpxchg failure_ordering")?;
            InstrAtomicOrdering::try_from(data)?
        };
        if matches!(
            failure_ordering,
            InstrAtomicOrdering::NotAtomic
                | InstrAtomicOrdering::Unordered
                | InstrAtomicOrdering::Release
                | InstrAtomicOrdering::AcquireRelease
        ) {
            return Err(WError::DataNotAllowed(
                "check Cmpxchg failure_ordering",
                format!("{:?}", failure_ordering),
            ));
        }
        
        let is_week = data.data.next_bool("Cmpxchg is_week")?;

        let align = data.next_alignment();

        let r = Self {
            ptr,
            cmp,
            val,
            ty,
            success_ordering,
            failure_ordering,
            ssid,
            align,
            is_volatile,
            is_weak: is_week,
        };
        Ok(r)
    }
}

//! Instr LoadAtomic

use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue, InstrAtomicOrdering,
};
use crate::bitcode::blocks::sync_scope_names::SyncScopeName;
use crate::bitcode::blocks::typeblock::TypeBlockUnit;
use crate::{WError, WResult};

#[derive(Debug)]
pub(in crate::bitcode) struct InstrLoadAtomic {
    pub(in crate::bitcode) ty: TypeBlockUnit,
    pub(in crate::bitcode) op: GlobalValue,
    pub(in crate::bitcode) ordering: InstrAtomicOrdering,
    pub(in crate::bitcode) ssid: SyncScopeName,
    pub(in crate::bitcode) is_volatile: bool,
    pub(in crate::bitcode) align: u16,
}

impl FunctionInstrTrait for InstrLoadAtomic {
    fn res(&self) -> Option<&TypeBlockUnit> {
        Some(&self.ty)
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let op = data.next_value("LoadAtomic op", None)?;

        let ty = data.next_ty("LoadAtomic ty")?;

        let align = data.data.next_alignment("LoadAtomic align")?;
        let is_volatile = data.data.next_bool("LoadAtomic is_volatile")?;

        let ordering = {
            let data = data.data.next("LoadAtomic ordering")?;
            InstrAtomicOrdering::try_from(data)?
        };
        if matches!(
            ordering,
            InstrAtomicOrdering::NotAtomic
                | InstrAtomicOrdering::Release
                | InstrAtomicOrdering::AcquireRelease
        ) {
            return Err(WError::DataNotAllowed(
                "check LoadAtomic ordering",
                format!("{:?}", ordering),
            ));
        }

        let ssid = {
            let idx = data.data.next("LoadAtomic ssid")?;
            data.other.sync_scope_names.parse(idx)?
        };

        let r = Self {
            op,
            ty,
            ordering,
            ssid,
            align,
            is_volatile,
        };
        Ok(r)
    }
}

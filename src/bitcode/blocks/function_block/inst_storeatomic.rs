//! Instr StoreAtomic

use crate::bitcode::blocks::function_block::utils::InstrAtomicOrdering;
use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue,
};
use crate::bitcode::blocks::sync_scope_names::SyncScopeName;
use crate::bitcode::blocks::typeblock::TypeBlockUnit;
use crate::{WError, WResult};

#[derive(Debug)]
pub(in crate::bitcode) struct InstrStoreAtomic {
    pub(in crate::bitcode) ptr: GlobalValue,
    pub(in crate::bitcode) val: GlobalValue,
    pub(in crate::bitcode) ordering: InstrAtomicOrdering,
    pub(in crate::bitcode) ssid: SyncScopeName,
    pub(in crate::bitcode) is_volatile: bool,
    pub(in crate::bitcode) align: u16,
}

impl FunctionInstrTrait for InstrStoreAtomic {
    fn res(&self) -> Option<&TypeBlockUnit> {
        None
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let ptr = data.next_value("StoreAtomic ptr", None)?;

        let val = data.next_value("StoreAtomic val", None)?;

        let align = data.data.next_alignment("StoreAtomic align")?;
        let is_volatile = data.data.next_bool("StoreAtomic is_volatile")?;

        let ordering = {
            let data = data.data.next("StoreAtomic ordering")?;
            InstrAtomicOrdering::try_from(data)?
        };
        if matches!(
            ordering,
            InstrAtomicOrdering::NotAtomic
                | InstrAtomicOrdering::Acquire
                | InstrAtomicOrdering::AcquireRelease
        ) {
            return Err(WError::DataNotAllowed(
                "check StoreAtomic ordering",
                format!("{:?}", ordering),
            ));
        }

        let ssid = {
            let idx = data.data.next("StoreAtomic ssid")?;
            data.other.sync_scope_names.parse(idx)?
        };

        let r = Self {
            ptr,
            val,
            ordering,
            ssid,
            align,
            is_volatile,
        };
        Ok(r)
    }
}

//! Instr Fence

use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, InstrAtomicOrdering,
};
use crate::bitcode::blocks::sync_scope_names::SyncScopeName;
use crate::bitcode::blocks::typeblock::TypeBlockUnit;
use crate::{WError, WResult};

#[derive(Debug)]
pub(in crate::bitcode) struct InstrFence {
    pub(in crate::bitcode) ordering: InstrAtomicOrdering,
    pub(in crate::bitcode) ssid: SyncScopeName,
}

impl FunctionInstrTrait for InstrFence {
    fn res(&self) -> Option<&TypeBlockUnit> {
        None
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        if data.data.len() != 2 {
            return Err(WError::DataNotFound("fence data"));
        }
        let ordering = {
            let data = data.data.next("fence ordering")?;
            InstrAtomicOrdering::try_from(data)?
        };
        if matches!(
            ordering,
            InstrAtomicOrdering::NotAtomic
                | InstrAtomicOrdering::Unordered
                | InstrAtomicOrdering::Monotonic
        ) {
            return Err(WError::DataNotAllowed(
                "check fence ordering",
                format!("{:?}", ordering),
            ));
        }

        let ssid = {
            let idx = data.data.next("fence ssid")?;
            data.other.sync_scope_names.parse(idx)?
        };

        let r = Self { ordering, ssid };
        Ok(r)
    }
}

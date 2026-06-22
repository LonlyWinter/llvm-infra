//! Instr LandingPad

use crate::bitcode::blocks::function_block::utils::{
    FunctionInstrData, FunctionInstrTrait, GlobalValue,
};
use crate::bitcode::blocks::typeblock::{TypeBlockData, TypeBlockUnit};
use crate::{WError, WResult};

enum_parse! {
    pub(in crate::bitcode) InstrLandingPadClauseType, usize;
    Debug, PartialEq, Clone, ;
    Catch = 0,
    Filter = 1,
}

#[derive(Debug)]
pub(in crate::bitcode) struct InstrLandingPadClause {
    pub(in crate::bitcode) ty: InstrLandingPadClauseType,
    pub(in crate::bitcode) val: GlobalValue,
}

#[derive(Debug)]
pub(in crate::bitcode) struct InstrLandingPad {
    pub(in crate::bitcode) clauses: Vec<InstrLandingPadClause>,
    pub(in crate::bitcode) is_cleanup: bool,
    pub(in crate::bitcode) ty: TypeBlockUnit,
}

impl FunctionInstrTrait for InstrLandingPad {
    fn res(&self) -> Option<&TypeBlockUnit> {
        Some(&self.ty)
    }

    fn new<'a>(mut data: FunctionInstrData<'a>) -> WResult<Self> {
        let ty = data.next_ty("LandingPad ty")?;
        let is_cleanup = data.data.next_bool("LandingPad is_cleanup")?;
        let num_clauses = data.data.next("LandingPad num_clauses")?;
        let clauses = (0..num_clauses)
            .map(|_| {
                let ty = {
                    let data = data.data.next("LandingPad clause ty")?;
                    InstrLandingPadClauseType::try_from(data)?
                };
                let val = data.next_value("LandingPad clause val", None)?;

                let ty_val = val.get_ty("clause val ty")?;
                if (ty == InstrLandingPadClauseType::Catch
                    && matches!(ty_val.as_ref(), TypeBlockData::Array(_)))
                    || (ty == InstrLandingPadClauseType::Filter
                        && !matches!(ty_val.as_ref(), TypeBlockData::Array(_)))
                {
                    return Err(WError::DataNotAllowed(
                        "LandingPad caluse",
                        format!("{:?} {:?}", ty, val),
                    ));
                }

                Ok(InstrLandingPadClause { ty, val })
            })
            .collect::<WResult<Vec<_>>>()?;

        let r = Self {
            clauses,
            is_cleanup,
            ty,
        };
        Ok(r)
    }
}

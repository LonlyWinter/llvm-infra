//! ParamAttrBlock
//! https://llvm.org/docs/BitCodeFormat.html#paramattr-block

use std::io::{Read, Seek};

use crate::bitcode::blocks::blockinfo::BlockInfoAll;
use crate::bitcode::blocks::param_attr_group::ParamAttrGroupBlock;
use crate::bitcode::entry::BitCodeEntryRecord;
use crate::bitcode::parse::StateBlock;
use crate::{WError, WResult};

use crate::bitreader::BitReader;

use crate::bitcode::blocks::param_attr_group::ParamAttrList;

const PARAMATTR_CODE_ENTRY_OLD: u32 = 1;
const PARAMATTR_CODE_ENTRY: u32 = 2;

pub(in crate::bitcode) type ParamAttrBlock = Vec<Vec<ParamAttrList>>;

fn parse_param_attr(
    code: u32,
    data: Vec<usize>,
    param_attr_group: &ParamAttrGroupBlock,
) -> WResult<Vec<ParamAttrList>> {
    let res = match code {
        PARAMATTR_CODE_ENTRY => data
            .into_iter()
            .map(|v| {
                param_attr_group
                    .get(&v)
                    .cloned()
                    .ok_or(WError::CodeNotAllowed("parse param attr entry", v as u32))
            })
            .collect::<WResult<Vec<_>>>()?
            .into_iter()
            .collect::<Vec<_>>(),
        PARAMATTR_CODE_ENTRY_OLD => return Err(WError::Deprecated("ParamAttr block old entry")),
        _ => return Err(WError::CodeNotAllowed("ParamAttr block record", code)),
    };
    Ok(res)
}

impl<F: Read + Seek> BitReader<F> {
    pub(in crate::bitcode) fn parse_param_attr_block(
        &mut self,
        mut state: StateBlock,
        block_infos: &BlockInfoAll,
        param_attr_group: &ParamAttrGroupBlock,
    ) -> WResult<ParamAttrBlock> {
        // eprintln!("parse entry in param_attr block ... {}", res);
        let mut res = Vec::with_capacity(8);
        let mut parse_param_attr_now = |data: BitCodeEntryRecord<usize>| {
            let t = parse_param_attr(data.code, data.data, param_attr_group)?;
            res.push(t);
            Ok(())
        };
        self.parse_entry_only_record(
            &mut state,
            block_infos,
            "ParamAttr Block",
            &mut parse_param_attr_now,
        )?;
        res.shrink_to_fit();
        Ok(res)
    }
}

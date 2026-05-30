//! ValueSymTab Block
//! https://llvm.org/docs/BitCodeFormat.html#value-symtab-block

use std::collections::HashMap;
use std::io::{Read, Seek};
use std::rc::Rc;

use crate::bitcode::blocks::blockinfo::BlockInfoAll;
use crate::bitcode::blocks::function_record::Function;
use crate::bitcode::blocks::moduleblock::{GlobalValueList, GlobalValueUnit};
use crate::bitcode::entry::BitCodeEntryRecord;
use crate::bitcode::parse::StateBlock;
use crate::{WError, WResult};

use crate::bitreader::BitReader;

const CODE_ENTRY: u32 = 1; // [valueid, namechar x N]
const CODE_BBENTRY: u32 = 2; // [bbid, namechar x N]
const CODE_FNENTRY: u32 = 3; // [valueid, offset, namechar x N]
// const CODE_COMBINED_ENTRY: u32 = 5; // [valueid, refguid]

pub(in crate::bitcode) type ValueSymTabUnit = HashMap<usize, String>;

#[derive(Debug, Default)]
pub(in crate::bitcode) struct ValueSymTabBlock {
    pub(in crate::bitcode) value: ValueSymTabUnit,
    pub(in crate::bitcode) basic_block: ValueSymTabUnit,
}

impl<F: Read + Seek> BitReader<F> {
    pub(in crate::bitcode) fn parse_value_symtab_block_skip(
        &mut self,
        mut state: StateBlock,
        block_infos: &BlockInfoAll,
        use_strtab: bool,
        value_list: &GlobalValueList,
    ) -> WResult<Vec<Rc<Function>>> {
        if !use_strtab {
            return Ok(Vec::with_capacity(0));
        }
        let mut res = Vec::with_capacity(16);
        let parse_value_symtab_block_record = |data: BitCodeEntryRecord<usize>| {
            if data.code != CODE_FNENTRY {
                return Err(WError::CodeNotAllowed(
                    "parse_valye_symtab_block",
                    data.code,
                ));
            }
            if data.data.len() != 2 {
                return Err(WError::DataNotAllowed(
                    "parse value symtab block",
                    format!("{:?}", data.data),
                ));
            }
            let idx = data.data[0];
            if let GlobalValueUnit::Function(v) = &value_list[idx] {
                res.push((data.data[1], v.clone()));
                return Ok(());
            }
            Err(WError::DataNotAllowed(
                "parse value symtab block",
                format!("{:?}", value_list[idx]),
            ))
        };

        self.parse_entry_only_record(
            &mut state,
            block_infos,
            "ValueSymTab Block",
            parse_value_symtab_block_record,
        )?;
        res.sort_by_key(|v| v.0);
        res.reverse();
        let res = res.into_iter().map(|v| v.1).collect();
        Ok(res)
    }

    pub(in crate::bitcode) fn parse_value_symtab_block_normal(
        &mut self,
        mut state: StateBlock,
        block_infos: &BlockInfoAll,
        value_list: &mut GlobalValueList,
    ) -> WResult<ValueSymTabBlock> {
        let mut res_value = HashMap::with_capacity(64);
        let mut res_bb = HashMap::with_capacity(64);
        let mut parse_value_symtab_block_record = |data: BitCodeEntryRecord<u16>| {
            match data.code {
                CODE_ENTRY => {
                    if data.data.is_empty() {
                        return Err(WError::DataNotFound("parse_valye_symtab_block ENTRY"));
                    }
                    let idx = data.data[0] as usize;
                    if !matches!(&value_list[idx], GlobalValueUnit::Instr(_, _, _)) {
                        return Err(WError::DataNotAllowed(
                            "parse_valye_symtab_block ENTRY",
                            format!("{:?}", value_list[idx]),
                        ));
                    }
                    // eprintln!("value: {} {:?} {:?}", idx, data.chars, data.blob);
                    res_value.insert(idx, data.chars);
                },
                CODE_BBENTRY => {
                    if data.data.len() != 1 {
                        return Err(WError::DataNotFound("parse_valye_symtab_block BBENTRY"));
                    }
                    let idx = data.data[0] as usize;
                    res_bb.insert(idx, data.chars);
                },
                CODE_FNENTRY => {},
                i => return Err(WError::CodeNotAllowed("parse_valye_symtab_block", i)),
            };
            Ok(())
        };

        self.parse_entry_only_record(
            &mut state,
            block_infos,
            "ValueSymTab Block",
            &mut parse_value_symtab_block_record,
        )?;
        res_value.shrink_to_fit();
        res_bb.shrink_to_fit();
        let res = ValueSymTabBlock {
            value: res_value,
            basic_block: res_bb,
        };
        Ok(res)
    }
}

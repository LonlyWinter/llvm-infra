//! block info
//! https://llvm.org/docs/BitCodeFormat.html#blockinfo-block

use std::{
    collections::HashMap,
    io::{Read, Seek},
    mem::take,
};

use crate::{WError, WResult, bitcode::parse::StateBlock};

use crate::filebit::BitReader;

use crate::bitcode::{abbrev::Abbrev, entry::BitCodeEntry};

#[derive(Debug)]
pub(in crate::bitcode) struct BlockInfo {
    pub(in crate::bitcode) name: String,
    pub(in crate::bitcode) abbrevs: Vec<Abbrev>,
    pub(in crate::bitcode) record_name: HashMap<usize, String>,
}

enum_parse! {
    BlockInfoContents, u32;
    Debug,;
    SetBid = 1,
    BlockName = 2,
    SetRecordName = 3,
}

pub(in crate::bitcode) type BlockInfoAll = HashMap<usize, BlockInfo>;

impl<F: Read + Seek> BitReader<F> {
    pub(in crate::bitcode) fn parse_block_info(
        &mut self,
        mut state: StateBlock,
        block_infos: &BlockInfoAll,
    ) -> WResult<BlockInfoAll> {
        // todo!("parse block info: {}", state.get_current_code_size());
        let mut block_id = usize::MAX;
        let mut name = String::new();
        let mut record_name = HashMap::with_capacity(16);
        let mut res = HashMap::with_capacity(16);
        while self.has_data() {
            let entry = self.parse_entry::<usize>(&mut state, block_infos)?;
            match entry {
                BitCodeEntry::Record(record) => {
                    let block_info_id = BlockInfoContents::try_from(record.code)?;
                    // eprintln!(
                    //     "parse in block info ... {} {:?} {:?}",
                    //     record.code, block_info_id, record.data
                    // );
                    match block_info_id {
                        BlockInfoContents::SetBid => {
                            if block_id != usize::MAX {
                                let abbrevs_now = take(&mut state.abbrevs);
                                let record_now = take(&mut record_name);
                                let name_now = take(&mut name);
                                // eprintln!("BlockInfo: {} {}\n{:?}\n{:?}", block_id, name_now, abbrevs_now, record_now);
                                res.insert(
                                    block_id,
                                    BlockInfo {
                                        name: name_now,
                                        abbrevs: abbrevs_now,
                                        record_name: record_now,
                                    },
                                );
                            }
                            block_id = record.data[0];
                        }
                        BlockInfoContents::BlockName => {
                            if !name.is_empty() {
                                return Err(WError::DataSetAlready("BlockInfo name"));
                            }
                            name = record.chars;
                        }
                        BlockInfoContents::SetRecordName => {
                            record_name.insert(record.data[0], record.chars);
                        }
                    }
                }
                BitCodeEntry::SubBlock(id, _) => {
                    return Err(WError::BlockNotAllowed("BlockInfo", id));
                }
                BitCodeEntry::EndBlock => break,
            }
        }
        // eprintln!("parse entry read in block info ...");
        let abbrevs = take(&mut state.abbrevs);
        res.insert(
            block_id,
            BlockInfo {
                name,
                abbrevs,
                record_name,
            },
        );
        res.shrink_to_fit();
        Ok(res)
    }
}

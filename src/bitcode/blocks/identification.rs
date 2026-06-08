//! Identification Block

use std::io::{Read, Seek};

use crate::bitcode::blocks::blockinfo::BlockInfoAll;
use crate::bitcode::parse::StateBlock;
use crate::{WError, WResult};

use crate::filebit::BitReader;

use crate::bitcode::entry::BitCodeEntryRecord;

impl<F: Read + Seek> BitReader<F> {
    pub(in crate::bitcode) fn parse_identification_block(
        &mut self,
        mut state: StateBlock,
        block_infos: &BlockInfoAll,
    ) -> WResult<String> {
        // todo!("parse identification block");
        let mut res = String::with_capacity(0);
        let mut parse_identification_block = |data: BitCodeEntryRecord<u8>| {
            match data.code {
                1 => {
                    res = data.chars;
                }
                2 if let Some(0) = data.data.first() => {}
                2 => return Err(WError::Version(data.data.first().cloned())),
                _ => return Err(WError::CodeNotAllowed("Identification record", data.code)),
            }
            Ok(())
        };
        self.parse_entry_only_record(
            &mut state,
            block_infos,
            "IdentificationBlock",
            &mut parse_identification_block,
        )?;
        res.shrink_to_fit();
        if res.is_empty() {
            return Err(WError::DataNotFound("info in identification block"));
        }
        Ok(res)
    }
}

//! StrTab Block

use std::io::{Read, Seek};

use crate::bitcode::blocks::blockinfo::BlockInfoAll;
use crate::bitcode::entry::BitCodeEntryRecord;
use crate::bitcode::parse::StateBlock;
use crate::{WError, WResult};

use crate::filebit::BitReader;

const STRTAB_BLOB: u32 = 1;

impl<F: Read + Seek> BitReader<F> {
    pub(in crate::bitcode) fn parse_str_sym_table_block(
        &mut self,
        mut state: StateBlock,
        block_infos: &BlockInfoAll,
    ) -> WResult<Vec<u8>> {
        let mut res = Vec::with_capacity(0);
        let mut parse_str_sym_table = |data: BitCodeEntryRecord<u8>| {
            if data.code != STRTAB_BLOB {
                return Err(WError::CodeNotAllowed("StrTab block record", data.code));
            }
            if data.blob.is_empty() {
                return Err(WError::DataNotFound("Str/Sym Table Block blob"));
            }
            if res.is_empty() {
                res = data.blob;
            }
            Ok(())
        };
        self.parse_entry_only_record(
            &mut state,
            block_infos,
            "UseList Block",
            &mut parse_str_sym_table,
        )?;
        res.shrink_to_fit();
        Ok(res)
    }
}

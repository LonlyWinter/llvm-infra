//! AppSpec/Unknown Block

use std::io::{Read, Seek};

use crate::WResult;
use crate::bitcode::blocks::blockinfo::BlockInfoAll;
use crate::bitcode::entry::BitCodeEntryRecord;
use crate::bitcode::parse::StateBlock;

use crate::filebit::BitReader;

fn parse_app_spec_block(_data: BitCodeEntryRecord<u32>) -> WResult<()> {
    todo!("app_spec_block");
}

impl<F: Read + Seek> BitReader<F> {
    pub(in crate::bitcode) fn parse_app_spec_block(
        &mut self,
        mut state: StateBlock,
        block_infos: &BlockInfoAll,
        _app_id: u8,
    ) -> WResult<()> {
        self.parse_entry_only_record(
            &mut state,
            block_infos,
            "AppSpec Block",
            parse_app_spec_block,
        )?;
        Ok(())
    }
}

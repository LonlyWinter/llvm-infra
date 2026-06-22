//! UseList Block

use std::io::{Read, Seek};

use crate::WResult;
use crate::bitcode::blocks::blockinfo::BlockInfoAll;
// use crate::bitcode::entry::BitCodeEntryRecord;
use crate::bitcode::parse::StateBlock;

use crate::filebit::BitReader;

// fn parse_uselist_block(_data: BitCodeEntryRecord<u64>) -> WResult<()> {
//     todo!("uselist_block");
// }

impl<F: Read + Seek> BitReader<F> {
    pub(in crate::bitcode) fn parse_uselist_block(
        &mut self,
        state: StateBlock,
        _block_infos: &BlockInfoAll,
    ) -> WResult<()> {
        self.skip_bits(state.block_size)?;
        // let res = HashMap::with_capacity(1024);
        // let mut parse_uselist_block = |_data: BitCodeEntryRecord<usize>| {
        //     // todo!("uselist_block");
        //     Ok(())
        // };

        // self.parse_entry_only_record(
        //     &mut state,
        //     block_infos,
        //     "UseList Block",
        //     &mut parse_uselist_block,
        // )?;
        Ok(())
    }
}

//! OperandBundleTags Block

use std::io::{Read, Seek};
use std::rc::Rc;

use crate::bitcode::blocks::blockinfo::BlockInfoAll;
use crate::bitcode::entry::BitCodeEntryRecord;
use crate::bitcode::parse::StateBlock;
use crate::{WError, WResult};

use crate::bitreader::BitReader;

const OPERAND_BUNDLE_TAG: u32 = 1;

fn parse_operand_bundle_tags_block(code: u32, data: Vec<u8>) -> WResult<String> {
    if code != OPERAND_BUNDLE_TAG {
        return Err(WError::CodeNotAllowed("OperandBundleTag", code));
    }
    let tag = data.into_iter().map(|v| v as char).collect::<String>();
    Ok(tag)
}

impl<F: Read + Seek> BitReader<F> {
    pub(in crate::bitcode) fn parse_operand_bundle_tags_block(
        &mut self,
        mut state: StateBlock,
        block_infos: &BlockInfoAll,
    ) -> WResult<Vec<Rc<String>>> {
        let mut res = Vec::with_capacity(64);
        let mut parse_operand_bundle_tags_block_now = |data: BitCodeEntryRecord<u8>| {
            let t = parse_operand_bundle_tags_block(data.code, data.data)?;
            res.push(Rc::new(t));
            Ok(())
        };
        self.parse_entry_only_record(
            &mut state,
            block_infos,
            "OperandBundleTags Block",
            &mut parse_operand_bundle_tags_block_now,
        )?;
        res.shrink_to_fit();
        Ok(res)
    }
}

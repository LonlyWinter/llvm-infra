//! MetaData Attachment Block

use std::collections::HashSet;
use std::io::{Read, Seek};

use crate::WResult;
use crate::bitcode::blocks::blockinfo::BlockInfoAll;
use crate::bitcode::blocks::metadata::{MetaData, MetaDataValue};
use crate::bitcode::blocks::metadata_kind::{MetaDataKindBlock, MetaDataKindMap};
use crate::bitcode::entry::BitCodeEntryRecord;
use crate::bitcode::parse::StateBlock;
use crate::bitreader::BitReader;

pub(in crate::bitcode) enum MetaDataAttachmentBlock {
    /// idx_instr, metadata
    Instr(usize, Vec<MetaDataValue>),
}

fn parse_metadata_attachment_block_function(_data: BitCodeEntryRecord<usize>) -> WResult<()> {
    todo!("parse metadata_attachment_block");
}

fn parse_metadata_attachment_block_instr(
    data: BitCodeEntryRecord<usize>,
    metadata_list: &MetaData,
    metadata_kind: &MetaDataKindBlock,
) -> WResult<(usize, Vec<MetaDataValue>)> {
    let idx_res = data.data[0];
    let mut res = Vec::with_capacity(64);
    let mut data = data.data.into_iter().skip(1);
    while let Some(kind) = data.next()
        && let Some(idx) = data.next()
    {
        let kind = metadata_kind
            .iter()
            .filter(|&v| v.0 == kind)
            .map(|v| &v.1)
            .collect::<HashSet<_>>();
        if kind.contains(&MetaDataKindMap::Tbaa) {
            continue;
        }
        let meta = metadata_list.get_value(idx)?;
        if let MetaDataValue::Node(vs) = meta {
            res.extend(vs);
        }
    }

    res.shrink_to_fit();
    Ok((idx_res, res))
}

const METADATA_ATTACHMENT: u32 = 11;

impl<F: Read + Seek> BitReader<F> {
    pub(in crate::bitcode) fn parse_metadata_attachment_block(
        &mut self,
        state: StateBlock,
        _block_infos: &BlockInfoAll,
        _metadata_list: &MetaData,
        _metadata_kind: &MetaDataKindBlock,
    ) -> WResult<()> {
        self.skip_bits(state.block_size)?;
        // let mut res = Vec::with_capacity(128);
        // let mut parse_metadata_attachment_block_record = | data: BitCodeEntryRecord<usize> | {
        //     if data.code != METADATA_ATTACHMENT {
        //         return Err(WError::CodeNotAllowed("metadata_attachment_block", data.code));
        //     }
        //     if (data.data.len() & 1) == 0 {
        //         parse_metadata_attachment_block_function(data)?;
        //     } else {
        //         let (v1, v2) = parse_metadata_attachment_block_instr(data, metadata_list, metadata_kind)?;
        //         // eprintln!("meta instr: {} {:?}", v1, v2);
        //         res.push(MetaDataAttachmentBlock::Instr(v1, v2));
        //     }
        //     Ok(())
        // };
        // self.parse_entry_only_record(
        //     &mut state,
        //     block_infos,
        //     "MetaDataAttachment Block",
        //     &mut parse_metadata_attachment_block_record,
        // )?;
        // res.shrink_to_fit();
        // Ok(res)
        Ok(())
    }
}

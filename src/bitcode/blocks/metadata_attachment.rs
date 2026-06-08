//! MetaData Attachment Block

use std::collections::HashMap;
use std::io::{Read, Seek};

use crate::bitcode::blocks::blockinfo::BlockInfoAll;
use crate::bitcode::blocks::metadata::{MetaDataBlock, MetaDataUnit, MetaDataValue};
use crate::bitcode::blocks::metadata_kind::{MetaDataKindBlock, MetaDataKindMap};
use crate::bitcode::entry::BitCodeEntryRecord;
use crate::bitcode::parse::StateBlock;
use crate::filebit::BitReader;
use crate::{WError, WResult};

#[derive(Debug)]
pub(in crate::bitcode) struct MetaDataWithKind {
    pub(in crate::bitcode) name: String,
    pub(in crate::bitcode) kind: MetaDataKindMap,
}

#[derive(Debug, Default)]
pub(in crate::bitcode) struct MetaDataAttachmentBlock {
    pub(in crate::bitcode) func: HashMap<usize, Vec<MetaDataValue>>,
    pub(in crate::bitcode) instr: HashMap<usize, Vec<MetaDataWithKind>>,
}

fn parse_metadata_attachment_block_function(_data: BitCodeEntryRecord<usize>) -> WResult<()> {
    todo!("parse metadata_attachment_block");
}

fn parse_metadata_attachment_block_instr(
    data: BitCodeEntryRecord<usize>,
    metadata: &[MetaDataUnit],
    metadata_kind: &MetaDataKindBlock,
) -> WResult<(usize, Vec<(usize, MetaDataKindMap)>)> {
    let idx_res = data.data[0];
    let mut res = Vec::with_capacity(64);
    let mut data = data.data.into_iter().skip(1);
    while let Some(idx_kind) = data.next()
        && let Some(idx_meta) = data.next()
    {
        let kind = metadata_kind.get(&idx_kind);
        if let Some(&MetaDataKindMap::Tbaa) = kind {
            continue;
        }
        let meta = metadata.get(idx_meta);
        if let Some(MetaDataUnit::Node(_, _)) = meta
            && let Some(kind) = kind.cloned()
        {
            res.push((idx_meta, kind));
        }
    }
    res.shrink_to_fit();
    Ok((idx_res, res))
}

const METADATA_ATTACHMENT: u32 = 11;

impl<F: Read + Seek> BitReader<F> {
    pub(in crate::bitcode) fn parse_metadata_attachment_block(
        &mut self,
        mut state: StateBlock,
        block_infos: &BlockInfoAll,
        metadata: &mut MetaDataBlock,
        metadata_kind: &MetaDataKindBlock,
    ) -> WResult<MetaDataAttachmentBlock> {
        // self.skip_bits(state.block_size)?;
        let mut res_func = HashMap::with_capacity(16);
        let mut instr_temp = Vec::with_capacity(128);
        let mut parse_metadata_attachment_block_record = |data: BitCodeEntryRecord<usize>| {
            if data.code != METADATA_ATTACHMENT {
                return Err(WError::CodeNotAllowed(
                    "metadata_attachment_block",
                    data.code,
                ));
            }
            if (data.data.len() & 1) == 0 {
                parse_metadata_attachment_block_function(data)?;
            } else {
                let (v1, v2) =
                    parse_metadata_attachment_block_instr(data, &metadata.data, metadata_kind)?;
                // eprintln!("meta instr: {} {:?}", v1, v2);
                instr_temp.push((v1, v2));
            }
            Ok(())
        };
        self.parse_entry_only_record(
            &mut state,
            block_infos,
            "MetaDataAttachment Block",
            &mut parse_metadata_attachment_block_record,
        )?;
        res_func.shrink_to_fit();
        instr_temp.shrink_to_fit();

        instr_temp.sort_by_key(|v| v.0);
        // eprintln!("meta attach: {:?}", instr_temp);

        let mut res_instr = HashMap::with_capacity(instr_temp.len());
        for (idx_instr, idxs) in instr_temp {
            let mut res_instr_temp = Vec::with_capacity(idxs.len());
            for (idx, kind) in idxs {
                let v = metadata.get_value(idx)?;
                // eprintln!("meta instr: {} {} {:?} {:?}", idx_instr, idx, kinds, v);
                if let MetaDataValue::Named(name) = v {
                    res_instr_temp.push(MetaDataWithKind { name, kind });
                }
            }
            res_instr.insert(idx_instr, res_instr_temp);
        }

        Ok(MetaDataAttachmentBlock {
            func: res_func,
            instr: res_instr,
        })
    }
}

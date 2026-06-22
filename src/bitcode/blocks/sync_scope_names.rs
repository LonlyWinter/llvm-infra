//! SyncScopeNames Block

use std::cell::LazyCell;
use std::io::{Read, Seek};
use std::ops::Deref;
use std::rc::Rc;

use crate::bitcode::blocks::blockinfo::BlockInfoAll;
use crate::bitcode::entry::BitCodeEntryRecord;
use crate::bitcode::parse::StateBlock;
use crate::bitcode::parse_data_to_string;
use crate::{WError, WResult};

use crate::filebit::BitReader;

const SYNC_SCOPE_NAME: u32 = 1;

pub(in crate::bitcode) const SYNC_SCOPE_SINGLE_THREAD: LazyCell<Rc<String>> =
    LazyCell::new(|| Rc::new("syncscope(\"singlethread\")".to_owned()));
pub(in crate::bitcode) const SYNC_SCOPE_SYSTEM: LazyCell<Rc<String>> =
    LazyCell::new(|| Rc::new("".to_owned()));

pub(in crate::bitcode) type SyncScopeName = Rc<String>;

#[derive(Debug, Default)]
pub(in crate::bitcode) struct SyncScopeBlock {
    names: Vec<SyncScopeName>,
}

impl SyncScopeBlock {
    pub(in crate::bitcode) fn parse(&self, idx: usize) -> WResult<Rc<String>> {
        let r = match idx {
            0 => SYNC_SCOPE_SINGLE_THREAD.deref().clone(),
            1 => SYNC_SCOPE_SYSTEM.deref().clone(),
            i if (i - 2) < self.names.len() => self.names[i - 2].clone(),
            _ => return Err(WError::DataNotFound("SyncScopeName")),
        };
        Ok(r)
    }
}

fn parse_sync_scope_names_block(code: u32, data: Vec<u8>) -> WResult<String> {
    if code != SYNC_SCOPE_NAME {
        return Err(WError::CodeNotAllowed("SyncScopeNames", code));
    }
    let res = format!("syncscope(\"{}\")", parse_data_to_string(data.into_iter()));
    Ok(res)
}

impl<F: Read + Seek> BitReader<F> {
    pub(in crate::bitcode) fn parse_sync_scope_names_block(
        &mut self,
        mut state: StateBlock,
        block_infos: &BlockInfoAll,
    ) -> WResult<SyncScopeBlock> {
        let mut res = Vec::with_capacity(64);
        let mut parse_sync_scope_names_block_now = |data: BitCodeEntryRecord<u8>| {
            let t = parse_sync_scope_names_block(data.code, data.data)?;
            res.push(Rc::new(t));
            Ok(())
        };

        self.parse_entry_only_record(
            &mut state,
            block_infos,
            "SyncScopeNames Block",
            &mut parse_sync_scope_names_block_now,
        )?;
        res.shrink_to_fit();
        Ok(SyncScopeBlock { names: res })
    }
}

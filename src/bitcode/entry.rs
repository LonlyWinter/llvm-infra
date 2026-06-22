//! entry

use std::{
    io::{Read, Seek},
    vec::IntoIter,
};

use crate::{WError, WResult, bitcode::{blocks::blockinfo::BlockInfoAll, parse::StringRef, parse_data_to_string}};

use crate::{
    bitcode::{
        abbrev::{AbbrevOp, UnabbrevRecordData},
        parse::StateBlock,
    },
    filebit::{BitReader, BitType},
};

use super::abbrev::BlockIDs;

pub(super) const CODE_LEN_WIDTH: usize = 4;
pub(super) const BLOCK_SIZE_WIDTH: usize = 32;
pub(super) const MAX_CHUNK_SIZE: usize = 32;
const MAX_NUM_ELEMENTS: usize = 2 << 20;

enum_reader! {
    /// https://llvm.org/docs/BitCodeFormat.html#abbreviation-ids
    AbbreviationIDs, AbbrevRecord;
    Debug,;
    EndBlock = 0,
    EnterSubblock = 1,
    DefineAbbrev = 2,
    UnabbrevRecord = 3,
}

#[derive(Debug)]
pub(super) struct DataNext<T: BitType> {
    idx: usize,
    len: usize,
    data: IntoIter<T>,
}

impl<T: BitType> From<Vec<T>> for DataNext<T> {
    fn from(data: Vec<T>) -> Self {
        Self {
            idx: 0,
            len: data.len(),
            data: data.into_iter(),
        }
    }
}

impl<T: BitType> DataNext<T> {
    pub(super) fn len(&self) -> usize {
        if self.idx >= self.len {
            return 0;
        }
        self.len - self.idx
    }

    pub(super) fn into_iter(self) -> IntoIter<T> {
        self.data
    }

    pub(super) fn next(&mut self, tag: &'static str) -> WResult<T> {
        self.idx += 1;
        self.data.next().ok_or(WError::DataNotFound(tag))
    }

    pub(super) fn next_bool(&mut self, tag: &'static str) -> WResult<bool> {
        let data = self.next(tag)?;
        Ok(data != T::from(0u8))
    }

    pub(super) fn next_convert<S: TryFrom<T, Error = WError>>(
        &mut self,
        tag: &'static str,
    ) -> WResult<S> {
        let data = self.next(tag)?;
        S::try_from(data)
    }

    pub(super) fn next_alignment(&mut self, tag: &'static str) -> WResult<u16> {
        let t = self.next(tag)?;
        let t = t.into_u8();
        if t == 0 {
            return Err(WError::DataNotFound(tag));
        }
        Ok(parse_alignment(t - 1))
    }
}

pub(super) fn parse_alignment(data: u8) -> u16 {
    1 << data
}


impl DataNext<u64> {
    pub(super) fn next_usize(&mut self, tag: &'static str) -> WResult<usize> {
        let data = self.next(tag)?;
        Ok(data as usize)
    }
    
    pub(super) fn next_name(&mut self, strtab: &StringRef) -> WResult<String> {
        let strtab_offset = self.next_usize("strtab_offset")?;
        let strtab_size = self.next_usize("strtab_size")?;
        let name = strtab
            .get(strtab_offset..(strtab_offset + strtab_size))
            .and_then(| v | if v.is_empty() { None } else { Some(v) })
            .map(|v| parse_data_to_string(v.iter().copied()))
            .ok_or(WError::DataNotFound("strtab_name"))?;
        Ok(name)
    }
}

#[derive(Debug)]
pub(super) struct BitCodeEntryRecord<T: BitType> {
    pub(super) code: u32,
    pub(super) data: Vec<T>,
    pub(super) chars: String,
    pub(super) blob: Vec<u8>,
}

#[derive(Debug)]
pub(super) enum BitCodeEntry<T: BitType> {
    Record(BitCodeEntryRecord<T>),
    EndBlock,
    SubBlock(BlockIDs, StateBlock),
}

impl<F: Read + Seek> BitReader<F> {
    pub(super) fn block_header(&mut self) -> WResult<(usize, usize)> {
        let code_size = self.variable_width_integers::<usize>(CODE_LEN_WIDTH)?;
        if code_size > MAX_CHUNK_SIZE {
            return Err(WError::Num("code size", true, code_size, MAX_CHUNK_SIZE));
        }
        self.word_alignment(32)?;
        let block_size = self.fixed_width_integers::<usize>(BLOCK_SIZE_WIDTH)? << 5;
        if code_size == 0 {
            return Err(WError::BlockEmpty);
        }
        Ok((block_size, code_size))
    }

    /// https://llvm.org/docs/BitCodeFormat.html#enter-subblock-encoding
    pub(super) fn enter_sub_block(
        &mut self,
        block_infos: &BlockInfoAll,
        abbrev_id: usize,
    ) -> WResult<StateBlock> {
        let abbrevs = block_infos
            .get(&abbrev_id)
            .map(|v| v.abbrevs.clone())
            .unwrap_or_default();
        let (block_size, code_size) = self.block_header()?;
        let n = self.has_bits(block_size)?;
        if !n {
            return Err(WError::BlockNotFound(block_size));
        }
        let state = StateBlock {
            code_size,
            block_size,
            abbrevs,
        };
        Ok(state)
    }

    /// https://llvm.org/docs/BitCodeFormat.html#define-abbrev-encoding
    pub(super) fn read_record_unabbrev<T>(&mut self) -> WResult<(u32, Vec<T>)>
    where
        T: BitType,
    {
        // eprintln!("read record: none");
        let code = self.variable_width_integers::<u32>(6)?;
        let num_elements = self.variable_width_integers::<usize>(6)?;
        if num_elements > MAX_NUM_ELEMENTS {
            return Err(WError::Num(
                "unabbrev num elements",
                true,
                num_elements,
                MAX_NUM_ELEMENTS,
            ));
        }
        let data = (0..num_elements)
            .map(|_| self.variable_width_integers::<T>(6))
            .collect::<WResult<Vec<_>>>()?;
        // eprintln!("read record unabbrev: {} {}", code, num_elements);
        Ok((code, data))
    }

    /// https://llvm.org/docs/BitCodeFormat.html#define-abbrev-encoding
    pub(super) fn read_record_abbrev<T: BitType>(
        &mut self,
        state: &mut StateBlock,
        abbrev_id: usize,
    ) -> WResult<BitCodeEntryRecord<T>> {
        // eprintln!("read record:  abbrev_id {}", abbrev_id);
        let abbvs = state
            .abbrevs
            .get(abbrev_id - 4)
            .ok_or(WError::CodeNotAllowed("Abbrev id", abbrev_id as u32))?;
        let code_op = abbvs.first().ok_or(WError::AbbrevOpEmpty)?;
        let code = self.read_abbreviated_field_code(code_op)?;
        // eprintln!("abbrevs: {} {} {:?}", abbrev_id, code, abbvs);
        let mut ops_iter = abbvs.iter();
        ops_iter.next();
        let mut data = Vec::with_capacity(64);
        let mut chars = String::with_capacity(32);
        let mut blob = Vec::with_capacity(64);
        while let Some(op) = ops_iter.next() {
            // eprintln!("op temp: {:?}", op);
            if let Ok(r) = self.read_abbreviated_field::<T>(op) {
                match r {
                    UnabbrevRecordData::Data(v) => data.push(v),
                    UnabbrevRecordData::Char(v) => chars.push(v),
                }
                continue;
            }
            let num_elements = self.variable_width_integers::<usize>(6)?;
            if num_elements > MAX_NUM_ELEMENTS {
                return Err(WError::Num(
                    "abbrev num elements",
                    true,
                    num_elements,
                    MAX_NUM_ELEMENTS,
                ));
            }
            // eprintln!("num elements: {}", num_elements);
            if matches!(op, AbbrevOp::Array) {
                match ops_iter.next() {
                    Some(AbbrevOp::Fixed(n)) => {
                        let rs = (0..num_elements)
                            .map(|_| self.fixed_width_integers::<T>(*n))
                            .collect::<WResult<Vec<T>>>()?;
                        data.extend(rs);
                    }
                    Some(AbbrevOp::Vbr(n)) => {
                        let rs = (0..num_elements)
                            .map(|_| self.variable_width_integers::<T>(*n))
                            .collect::<WResult<Vec<T>>>()?;
                        data.extend(rs);
                    }
                    Some(AbbrevOp::Char6) => {
                        let rs = (0..num_elements)
                            .map(|_| self.characters_6bit())
                            .collect::<WResult<Vec<_>>>()?;
                        chars.extend(rs);
                    }
                    Some(v) => {
                        return Err(WError::AbbrevType(v.clone()));
                    }
                    None => {
                        return Err(WError::AbbrevOpArrayElement);
                    }
                }
                continue;
            }
            // blob
            self.word_alignment(32)?;
            // eprintln!("blob num: {}", num_elements);
            let blobs = self.read_bytes(num_elements)?;
            // eprintln!("blob num: {}", num_elements);
            blob.extend(blobs);
            let t = num_elements & 3;
            if t > 0 {
                // tail padding, 需要skip
                let n_skip = (4 - t) << 3;
                self.skip_bits(n_skip)?;
            }
        }

        data.shrink_to_fit();
        chars.shrink_to_fit();
        blob.shrink_to_fit();
        Ok(BitCodeEntryRecord {
            code,
            data,
            chars,
            blob,
        })
    }

    pub(in crate::bitcode) fn parse_entry<T: BitType>(
        &mut self,
        state: &mut StateBlock,
        block_infos: &BlockInfoAll,
    ) -> WResult<BitCodeEntry<T>> {
        // let mut idx = 0;
        while self.has_data() {
            // idx += 1;
            // eprintln!("entry parse abbrev ... {} {}", code_size, self.position()?);
            let code = AbbreviationIDs::parse_with_data(self, &state.code_size)?;
            // eprintln!("entry parse abbrev: {:?} ...", code);
            match code {
                AbbreviationIDs::EndBlock => {
                    // align 4 byte
                    self.word_alignment(32)?;
                    return Ok(BitCodeEntry::EndBlock);
                }
                AbbreviationIDs::EnterSubblock => {
                    let id = BlockIDs::parse(self)?;
                    // eprintln!("Sub-block {:?} ...", id);
                    let idx = id.clone().into();
                    let state = self.enter_sub_block(block_infos, idx)?;
                    return Ok(BitCodeEntry::SubBlock(id, state));
                }
                AbbreviationIDs::DefineAbbrev => {
                    self.read_abbrev_record(state)?;
                }
                AbbreviationIDs::UnabbrevRecord => {
                    // eprintln!("UnabbrevRecord ... {}", state.code_size);
                    let (code, data) = self.read_record_unabbrev::<T>()?;
                    return Ok(BitCodeEntry::Record(BitCodeEntryRecord {
                        code,
                        data,
                        chars: String::with_capacity(0),
                        blob: Vec::with_capacity(0),
                    }));
                }
                AbbreviationIDs::AbbrevRecord(id) => {
                    // eprintln!("AbbrevRecord {} ...", id);
                    let v = self.read_record_abbrev(state, id)?;
                    return Ok(BitCodeEntry::Record(v));
                }
            }
        }
        Err(WError::DataNotFound("entry"))
    }

    pub(in crate::bitcode) fn parse_entry_only_record<T, R>(
        &mut self,
        state: &mut StateBlock,
        block_infos: &BlockInfoAll,
        tag: &'static str,
        mut func_record: R,
    ) -> WResult<()>
    where
        T: BitType,
        R: FnMut(BitCodeEntryRecord<T>) -> WResult<()>,
    {
        while self.has_data() {
            let entry = self.parse_entry::<T>(state, block_infos)?;
            // eprintln!("parse_entry_only_record, {}, {:?} ...", tag, entry);
            match entry {
                BitCodeEntry::SubBlock(id, _) => return Err(WError::BlockNotAllowed(tag, id)),
                BitCodeEntry::Record(record) => func_record(record)?,
                BitCodeEntry::EndBlock => break,
            }
        }
        Ok(())
    }
}

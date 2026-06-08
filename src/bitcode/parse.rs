//! parse

use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek},
    path::Path,
};

use crate::bitcode::{
    abbrev::{Abbrev, BlockIDs},
    blocks::moduleblock::ModuleBlock,
    entry::BitCodeEntry,
};
use crate::filebit::BitReader;
use crate::{WError, WResult};

/// 0x4243C0DE: 'B' 'C' 0 C E D
/// https://llvm.org/docs/BitCodeFormat.html#llvm-ir-magic-number
static MAGIC_NUMBER: [u8; 4] = [0x42, 0x43, 0xC0, 0xDE];

pub(super) type StringRef = Vec<u8>;

#[derive(Debug)]
pub(super) struct StateBlock {
    pub(super) code_size: usize,
    pub(super) block_size: usize,
    pub(super) abbrevs: Vec<Abbrev>,
}

impl Default for StateBlock {
    fn default() -> Self {
        Self {
            code_size: 2,
            block_size: 0,
            abbrevs: Vec::with_capacity(16),
        }
    }
}

pub struct BitCode {
    pub(super) info: String,
    pub(super) module_block: ModuleBlock,
}

impl<F: Read + Seek> BitReader<F> {
    fn parse_bitcode(&mut self) -> WResult<BitCode> {
        let mut info = None;
        let mut state = StateBlock::default();
        // sub-block没有block_info，所以不存在
        let block_infos = HashMap::with_capacity(0);
        let mut strtab = Vec::with_capacity(128);
        let mut symtab = Vec::with_capacity(128);
        let mut state_module_block = None;
        while self.has_data() {
            let entry = self.parse_entry::<u32>(&mut state, &block_infos)?;
            match entry {
                BitCodeEntry::SubBlock(id, state_sub) => match id {
                    BlockIDs::IdentificationBlock => {
                        let info_temp = self.parse_identification_block(state_sub, &block_infos)?;
                        info.replace(info_temp);
                        let posi = self.position()?;
                        let entry = self.parse_entry::<u32>(&mut state, &block_infos)?;
                        if let BitCodeEntry::SubBlock(BlockIDs::ModuleBlock, _) = entry {
                            self.reload(posi)?;
                        } else {
                            return Err(WError::DataNotFound(
                                "ModuleBlock after IdentificationBlock",
                            ));
                        }
                    }
                    BlockIDs::ModuleBlock => {
                        if state_module_block.is_some() {
                            return Err(WError::DataSetAlready("module block"));
                        }
                        let posi = self.position()?;
                        self.skip_bits(state_sub.block_size)?;
                        state_module_block.replace((state_sub, posi));
                    }
                    BlockIDs::StrtabBlock => {
                        strtab = self.parse_str_sym_table_block(state_sub, &block_infos)?;
                    }
                    BlockIDs::SymtabBlock => {
                        symtab = self.parse_str_sym_table_block(state_sub, &block_infos)?;
                    }
                    v => return Err(WError::BlockNotAllowed("BitCode", v)),
                },
                BitCodeEntry::Record(data) => {
                    return Err(WError::CodeNotAllowed("bitcode record", data.code));
                }
                BitCodeEntry::EndBlock => break,
            }
        }
        let (state, posi) = state_module_block.ok_or(WError::DataNotFound("module block"))?;
        self.reload(posi)?;
        let module_block = self.parse_module_block(state, strtab, symtab)?;
        Ok(BitCode {
            info: info.unwrap_or_default(),
            module_block,
        })
    }
}

impl BitCode {
    pub(super) fn check_file_format<P: AsRef<Path>>(p: P) -> WResult<P> {
        if let Some(ext) = p.as_ref().extension()
            && ext.eq_ignore_ascii_case("bc")
        {
            Ok(p)
        } else {
            Err(WError::FileExtNeed(".bc"))
        }
    }

    pub fn from_file<P: AsRef<Path>>(p: P) -> WResult<Self> {
        let p = Self::check_file_format(p)?;
        // 打开文件
        let f = File::open(p)?;
        let mut reader = BitReader::new(f);
        // 读取
        // magic
        let magic = reader.read_bitn(32)?;
        if magic.as_slice() != MAGIC_NUMBER {
            return Err(WError::MagicNumber(magic));
        }
        // 解析数据
        reader.parse_bitcode()
    }
}

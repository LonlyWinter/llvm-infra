//! module block
//! https://llvm.org/docs/BitCodeFormat.html#module-block-contents

use std::collections::HashMap;
use std::io::{Read, Seek};
use std::rc::Rc;

use crate::bitcode::blocks::data_layout::{DataLayout, parse_data_layout};
use crate::bitcode::blocks::function_block::utils::FunctionBlock;
use crate::bitcode::blocks::metadata::MetaDataBlock;
use crate::bitcode::blocks::metadata_kind::MetaDataKindBlock;
use crate::bitcode::blocks::typeblock::{TypeBlockData, TypeBlockUnit};
use crate::bitcode::parse::StateBlock;
use crate::bitcode::parse_data_to_string;
use crate::{WError, WResult};

use crate::filebit::BitReader;

use crate::bitcode::blocks::blockinfo::BlockInfo;
use crate::bitcode::blocks::constants::{ConstantsBlock, DataSingle};
use crate::bitcode::blocks::function_record::{Function, parse_function_record};
use crate::bitcode::blocks::globalvar::{GlobalVar, parse_global_var_record};
use crate::bitcode::{
    abbrev::BlockIDs,
    blocks::{
        param_attr::ParamAttrBlock, param_attr_group::ParamAttrGroupBlock, typeblock::TypeBlock,
    },
    entry::BitCodeEntry,
    parse::StringRef,
};

#[derive(Debug, Clone, PartialEq)]
pub(in crate::bitcode) enum GlobalValueUnit {
    Meta(usize),
    Data(Rc<DataSingle>),
    GlobalVar(Rc<GlobalVar>),
    Function(Rc<Function>),
    /// idx_instr
    Instr(TypeBlockUnit, usize, usize),
    /// idx_block
    Label(usize),
    /// idx_value, ty
    Future(usize, TypeBlockUnit),
}

pub(in crate::bitcode) type GlobalValueList = Vec<GlobalValueUnit>;

impl GlobalValueUnit {
    pub(super) fn get_ty(&self, tag: &'static str) -> WResult<TypeBlockUnit> {
        let r = match &self {
            GlobalValueUnit::Instr(v, _, _) => v.clone(),
            GlobalValueUnit::Data(v) => v.ty(),
            GlobalValueUnit::Future(_, ty) => ty.clone(),
            GlobalValueUnit::GlobalVar(v) => v.ty.clone(),
            GlobalValueUnit::Meta(_) => Rc::new(TypeBlockData::MetaData),
            _ => {
                return Err(WError::DataNotAllowed(
                    "parse value type",
                    format!("{} {:?}", tag, self),
                ));
            }
        };
        Ok(r)
    }
}

#[derive(Debug, Default)]
pub(in crate::bitcode) struct ModuleBlockBasic {
    pub(in crate::bitcode) use_relative_ids: bool,
    pub(in crate::bitcode) use_strtab: bool,
    pub(in crate::bitcode) version: u8,
    pub(in crate::bitcode) triple: String,
    pub(in crate::bitcode) data_layout: DataLayout,
    pub(in crate::bitcode) asm: String,
    pub(in crate::bitcode) section_name: String,
    pub(in crate::bitcode) deplib: String,
    pub(in crate::bitcode) gc_name: String,
    pub(in crate::bitcode) source_filename: String,
    pub(in crate::bitcode) vst_offset: Option<usize>,
}

#[derive(Debug)]
pub(in crate::bitcode) struct ModuleBlock {
    pub(in crate::bitcode) strtab: StringRef,
    pub(in crate::bitcode) symtab: StringRef,
    pub(in crate::bitcode) basic: ModuleBlockBasic,
    pub(in crate::bitcode) global_var: Vec<Rc<GlobalVar>>,
    pub(in crate::bitcode) function: Vec<Rc<Function>>,
    pub(in crate::bitcode) data_types: TypeBlock,
    pub(in crate::bitcode) param_attr_group: ParamAttrGroupBlock,
    pub(in crate::bitcode) param_attrs: ParamAttrBlock,
    pub(in crate::bitcode) constants: ConstantsBlock,
    pub(in crate::bitcode) metadata_kind: MetaDataKindBlock,
    pub(in crate::bitcode) metadata: MetaDataBlock,
    pub(in crate::bitcode) operand_bundle_tags: Vec<Rc<String>>,
    pub(in crate::bitcode) sync_scope_names: Vec<Rc<String>>,
    pub(in crate::bitcode) funcs: Vec<FunctionBlock>,
    pub(in crate::bitcode) vars: GlobalValueList,
}

fn parse_record_to_string(data: Vec<u64>) -> String {
    parse_data_to_string(data.into_iter().map(|v| v as u8))
}

enum_parse! {
    ModuleBlockContents, u32;
    Debug,;
    Version = 1,     // VERSION:     [version#]
    Triple = 2,      // TRIPLE:      [strchr x N]
    Datalayout = 3,  // DATALAYOUT:  [strchr x N]
    Asm = 4,         // ASM:         [strchr x N]
    SectionName = 5, // SECTIONNAME: [strchr x N]
    // Deprecated, but still needed to read old bitcode files.
    DepLib = 6, // DEPLIB:      [strchr x N]
    // GLOBALVAR: [pointer type, isconst, initid,
    //             linkage, alignment, section, visibility, threadlocal]
    GlobalVar = 7,
    // FUNCTION:  [type, callingconv, isproto, linkage, paramattrs, alignment,
    //             section, visibility, gc, unnamed_addr]
    Function = 8,
    // ALIAS: [alias type, aliasee val#, linkage, visibility]
    AliasOld = 9,
    GcName = 11, // GCNAME: [strchr x N]
    Comdat = 12, // COMDAT: [selection_kind, name]
    Vstoffset = 13, // VSTOFFSET: [offset]
    // ALIAS: [alias value type, addrspace, aliasee val#, linkage, visibility]
    Alias = 14,
    MetadataValuesUnused = 15,
    // SOURCE_FILENAME: [namechar x N]
    SourceFileName = 16,
    // HASH: [5*i32]
    Hash = 17,
    // IFUNC: [ifunc value type, addrspace, resolver val#, linkage, visibility]
    Ifunc = 18,
}

impl<F: Read + Seek> BitReader<F> {
    pub(in crate::bitcode) fn parse_module_block(
        &mut self,
        mut state: StateBlock,
        mut strtab: StringRef,
        mut symtab: StringRef,
    ) -> WResult<ModuleBlock> {
        // todo!("parse module block: {:?} {}", state.get_current(), reader.position()?);
        let mut block_infos: HashMap<usize, BlockInfo> = HashMap::with_capacity(8);
        let mut basic = ModuleBlockBasic::default();
        let mut global_var = Vec::with_capacity(64);
        let mut function = Vec::with_capacity(64);
        let mut data_types = Vec::with_capacity(64);
        let mut param_attrs = Vec::with_capacity(64);
        let mut param_attr_group = HashMap::with_capacity(64);
        let mut constants = Vec::with_capacity(64);
        let mut metadata_kind = HashMap::with_capacity(64);
        let mut metadata = MetaDataBlock::default();
        let mut operand_bundle_tags = Vec::with_capacity(64);
        let mut sync_scope_names = Vec::with_capacity(64);
        let mut value_list = Vec::with_capacity(256);
        let mut function_list = Vec::with_capacity(16);
        let mut funcs = Vec::with_capacity(32);
        while self.has_data() {
            let entry = self.parse_entry::<u64>(&mut state, &block_infos)?;
            // eprintln!("parse module block ... {:?}", entry);
            match entry {
                BitCodeEntry::SubBlock(id, state_sub) => {
                    // eprintln!(
                    //     "parse module block sub block ... {:?} {}",
                    //     id, state_sub.code_size
                    // );
                    match id {
                        BlockIDs::BlockInfo => {
                            let t = self.parse_block_info(state_sub, &block_infos)?;
                            block_infos.extend(t);
                        }
                        BlockIDs::TypeBlock => {
                            let t = self.parse_type_block(state_sub, &block_infos)?;
                            data_types.extend(t);
                        }
                        BlockIDs::ParamAttrGroupBlock => {
                            let t = self.parse_param_attr_group_block(
                                state_sub,
                                &block_infos,
                                &data_types,
                            )?;
                            param_attr_group.extend(t);
                        }
                        BlockIDs::ParamAttrBlock => {
                            let t = self.parse_param_attr_block(
                                state_sub,
                                &block_infos,
                                &param_attr_group,
                            )?;
                            param_attrs.extend(t);
                        }
                        BlockIDs::ConstantsBlock => {
                            let t =
                                self.parse_constants_block(state_sub, &block_infos, &data_types)?;
                            value_list.extend(t.clone().into_iter().map(GlobalValueUnit::Data));
                            // eprintln!("Constant: {}", value_list.len());
                            constants.extend(t);
                        }
                        BlockIDs::MetaDataBlock => {
                            self.parse_metadata_block(
                                state_sub,
                                &block_infos,
                                &data_types,
                                &value_list,
                                &mut metadata,
                            )?;
                        }
                        BlockIDs::MetaDataKindBlock => {
                            let t = self.parse_meta_data_kind_block(state_sub, &block_infos)?;
                            metadata_kind.extend(t);
                        }
                        BlockIDs::OperandBundleTagsBlock => {
                            let t =
                                self.parse_operand_bundle_tags_block(state_sub, &block_infos)?;
                            // value_list.extend(t.clone()
                            //     .into_iter()
                            //     .map(ValueUnit::Tag));
                            operand_bundle_tags.extend(t);
                        }
                        BlockIDs::ValueSymtabBlock => {
                            self.parse_value_symtab_block_normal(
                                state_sub,
                                &block_infos,
                                &mut value_list,
                            )?;
                        }
                        BlockIDs::FunctionBlock => {
                            if let Some(offset) = basic.vst_offset.take() {
                                let posi = self.reload(offset)?;
                                // eprintln!("skip vst: {} {}", offset, posi);
                                let entry1 = self.parse_entry::<u64>(&mut state, &block_infos)?;
                                let entry2 = self.parse_entry::<u64>(&mut state, &block_infos)?;
                                if let BitCodeEntry::EndBlock = entry1
                                    && let BitCodeEntry::SubBlock(
                                        BlockIDs::ValueSymtabBlock,
                                        state_skip,
                                    ) = entry2
                                {
                                    function_list = self.parse_value_symtab_block_skip(
                                        state_skip,
                                        &block_infos,
                                        basic.use_strtab,
                                        &value_list,
                                    )?;
                                    // eprintln!("skip back: {}", posi);
                                    self.reload(posi)?;
                                } else {
                                    return Err(WError::DataNotAllowed(
                                        "Skip ValueSymtabBlock",
                                        format!("{:?} {:?}", entry1, entry2),
                                    ));
                                }
                            }
                            let function = function_list
                                .pop()
                                .ok_or(WError::DataNotFound("function block meta"))?;
                            let func = self.parse_function_block(
                                state_sub,
                                function,
                                value_list.clone(),
                                &mut metadata,
                                &metadata_kind,
                                &block_infos,
                                &data_types,
                                &param_attrs,
                                &basic,
                            )?;
                            // eprintln!(
                            //     "Function: {} {}",
                            //     func.base.name,
                            //     func.blocks.iter().map(|v| v.len()).sum::<usize>()
                            // );
                            funcs.push(func);
                        }
                        BlockIDs::AppSpec(v) => {
                            self.parse_app_spec_block(state_sub, &block_infos, v)?;
                        }
                        BlockIDs::SyncScopeNamesBlock => {
                            let t = self.parse_sync_scope_names_block(state_sub, &block_infos)?;
                            // value_list.extend(t.clone()
                            //     .into_iter()
                            //     .map(ValueUnit::Tag));
                            sync_scope_names.extend(t);
                        }
                        _ => return Err(WError::BlockNotAllowed("Module Block", id)),
                    }
                }
                BitCodeEntry::Record(data) => {
                    // eprintln!("parse module block record ... {}", code);
                    let content = ModuleBlockContents::try_from(data.code)?;
                    // eprintln!("parse module block record ... {:?}", content);
                    match content {
                        ModuleBlockContents::Version => {
                            if data.data.is_empty() {
                                return Err(WError::DataNotFound("Version in parse module_block"));
                            }
                            let version = data.data[0];
                            if version > 2 {
                                return Err(WError::CodeNotAllowed(
                                    "Version record error",
                                    version as u32,
                                ));
                            }
                            // eprintln!("version: {}", version);
                            basic.version = version as u8;
                            basic.use_relative_ids = version >= 1;
                            basic.use_strtab = version >= 2;
                        }
                        ModuleBlockContents::Triple => {
                            basic.triple = parse_record_to_string(data.data);
                        }
                        ModuleBlockContents::Datalayout => {
                            basic.data_layout = parse_data_layout(data.data)?;
                        }
                        ModuleBlockContents::Asm => {
                            basic.asm = data.chars;
                        }
                        ModuleBlockContents::SectionName => {
                            basic.section_name = data.chars;
                        }
                        ModuleBlockContents::DepLib => {
                            basic.deplib = data.chars;
                        }
                        ModuleBlockContents::GcName => {
                            basic.gc_name = data.chars;
                        }
                        ModuleBlockContents::SourceFileName => {
                            basic.source_filename = parse_record_to_string(data.data);
                        }
                        ModuleBlockContents::GlobalVar => {
                            let global_var_temp =
                                parse_global_var_record(data.data, &strtab, &data_types)?;
                            let global_var_temp = Rc::new(global_var_temp);
                            value_list.push(GlobalValueUnit::GlobalVar(global_var_temp.clone()));
                            global_var.push(global_var_temp);
                        }
                        ModuleBlockContents::Function => {
                            let function_temp = parse_function_record(
                                data.data,
                                &strtab,
                                &data_types,
                                &param_attrs,
                            )?;
                            let function_temp = Rc::new(function_temp);
                            value_list.push(GlobalValueUnit::Function(function_temp.clone()));
                            function.push(function_temp);
                        }
                        ModuleBlockContents::Vstoffset => {
                            if data.data.is_empty() {
                                return Err(WError::DataNotFound(
                                    "Vstoffset in parse module_block",
                                ));
                            }
                            let posi = ((data.data[0] - 1) << 5) as usize;
                            basic.vst_offset = Some(posi);
                        }
                        _ => {
                            todo!("TODO: {:?}", content)
                        }
                    }
                    // eprintln!("parse module block record: {:?}", bitcode.module_block);
                }
                BitCodeEntry::EndBlock => break,
            };
        }

        strtab.shrink_to_fit();
        symtab.shrink_to_fit();
        global_var.shrink_to_fit();
        function.shrink_to_fit();
        data_types.shrink_to_fit();
        param_attrs.shrink_to_fit();
        param_attr_group.shrink_to_fit();
        constants.shrink_to_fit();
        metadata_kind.shrink_to_fit();
        operand_bundle_tags.shrink_to_fit();
        sync_scope_names.shrink_to_fit();
        funcs.shrink_to_fit();
        value_list.shrink_to_fit();
        Ok(ModuleBlock {
            strtab,
            symtab,
            basic,
            global_var,
            function,
            data_types,
            param_attr_group,
            param_attrs,
            constants,
            metadata_kind,
            metadata,
            operand_bundle_tags,
            sync_scope_names,
            funcs,
            vars: value_list,
        })
    }
}

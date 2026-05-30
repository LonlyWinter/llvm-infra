//! Function Block
//! https://llvm.org/docs/BitCodeFormat.html#function-block-contents

use std::cell::LazyCell;
use std::collections::HashMap;
use std::io::{Read, Seek};
use std::mem::take;
use std::rc::Rc;

use crate::bitcode::blocks::blockinfo::BlockInfoAll;
use crate::bitcode::blocks::constants::DataSingle;
use crate::bitcode::blocks::function_block::inst_alloca::InstrAlloca;
use crate::bitcode::blocks::function_block::inst_binop::InstrBinOP;
use crate::bitcode::blocks::function_block::inst_br::InstrBr;
use crate::bitcode::blocks::function_block::inst_call::InstrCall;
use crate::bitcode::blocks::function_block::inst_cast::InstrCast;
use crate::bitcode::blocks::function_block::inst_cmp::InstrCmp;
use crate::bitcode::blocks::function_block::inst_extractelt::InstrExtractElt;
use crate::bitcode::blocks::function_block::inst_gep::InstrGEP;
use crate::bitcode::blocks::function_block::inst_insertelt::InstrInsertElt;
use crate::bitcode::blocks::function_block::inst_load::InstrLoad;
use crate::bitcode::blocks::function_block::inst_phi::InstrPhi;
use crate::bitcode::blocks::function_block::inst_ret::InstrRet;
use crate::bitcode::blocks::function_block::inst_shufflevec::InstrShuffleVec;
use crate::bitcode::blocks::function_block::inst_store::InstrStore;
use crate::bitcode::blocks::function_block::inst_vselect::InstrVSelect;
use crate::bitcode::blocks::function_record::Function;
use crate::bitcode::blocks::metadata::{MetaData, NamedMetaData};
use crate::bitcode::blocks::metadata_kind::MetaDataKindBlock;
use crate::bitcode::blocks::moduleblock::{GlobalValueList, GlobalValueUnit, ModuleBlockBasic};
use crate::bitcode::blocks::param_attr::ParamAttrBlock;
use crate::bitcode::blocks::typeblock::{TypeBlock, TypeBlockData, TypeBlockUnit};
use crate::bitcode::blocks::value_symtab::ValueSymTabBlock;
use crate::bitcode::entry::DataNext;
use crate::bitcode::parse::StateBlock;
use crate::{WError, WResult};

use crate::{
    bitcode::{abbrev::BlockIDs, entry::BitCodeEntry},
    bitreader::BitReader,
};

macro_rules! enum_function {
    (
        // 一个code value对应一个指令，内部是哪个类型数据，内部类型数据有个new方法
        $(#[doc = $doc1:expr] $code1:expr, $instr1:ident, $ty1:ident)*
        ;
        // 同上，但是是终结指令
        $(#[doc = $doc2:expr] $code2:expr, $instr2:ident, $ty2:ident)*
    ) => {
        #[derive(Debug)]
        pub(in crate::bitcode) enum FunctionInstr {
            $(
                #[doc = $doc1]
                $instr1($ty1),
            )*
            $(
                #[doc = $doc2]
                $instr2($ty2),
            )*
        }

        impl FunctionInstr {
            pub(in crate::bitcode) fn is_terminator(&self) -> bool {
                match self {
                    $(
                        Self::$instr2(_) => true,
                    )*
                    _ => false,
                }
            }

            pub(in crate::bitcode) fn res(&self) -> Option<&TypeBlockUnit> {
                match self {
                    $(
                        Self::$instr1(v) => v.res(),
                    )*
                    $(
                        Self::$instr2(v) => v.res(),
                    )*
                }
            }

            fn new<'a>(
                code: u32,
                data: FunctionInstrData<'a>
            ) -> WResult<FunctionInstr> {
                // eprintln!("parse instr: {} {}", data.other.idx, code);
                let res = match code {
                    $(
                        $code1 => {
                            let v = $ty1::new(data)?;
                            FunctionInstr::$instr1(v)
                        },
                    )*
                    $(
                        $code2 => {
                            let v = $ty2::new(data)?;
                            FunctionInstr::$instr2(v)
                        },
                    )*
                    _ => return Err(WError::CodeNotAllowed("parse function instr", code))
                };
                Ok(res)
            }
        }
    };
}

pub(super) struct FunctionInstrDataOther<'a> {
    pub(super) idx: usize,
    pub(super) data_types: &'a TypeBlock,
    pub(super) param_attrs: &'a ParamAttrBlock,
    pub(super) data_basic: &'a ModuleBlockBasic,
    pub(super) values: &'a GlobalValueList,
    pub(super) metadata: &'a MetaData,
}
pub(super) struct FunctionInstrData<'a> {
    pub(super) data: DataNext<usize>,
    pub(super) chars: String,
    pub(super) blob: Vec<u8>,
    pub(super) other: FunctionInstrDataOther<'a>,
}

pub(super) trait FunctionInstrTrait: Sized {
    fn new<'a>(data: FunctionInstrData<'a>) -> WResult<Self>;

    fn res(&self) -> Option<&TypeBlockUnit> {
        None
    }
}

impl<'a> FunctionInstrDataOther<'a> {
    pub(super) fn get_value<F: FnMut() -> WResult<usize>>(
        &self,
        idx: usize,
        ty: Option<TypeBlockUnit>,
        tag: &'static str,
        mut ty_future: F,
    ) -> WResult<(GlobalValueUnit, usize)> {
        // eprintln!("Get value: {} {} {}", self.data_basic.use_relative_ids, self.values.len(), idx);
        let idx = if self.data_basic.use_relative_ids {
            self.values.len().wrapping_sub(idx)
        } else {
            idx
        };
        let val = self.values.get(idx).cloned();
        let r = if let Some(ty) = ty {
            match (ty.as_ref(), val) {
                (TypeBlockData::Label, _) => GlobalValueUnit::Label(idx),
                (TypeBlockData::OpaquePointer(idx), Some(GlobalValueUnit::Instr(v, i1, i2))) => {
                    GlobalValueUnit::Instr(v, i1, i2 - *idx)
                }
                (TypeBlockData::MetaData, _) => {
                    GlobalValueUnit::Meta(idx)
                }
                (_, Some(GlobalValueUnit::Data(v))) if v.ty() == ty => GlobalValueUnit::Data(v),
                (_, Some(GlobalValueUnit::Instr(v, i1, i2))) if v == ty => {
                    GlobalValueUnit::Instr(v, i1, i2)
                }
                (_, None) => GlobalValueUnit::Future(idx, ty),
                (i, v) => {
                    return Err(WError::DataNotAllowed(
                        tag,
                        format!("ty: {:?}, val: {:?}", i, v),
                    ));
                }
            }
        } else if let Some(val) = val {
            val
        } else {
            let idx_ty = ty_future()?;
            let ty = self
                .data_types
                .get(idx_ty)
                .cloned()
                .ok_or(WError::DataNotFound(tag))?;
            GlobalValueUnit::Future(idx, ty)
        };
        // eprintln!("Get value: {} {:?}", idx, r);
        Ok((r, idx))
    }
}

#[derive(Debug)]
pub(in crate::bitcode) struct GlobalValue {
    pub(in crate::bitcode) data: GlobalValueUnit,
    pub(in crate::bitcode) idx: usize,
}

impl GlobalValue {
    pub(in crate::bitcode) fn get_ty(&self, tag: &'static str) -> WResult<TypeBlockUnit> {
        self.data.get_ty(tag)
    }
}

impl<'a> FunctionInstrData<'a> {
    pub(super) fn next_value(
        &mut self,
        tag: &'static str,
        ty: Option<TypeBlockUnit>,
    ) -> WResult<GlobalValue> {
        let idx = self.data.next(tag)?;
        let (data, idx) = self.other.get_value(idx, ty, tag, || self.data.next(tag))?;
        Ok(GlobalValue { data, idx })
    }

    pub(super) fn next_ty(&mut self, tag: &'static str) -> WResult<TypeBlockUnit> {
        let idx = self.data.next(tag)?;
        self.other
            .data_types
            .get(idx)
            .cloned()
            .ok_or(WError::DataNotFound(tag))
    }
}

enum_function! {
    /// [instty, opty, op, align]
    19, Alloca, InstrAlloca
    /// [paramattrs, cc, fmf, fnty, fnid, arg0, arg1...]
    34, Call, InstrCall
    /// [inbounds, n x operands]
    43, Gep, InstrGEP
    /// [opty, op, align, vol]
    20, Load, InstrLoad
    /// [ptrty, ptr, valty, val, align, vol]
    44, Store, InstrStore
    /// [ty, val0,bb0, ...]
    16, Phi, InstrPhi
    /// [opcode, ty, opval, opval]
    2, BinOP, InstrBinOP
    /// [opty, opval, opval, pred]
    28, Cmp, InstrCmp
    /// [opcode, ty, opty, opval]
    3, Cast, InstrCast
    /// [ty, opval, opval, opval]
    7, InsertElt, InstrInsertElt
    /// [ty, opval, opval, opval]
    8, ShuffleVec, InstrShuffleVec
    // fcmp/icmp returning Int1TY or vector of Int1Ty. Same as CMP, exists to
    // support legacy vicmp/vfcmp instructions.
    // new select on i1 or [N x i1]
    /// [ty, opval, opval, predty, pred]
    29, VSelect, InstrVSelect
    /// [opty, opval, opval]
    6, ExtractElt, InstrExtractElt

    ;
    /// [bb#, bb#, cond] or [bb#]
    11, Br, InstrBr
    /// [opty, opval<both optional>]
    10, Ret, InstrRet
}

#[allow(clippy::declare_interior_mutable_const)]
pub(super) const TY_BOOL: LazyCell<TypeBlockUnit> =
    LazyCell::new(|| Rc::new(TypeBlockData::Integer(1)));

#[derive(Debug, Default)]
pub(in crate::bitcode) struct InstrDataFastMathFlags {
    pub(in crate::bitcode) allow_reassoc: bool,
    pub(in crate::bitcode) no_nans: bool,
    pub(in crate::bitcode) no_infs: bool,
    pub(in crate::bitcode) no_signed_zeros: bool,
    pub(in crate::bitcode) allow_reciprocal: bool,
    pub(in crate::bitcode) allow_contract: bool,
    pub(in crate::bitcode) approx_func: bool,
    pub(in crate::bitcode) all: bool,
    pub(in crate::bitcode) any: bool,
}

impl InstrDataFastMathFlags {
    pub(super) fn parse(data: usize) -> WResult<Self> {
        let allow_reassoc = (data & 1) != 0;
        let no_nans = (data & 2) != 0;
        let no_infs = (data & 4) != 0;
        let no_signed_zeros = (data & 8) != 0;
        let allow_reciprocal = (data & 16) != 0;
        let allow_contract = (data & 32) != 0;
        let approx_func = (data & 64) != 0;
        let all = allow_reassoc
            && no_nans
            && no_infs
            && no_signed_zeros
            && allow_reciprocal
            && allow_contract
            && approx_func;
        let any = allow_reassoc
            || no_nans
            || no_infs
            || no_signed_zeros
            || allow_reciprocal
            || allow_contract
            || approx_func;
        if !any {
            return Err(WError::DataNotAllowed(
                "parse call fmf",
                format!("data: {:b}", data),
            ));
        }
        Ok(Self {
            allow_reassoc,
            no_nans,
            no_infs,
            no_signed_zeros,
            allow_reciprocal,
            allow_contract,
            approx_func,
            all,
            any,
        })
    }
}

pub(super) fn parse_alignment(data: u8) -> WResult<u16> {
    Ok(1 << (data - 1))
}

impl DataNext<usize> {
    pub(super) fn next_alignment(&mut self) -> WResult<u16> {
        let t = self.next("alignment")? as u8;
        parse_alignment(t)
    }
}

const DECLAREBLOCKS: u32 = 1; // [n]

#[derive(Debug)]
pub(in crate::bitcode) struct FunctionBlock {
    pub(in crate::bitcode) base: Rc<Function>,
    pub(in crate::bitcode) metas: MetaData,
    pub(in crate::bitcode) blocks: Vec<Vec<FunctionInstr>>,
    pub(in crate::bitcode) symtab: ValueSymTabBlock,
    pub(in crate::bitcode) constants: Vec<Rc<DataSingle>>,
    pub(in crate::bitcode) vars: GlobalValueList,
    pub(in crate::bitcode) idxs_instr: Vec<HashMap<usize, usize>>,
    pub(in crate::bitcode) idx_start: usize,
}

impl<F: Read + Seek> BitReader<F> {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::bitcode) fn parse_function_block(
        &mut self,
        mut state: StateBlock,
        function: Rc<Function>,
        mut metadata: MetaData,
        mut named_metadata: NamedMetaData,
        mut value_list: GlobalValueList,
        metadata_kind: &MetaDataKindBlock,
        block_infos: &BlockInfoAll,
        data_types: &TypeBlock,
        param_attrs: &ParamAttrBlock,
        data_basic: &ModuleBlockBasic,
    ) -> WResult<FunctionBlock> {
        let idx_start = value_list.len();
        // args
        for arg in function.ty.paramty.clone().into_iter() {
            value_list.push(GlobalValueUnit::Instr(arg, usize::MAX, value_list.len()));
        }
        // eprintln!("function block value list: {} {:?}", value_list.len(), value_list[9]);
        let mut constants = Vec::with_capacity(64);
        let mut basic_blocks = Vec::with_capacity(64);
        let mut value_symtab = None;
        let mut block_now = Vec::with_capacity(64);
        let mut idxs_instr = Vec::with_capacity(64);
        let mut idxs_block = HashMap::with_capacity(128);
        let mut idx = 0;
        while self.has_data() {
            let entry = self.parse_entry::<usize>(&mut state, block_infos)?;
            // eprintln!("parse function block ... {:?} {}", entry, state.code_size);
            match entry {
                BitCodeEntry::Record(record) => {
                    // eprintln!("function record: {}", record.code);
                    if record.code == DECLAREBLOCKS {
                        let n = record
                            .data
                            .first()
                            .cloned()
                            .ok_or(WError::DataNotFound("declare blocks num"))?;
                        basic_blocks.reserve(n);
                        continue;
                    }
                    let data = FunctionInstrData {
                        data: DataNext::from(record.data),
                        chars: record.chars,
                        blob: record.blob,
                        other: FunctionInstrDataOther {
                            idx,
                            data_types,
                            param_attrs,
                            data_basic,
                            values: &value_list,
                            metadata: &metadata,
                        },
                    };
                    let instr = FunctionInstr::new(record.code, data)?;
                    // eprintln!("    Instr: {:?}", instr);
                    let is_terminator = instr.is_terminator();
                    if let Some(v) = instr.res() {
                        idxs_block.insert(idx, value_list.len());
                        value_list.push(GlobalValueUnit::Instr(v.clone(), basic_blocks.len(), idx));
                    }
                    idx += 1;
                    block_now.push(instr);
                    if is_terminator {
                        let mut block_temp = take(&mut block_now);
                        block_temp.shrink_to_fit();
                        basic_blocks.push(block_temp);
                        let mut idxs_block_temp = take(&mut idxs_block);
                        idxs_block_temp.shrink_to_fit();
                        idxs_instr.push(idxs_block_temp);
                        idx = 0;
                    }
                }
                BitCodeEntry::SubBlock(id, state_sub) => {
                    // eprintln!("function sub: {:?}", id);
                    match id {
                        BlockIDs::ConstantsBlock => {
                            let t =
                                self.parse_constants_block(state_sub, block_infos, data_types)?;
                            value_list.extend(t.clone().into_iter().map(GlobalValueUnit::Data));
                            // eprintln!("Constant: {}", value_list.len());
                            constants.extend(t);
                        }
                        BlockIDs::MetaDataBlock => {
                            self.parse_metadata_block(
                                state_sub,
                                block_infos,
                                data_types,
                                &value_list,
                                &mut metadata,
                                &mut named_metadata,
                            )?;
                        }
                        BlockIDs::ValueSymtabBlock => {
                            let t = self.parse_value_symtab_block_normal(
                                state_sub,
                                block_infos,
                                &mut value_list,
                            )?;
                            if t.basic_block.len() != basic_blocks.len() {
                                return Err(WError::Num(
                                    "Basic Block Names",
                                    false,
                                    t.basic_block.len(),
                                    basic_blocks.len(),
                                ));
                            }
                            value_symtab.replace(t);
                        }
                        BlockIDs::MetaDataAttachment => {
                            self.parse_metadata_attachment_block(
                                state_sub,
                                block_infos,
                                &metadata,
                                metadata_kind,
                            )?;
                            // value_list.extend(t.clone()
                            //     .into_iter()
                            //     .map(ValueUnit::Constant));
                        }
                        BlockIDs::UselistBlock => {
                            self.parse_uselist_block(state_sub, block_infos)?;
                        }
                        _ => return Err(WError::BlockNotAllowed("Function block", id)),
                    }
                }
                BitCodeEntry::EndBlock => break,
            }
        }
        idxs_instr.shrink_to_fit();
        Ok(FunctionBlock {
            base: function,
            metas: metadata,
            blocks: basic_blocks,
            symtab: value_symtab.unwrap_or_default(),
            constants,
            vars: value_list,
            idxs_instr,
            idx_start
        })
    }
}

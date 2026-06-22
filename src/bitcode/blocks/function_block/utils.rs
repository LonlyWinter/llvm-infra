//! Function Block
//! https://llvm.org/docs/BitCodeFormat.html#function-block-contents

use std::cell::LazyCell;
use std::collections::HashMap;
use std::io::{Read, Seek};
use std::mem::take;
use std::rc::Rc;

use crate::bitcode::blocks::blockinfo::BlockInfoAll;
use crate::bitcode::blocks::constants::{ConstantInlineASM, DataSingle};
use crate::bitcode::blocks::function_block::inst_alloca::InstrAlloca;
use crate::bitcode::blocks::function_block::inst_atomicrmw::InstrAtomicRMW;
use crate::bitcode::blocks::function_block::inst_binop::InstrBinOP;
use crate::bitcode::blocks::function_block::inst_br::InstrBr;
use crate::bitcode::blocks::function_block::inst_call::InstrCall;
use crate::bitcode::blocks::function_block::inst_cast::InstrCast;
use crate::bitcode::blocks::function_block::inst_cmp::InstrCmp;
use crate::bitcode::blocks::function_block::inst_cmpxchg::InstrCmpxchg;
use crate::bitcode::blocks::function_block::inst_extractelt::InstrExtractElt;
use crate::bitcode::blocks::function_block::inst_extractval::InstrExtractVal;
use crate::bitcode::blocks::function_block::inst_fence::InstrFence;
use crate::bitcode::blocks::function_block::inst_freeze::InstrFreeze;
use crate::bitcode::blocks::function_block::inst_gep::InstrGEP;
use crate::bitcode::blocks::function_block::inst_insertelt::InstrInsertElt;
use crate::bitcode::blocks::function_block::inst_insertval::InstrInsertVal;
use crate::bitcode::blocks::function_block::inst_invoke::InstrInvoke;
use crate::bitcode::blocks::function_block::inst_landingpad::InstrLandingPad;
use crate::bitcode::blocks::function_block::inst_load::InstrLoad;
use crate::bitcode::blocks::function_block::inst_loadatomic::InstrLoadAtomic;
use crate::bitcode::blocks::function_block::inst_phi::InstrPhi;
use crate::bitcode::blocks::function_block::inst_resume::InstrResume;
use crate::bitcode::blocks::function_block::inst_ret::InstrRet;
use crate::bitcode::blocks::function_block::inst_shufflevec::InstrShuffleVec;
use crate::bitcode::blocks::function_block::inst_store::InstrStore;
use crate::bitcode::blocks::function_block::inst_storeatomic::InstrStoreAtomic;
use crate::bitcode::blocks::function_block::inst_switch::InstrSwitch;
use crate::bitcode::blocks::function_block::inst_unop::InstrUnOp;
use crate::bitcode::blocks::function_block::inst_unreachble::InstrUnreachble;
use crate::bitcode::blocks::function_block::inst_vselect::InstrVSelect;
use crate::bitcode::blocks::function_record::Function;
use crate::bitcode::blocks::metadata::MetaDataBlock;
use crate::bitcode::blocks::metadata_attachment::MetaDataAttachmentBlock;
use crate::bitcode::blocks::metadata_kind::MetaDataKindBlock;
use crate::bitcode::blocks::moduleblock::{GlobalValueList, GlobalValueUnit, ModuleBlockBasic};
use crate::bitcode::blocks::param_attr::ParamAttrBlock;
use crate::bitcode::blocks::sync_scope_names::SyncScopeBlock;
use crate::bitcode::blocks::typeblock::{
    TypeBlock, TypeBlockData, TypeBlockFunction, TypeBlockUnit,
};
use crate::bitcode::blocks::value_symtab::ValueSymTabBlock;
use crate::bitcode::entry::DataNext;
use crate::bitcode::parse::StateBlock;
use crate::meta::FastMathFlags;
use crate::{WError, WResult};

use crate::{
    bitcode::{abbrev::BlockIDs, entry::BitCodeEntry},
    filebit::BitReader,
};

macro_rules! enum_function {
    (
        // 内部是哪个类型数据，内部类型数据有个new方法
        $(#[doc = $doc1:expr] $code1:pat, $instr1:ident, $ty1:ident)*
        ;
        // 同上，但是是终结指令
        $(#[doc = $doc2:expr] $code2:pat, $instr2:ident, $ty2:ident)*
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
                // eprintln!("instr: {} {}", data.other.idx, code);
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
                    _ => return Err(WError::CodeNotAllowed("function instr", code))
                };
                Ok(res)
            }
        }
    };
}

pub(super) struct FunctionInstrDataOther<'a> {
    pub(super) data_types: &'a TypeBlock,
    pub(super) param_attrs: &'a ParamAttrBlock,
    pub(super) data_basic: &'a ModuleBlockBasic,
    pub(super) sync_scope_names: &'a SyncScopeBlock,
    pub(super) values: &'a GlobalValueList,
}
pub(super) struct FunctionInstrData<'a> {
    pub(super) data: DataNext<usize>,
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
    ) -> WResult<GlobalValue> {
        // eprintln!("Get value: {} {} {}", self.data_basic.use_relative_ids, self.values.len(), idx);
        let idx = if self.data_basic.use_relative_ids {
            let idx = idx as u32;
            let len = self.values.len() as u32;
            len.wrapping_sub(idx) as usize
        } else {
            idx
        };
        // eprintln!("Get value: {} {:?}", idx, val);
        let r = if let Some(ty) = ty {
            if let TypeBlockData::MetaData = ty.as_ref() {
                return Ok(GlobalValue {
                    data: GlobalValueUnit::Meta(idx),
                    idx: usize::MAX
                });
            }
            match self.values.get(idx).cloned() {
                Some(data) => {
                    let ty_now = data.get_ty(tag)?;
                    if ty_now == ty 
                        || matches!(ty.as_ref(), TypeBlockData::OpaquePointer(_))
                        || matches!(ty_now.as_ref(), TypeBlockData::OpaquePointer(_))
                        || (
                            matches!(ty_now.as_ref(), TypeBlockData::Pointer(_) | TypeBlockData::OpaquePointer(_))
                            && matches!(ty.as_ref(), TypeBlockData::Pointer(_) | TypeBlockData::OpaquePointer(_))
                        ) {
                        data
                    } else {
                        return Err(WError::DataNotAllowed(
                            tag,
                            format!("ty: {:?}, data: {:?}", ty, data)
                        ))
                    }
                },
                None => GlobalValueUnit::Future(idx, ty),
            }
        } else if let Some(val) = self.values.get(idx).cloned() {
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
        Ok(GlobalValue { data: r, idx })
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
    pub(super) fn next_alignment(&mut self) -> u16 {
        if let Ok(align) = self.data.next_alignment("align") {
            align
        } else {
            self.other.data_basic.data_layout.align_stack
        }
    }

    pub(super) fn next_value(
        &mut self,
        tag: &'static str,
        ty: Option<TypeBlockUnit>,
    ) -> WResult<GlobalValue> {
        let idx = self.data.next(tag)?;
        self.other.get_value(idx, ty, tag, || self.data.next(tag))
    }

    pub(super) fn next_ty(&mut self, tag: &'static str) -> WResult<TypeBlockUnit> {
        let idx = self.data.next(tag)?;
        self.other
            .data_types
            .get(idx)
            .cloned()
            .ok_or(WError::DataNotFound(tag))
    }

    pub(super) fn next_func_name(
        &mut self,
        func_ty: &mut Option<Rc<TypeBlockFunction>>,
        tag: &'static str,
    ) -> WResult<InstrCallFunc> {
        let func_name = self.next_value(tag, None)?;
        let r = match &func_name.data {
            GlobalValueUnit::Data(vv) if let DataSingle::Inlineasm(_, v) = vv.as_ref() => {
                InstrCallFunc::Asm(v.clone())
            }
            GlobalValueUnit::Data(v) => {
                let t = v.convert_string()?;
                InstrCallFunc::Func(Rc::new(t))
            }
            GlobalValueUnit::Function(f) => {
                if func_ty.is_none() {
                    func_ty.replace(f.ty.clone());
                }
                InstrCallFunc::Func(f.name.clone())
            }
            GlobalValueUnit::Instr(_, idx_block, idx_instr) => {
                InstrCallFunc::VirtualFunc(*idx_block, *idx_instr)
            }
            _ => return Err(WError::DataNotAllowed(tag, format!("{:?}", func_name))),
        };
        Ok(r)
    }
}

enum_parse! {
    pub(in crate::bitcode) InstrAtomicOrdering, usize;
    Debug, PartialEq, Clone, ;
    NotAtomic = 0,
    Unordered = 1,
    Monotonic = 2,
    Acquire = 3,
    Release = 4,
    AcquireRelease = 5,
    SequentiallyConsistent = 6,
}

pub(super) const TY_BOOL: LazyCell<TypeBlockUnit> =
    LazyCell::new(|| Rc::new(TypeBlockData::Integer(1)));

impl FastMathFlags {
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
                "call fmf",
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

#[derive(Debug)]
pub(in crate::bitcode) enum InstrCallFunc {
    Func(Rc<String>),
    /// 虚拟函数，idx_block, idx_instr
    VirtualFunc(usize, usize),
    Asm(Rc<ConstantInlineASM>),
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
    /// [opty, op, align, vol, ordering, syncscope]
    41, LoadAtomic, InstrLoadAtomic
    // INST_STOREATOMIC = 45, [ptrty, ptr, val, align, vol]
    // INST_STOREATOMIC_OLD = 42, [ptrty, ptr,val, align, vol, ordering, syncscope]
    /// [ptrty, ptr, val, align, vol]
    45 | 42, StoreAtomic, InstrStoreAtomic
    /// [ptrty, ptr, cmp, val, vol, success_ordering, syncscope, failure_ordering, weak]
    46, Cmpxchg, InstrCmpxchg
    /// [n x operands]
    26, ExtractVal, InstrExtractVal
    /// [n x operands]
    27, InsertVal, InstrInsertVal
    /// [ordering, syncscope]
    36, Fence, InstrFence
    /// [ty,val,num,id0,val0...]
    47, LandingPad, InstrLandingPad
    /// [opty, opval]
    58, Freeze, InstrFreeze
    /// [ptrty, ptr, valty, val, operation, align, vol, ordering, syncscope]
    59, AtomicRMW, InstrAtomicRMW
    /// INST_UNOP = 56, // [opcode, ty, opval]
    56, UnOp, InstrUnOp
    ;
    /// [bb#, bb#, cond] or [bb#]
    11, Br, InstrBr
    /// [opty, opval<both optional>]
    10, Ret, InstrRet
    /// [opty, op0, op1, ...]
    12, Switch, InstrSwitch
    /// [attr, fnty, op0,op1, ...]
    13, Invoke, InstrInvoke
    /// INST_UNREACHABLE = 15,
    15, Unreachable, InstrUnreachble
    /// RESUME: [opval]
    39, Resume, InstrResume
}

const DECLAREBLOCKS: u32 = 1; // [n]

#[derive(Debug)]
pub(in crate::bitcode) struct FunctionBlock {
    pub(in crate::bitcode) base: Rc<Function>,
    pub(in crate::bitcode) blocks: Vec<Vec<FunctionInstr>>,
    pub(in crate::bitcode) symtab: ValueSymTabBlock,
    pub(in crate::bitcode) vars: GlobalValueList,
    pub(in crate::bitcode) metadata_attachment: MetaDataAttachmentBlock,
    pub(in crate::bitcode) metadata: MetaDataBlock,
    /// 第n个block内第几个instr对应的返回值在value list内的idx
    pub(in crate::bitcode) idxs_instr: Vec<HashMap<usize, usize>>,
    /// block/instr idx整体是第几个instr
    pub(in crate::bitcode) idxs_instr_global: HashMap<(usize, usize), usize>,
    pub(in crate::bitcode) idx_start: usize,
}

impl<F: Read + Seek> BitReader<F> {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::bitcode) fn parse_function_block(
        &mut self,
        mut state: StateBlock,
        function: Rc<Function>,
        mut value_list: GlobalValueList,
        mut metadata: MetaDataBlock,
        metadata_kind: &MetaDataKindBlock,
        block_infos: &BlockInfoAll,
        data_types: &TypeBlock,
        param_attrs: &ParamAttrBlock,
        data_basic: &ModuleBlockBasic,
        sync_scope_names: &SyncScopeBlock,
    ) -> WResult<FunctionBlock> {
        let idx_start = value_list.len();
        // args
        for arg in function.ty.paramty.clone().into_iter() {
            value_list.push(GlobalValueUnit::Instr(arg, usize::MAX, value_list.len()));
        }
        // eprintln!("function block value list: {} {:?}", value_list.len(), value_list[9]);
        let mut metadata_attachment = MetaDataAttachmentBlock::default();
        let mut basic_blocks = Vec::with_capacity(64);
        let mut value_symtab = None;
        let mut block_now = Vec::with_capacity(64);
        let mut idxs_instr = Vec::with_capacity(64);
        let mut idxs_block = HashMap::with_capacity(128);
        let mut idxs_instr_global = HashMap::with_capacity(2048);
        let mut idx: usize = 0;
        while self.has_data() {
            let entry = self.parse_entry::<usize>(&mut state, block_infos)?;
            // eprintln!("function block ... {:?} {}", entry, state.code_size);
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
                    // eprintln!("instr: {:?} {:?} {:?}", record.data, record.chars, record.blob);
                    let data = FunctionInstrData {
                        data: DataNext::from(record.data),
                        other: FunctionInstrDataOther {
                            data_types,
                            param_attrs,
                            data_basic,
                            sync_scope_names,
                            values: &value_list,
                        },
                    };
                    let instr = FunctionInstr::new(record.code, data)?;
                    // eprintln!("    Instr: {:?}", instr);
                    idxs_instr_global.insert((basic_blocks.len(), idx), idxs_instr_global.len());
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
                        }
                        BlockIDs::MetaDataBlock => {
                            self.parse_metadata_block(
                                state_sub,
                                block_infos,
                                data_types,
                                &value_list,
                                &mut metadata,
                            )?;
                        }
                        BlockIDs::ValueSymtabBlock => {
                            let t = self.parse_value_symtab_block_normal(
                                state_sub,
                                block_infos,
                                &mut value_list,
                            )?;
                            value_symtab.replace(t);
                        }
                        BlockIDs::MetaDataAttachment => {
                            metadata_attachment = self.parse_metadata_attachment_block(
                                state_sub,
                                block_infos,
                                &mut metadata,
                                metadata_kind,
                            )?;
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
        idxs_instr_global.shrink_to_fit();
        metadata.update_value()?;
        Ok(FunctionBlock {
            base: function,
            blocks: basic_blocks,
            symtab: value_symtab.unwrap_or_default(),
            vars: value_list,
            metadata_attachment,
            metadata,
            idxs_instr,
            idxs_instr_global,
            idx_start,
        })
    }
}

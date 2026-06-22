//! LLVM from BitCode

use std::{collections::HashMap, fmt::Display, path::Path, rc::Rc};

use crate::{
    BitCode, LLVM, WError, WResult,
    bitcode::blocks::{
        constants::{ConstantInlineASMDialect, DataSingle},
        function_block::{
            inst_atomicrmw::InstrAtomicRMWOp, inst_binop::InstrBinOPType,
            inst_br::InstrBr, inst_call::InstrCallTailTy, inst_cast::InstrCastType,
            inst_cmp::{InstrCmpFloatType, InstrCmpIntType, InstrCmpType},
            inst_landingpad::InstrLandingPadClauseType, inst_unop::InstrUnOpType,
            utils::{
                FunctionBlock, FunctionInstr, GlobalValue, InstrAtomicOrdering,
                InstrCallFunc,
            }
        },
        function_record::{Function, FunctionCallingConv},
        globalvar::{
            GlobalVarDllStorageClass, GlobalVarLinkAge, GlobalVarThreadLocal,
            GlobalVarUnnamedAddr, GlobalVarVisibility
        },
        metadata::{MetaDataValue, MetaDataValueSingle},
        moduleblock::{GlobalValueList, GlobalValueUnit},
        param_attr_group::ParamAttrUnit,
        typeblock::TypeBlockData,
        value_symtab::ValueSymTabUnit,
    },
    meta::{
        Arg, AttrDatas, Block, FunctionNative, FunctionProto, GlobalVar as GlobalVarMeta, GlobalVars, ID, IDGenerater, InstrCalc, InstrCalcAlloca, InstrCalcAtomicOrdering, InstrCalcAtomicRMW, InstrCalcAtomicRMWOp, InstrCalcBinOP, InstrCalcBinOPOp, InstrCalcCall, InstrCalcCast, InstrCalcCastOp, InstrCalcCmp, InstrCalcCmpFloatOp, InstrCalcCmpIntOp, InstrCalcCmpOp, InstrCalcCmpxchg, InstrCalcExtractElt, InstrCalcExtractVal, InstrCalcFence, InstrCalcFreeze, InstrCalcGEP, InstrCalcInsertElt, InstrCalcInsertVal, InstrCalcLandingPad, InstrCalcLandingPadClause, InstrCalcLandingPadClauseOp, InstrCalcLoad, InstrCalcLoadAtomic, InstrCalcShuffleVec, InstrCalcStore, InstrCalcStoreAtomic, InstrCalcUnOp, InstrCalcUnOpOp, InstrCalcVSelect, InstrFunc, InstrFuncAsm, InstrFuncAsmDialect, InstrMeta, InstrPhi, InstrPhiSrc, InstrTerm, InstrTermCondBr, InstrTermDirectBr, InstrTermInvoke, InstrTermResume, InstrTermRet, InstrTermSwitch, InstrTermSwitchCase, InstrWithMeta, MetaData, MetaDatas, Ty, TyBasic, ValBasic, Var, VarInstrBinOP, VarInstrCast, VarInstrGEP, Vars
    },
    utils::{
        ID_STATE_ATTR, ID_STATE_BASIC, ID_STATE_FUNC, ID_STATE_GLOBAL_VAR, ID_STATE_META,
        ID_STATE_META_KIND, ID_TAG_BLOCK, ID_TAG_VAR,
    },
};

#[derive(Default)]
struct IDGeneraterNow {
    idx: IDGenerater,
    idx_var_unknown: usize,
}

impl IDGeneraterNow {
    fn reset_var_idx(&mut self) {
        self.idx_var_unknown = 0;
    }

    fn get_block(&mut self, func_name: &str, idx_block: usize) -> ID {
        let r = format!("{}", self.idx_var_unknown);
        self.idx_var_unknown += 1;
        self.idx.generate_with_id(
            format!("{}_{}_{}", ID_TAG_BLOCK, func_name, idx_block),
            &r
        )
    }

    /// idx in value list
    fn get_var(
        &mut self,
        idx_var: usize,
        func_name: &str,
        var: Option<&Var>,
        symtab: Option<&ValueSymTabUnit>,
    ) -> WResult<ID> {
        let name = format!("{func_name}@@@{idx_var}");
        let r = if let Some(Var::MetaData(Some(r))) = var {
            r.clone()
        } else if let Some(v) = var && v.is_const() {
            self.idx.generate_const(format!("{:?}", v))
        } else if let Ok(r) = self.idx.get(ID_TAG_VAR, &name) {
            r
        } else if let Some(symtab) = symtab {
            let r = symtab.get(&idx_var).cloned().unwrap_or_else(|| {
                let r = format!("{}", self.idx_var_unknown);
                self.idx_var_unknown += 1;
                r
            });
            self.idx.generate_with_id(
                format!("{}_{}", ID_TAG_VAR, name),
                &r
            )
        } else {
            return Err(WError::DataNotAllowed("get var", format!("{}", idx_var)));
        };
        Ok(r)
    }
}

impl TryFrom<&TypeBlockData> for Ty {
    type Error = WError;
    fn try_from(data: &TypeBlockData) -> WResult<Self> {
        let r = match data {
            TypeBlockData::Void => Self::Basic(TyBasic::Void),
            TypeBlockData::Float => Self::Basic(TyBasic::Float),
            TypeBlockData::Double => Self::Basic(TyBasic::Double),
            TypeBlockData::Label => Self::Basic(TyBasic::Label),
            TypeBlockData::Half => Self::Basic(TyBasic::Half),
            TypeBlockData::MetaData => Self::MetaData,
            TypeBlockData::Integer(v) => Self::Basic(TyBasic::Int(*v)),
            TypeBlockData::Pointer(v) => {
                let ty = Self::try_from(v.ty.as_ref())?;
                Self::Pointer(Some(Box::new(ty)))
            }
            TypeBlockData::OpaquePointer(_) | TypeBlockData::Function(_) => {
                Self::Pointer(None)
            }
            TypeBlockData::Array(v) => {
                let ty = Self::try_from(v.ty.as_ref())?;
                let num = v.num;
                Self::Array(Box::new(ty), num)
            }
            TypeBlockData::Vector(v) => {
                let ty = Self::try_from(v.ty.as_ref())?;
                let num = v.num;
                Self::Vector(Box::new(ty), num)
            }
            TypeBlockData::StructAnon(v) => {
                let tys = v.tys.iter()
                    .map(|v| Self::try_from(v.as_ref()))
                    .collect::<WResult<Vec<_>>>()?;
                Self::Struct(tys, v.is_packed)
            }
            TypeBlockData::StructNamed(v) => {
                let tys = v.tys.iter()
                    .map(|v| Self::try_from(v.as_ref()))
                    .collect::<WResult<Vec<_>>>()?;
                Self::Struct(tys, v.is_packed)
            }
            _ => {
                return Err(WError::DataNotAllowed(
                    "convert type block to ty",
                    format!("{data:?}"),
                ));
            }
        };
        Ok(r)
    }
}

#[derive(Debug)]
enum Instr {
    Calc(Box<InstrCalc>),
    Term(InstrTerm),
    Phi(InstrPhi),
}


macro_rules! enum_convert {
    (
        $src:expr;
        // basic integer: name_data, name_val, bit
        $($name11:ident = $name12:ident = $bit1:pat,)*;
        // basic float: name_data, name_ty, name_val
        $($name21:ident = $name22:ident = $name23:ident,)*;
        // array/vector integer
        $($name31:ident = $name32:ident = $name33:ident = $bit3:pat,)*;
        // array/vector float
        $($name41:ident = $name42:ident = $name43:ident = $name44:ident,)*;
        // simple
        $($name5:ident,)*
    ) => {
        match $src {
            $(
                DataSingle::$name11(ty, val) if let TypeBlockData::Integer(bit) = ty.as_ref()
                    && matches!(bit, $bit1) => {
                    let r = Var::Basic(TyBasic::Int(*bit), Some(ValBasic::$name12(*val)));
                    return Ok(r)
                },
            )*
            $(
                DataSingle::$name21(ty, val) if let TypeBlockData::$name22 = ty.as_ref() => {
                    let r = Var::Basic(TyBasic::$name22, Some(ValBasic::$name23(*val)));
                    return Ok(r)
                },
            )*
            $(
                DataSingle::$name31(ty, val) if let TypeBlockData::$name32(ty_inner) = ty.as_ref()
                    && let TypeBlockData::Integer(bit) = ty_inner.ty.as_ref()
                     && matches!(bit, $bit3)
                    && (val.len() == ty_inner.num) => {
                    let vs = val.into_iter()
                        .map(| v | Var::Basic(TyBasic::Int(*bit), Some(ValBasic::$name33(*v))))
                        .collect();
                    let r = Var::$name32(Ty::Basic(TyBasic::Int(*bit)), vs);
                    return Ok(r)
                },
            )*
            $(
                DataSingle::$name41(ty, val) if let TypeBlockData::$name42(ty_inner) = ty.as_ref()
                    && let TypeBlockData::$name43 = ty_inner.ty.as_ref()
                    && (val.len() == ty_inner.num) => {
                    let vs = val.into_iter()
                        .map(| v | Var::Basic(TyBasic::$name43, Some(ValBasic::$name44(*v))))
                        .collect();
                    let r = Var::$name42(Ty::Basic(TyBasic::$name43), vs);
                    return Ok(r)
                },
            )*
            $(
                DataSingle::$name5(v) => {
                    let ty = Ty::try_from(v.as_ref())?;
                    let r = Var::$name5(ty);
                    return Ok(r);
                },

            )*
            _ => {},
        }
    };
    
    (
        "type_to_op", $name1:ident, $name2:ident;
        // no related data
        $($var1:ident,)*;
        // one data
        $($var2:ident,)*;
        // two data
        $($var3:ident,)*
    ) => {
        impl From<$name1> for $name2 {
            fn from(value: $name1) -> Self {
                match value {
                    $(
                        $name1::$var1 => Self::$var1,
                    )*
                    $(
                        $name1::$var2(v) => Self::$var2(v),
                    )*
                    $(
                        $name1::$var3((v1, v2)) => Self::$var3(v1, v2),
                    )*
                }
            }
        }
    };

    (
        "op_to_str", $name:ident;
        // no related data
        $($id1:ident = $val1:expr)*;
        // on data but ignore
        $($id2:ident = $val2:expr)*
    ) => {
        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$id1 => write!(f, $val1),
                    )*
                    $(
                        Self::$id2(_) => write!(f, $val2),
                    )*
                }
            }
        }
    };
}

fn parse_value_to_var(
    data: &GlobalValueUnit,
    func_name: &str,
    value_list: &GlobalValueList,
    meta_records: &HashMap<String, String>,
    ids_global: &mut IDGeneraterNow,
    vars_global: &mut GlobalVars,
    data_idx_var: &HashMap<(usize, usize), usize>,
) -> WResult<Var> {
    let r = match data {
        GlobalValueUnit::Data(data) => {
            enum_convert!(data.as_ref();
                Integer1 = Bool = 1,
                Integer8 = I8 = 2..=8,
                Integer16 = I16 = 9..=16,
                Integer32 = I32 = 17..=32,
                Integer64 = I64 = 33..=64,
                Wide128 = I128 = 65..=128,
                ;
                F16 = Half = F16,
                F32 = Float = F32,
                F64 = Double = F64,
                ;
                ArrayBool = Array = Bool = 1, VectorBool = Vector = Bool = 1,
                ArrayU8 = Array = I8 = 2..=8, VectorU8 = Vector = I8 = 2..=8,
                ArrayU16 = Array = I16 = 9..=16, VectorU16 = Vector = I16 = 9..=16,
                ArrayU32 = Array = I32 = 17..=32, VectorU32 = Vector = I32 = 17..=32,
                ArrayU64 = Array = I64 = 33..=64, VectorU64 = Vector = I64 = 33..=64,
                ArrayU128 = Array = I128 = 65..=128, VectorU128 = Vector = I128 = 65..=128,
                ;
                ArrayF16 = Array = Half = F16,
                VectorF16 = Vector = Half = F16,
                ArrayF32 = Array = Float = F32,
                VectorF32 = Vector = Float = F32,
                ArrayF64 = Array = Double = F64,
                VectorF64 = Vector = Double = F64,
                ;
                Null, Poison, Undef,
            );
            match data.as_ref() {
                DataSingle::AggVector(ty, idxs)
                    if let TypeBlockData::Vector(ty_arr) = ty.as_ref()
                    && ty_arr.num == idxs.len() => {
                    let ty = Ty::try_from(ty_arr.ty.as_ref())?;
                    let vs = idxs.iter()
                        .map(| idx | parse_idx_to_var(
                            *idx,
                            func_name,
                            value_list,
                            meta_records,
                            ids_global,
                            vars_global,
                            data_idx_var
                        ))
                        .collect::<WResult<Vec<_>>>()?;
                    Var::Vector(ty, vs)
                },
                DataSingle::AggArray(ty, idxs)
                    if let TypeBlockData::Array(ty_arr) = ty.as_ref()
                    && ty_arr.num == idxs.len() => {
                    let ty = Ty::try_from(ty_arr.ty.as_ref())?;
                    let vs = idxs.iter()
                        .map(| idx | parse_idx_to_var(
                            *idx,
                            func_name,
                            value_list,
                            meta_records,
                            ids_global,
                            vars_global,
                            data_idx_var
                        ))
                        .collect::<WResult<Vec<_>>>()?;
                    Var::Array(ty, vs)
                },
                DataSingle::AggStruct(ty, idxs) => {
                    let vs = idxs.iter()
                        .map(| idx | parse_idx_to_var(
                            *idx,
                            func_name,
                            value_list,
                            meta_records,
                            ids_global,
                            vars_global,
                            data_idx_var
                        ))
                        .collect::<WResult<Vec<_>>>()?;
                    let is_packed = match ty.as_ref() {
                        TypeBlockData::StructNamed(ty_struct) => ty_struct.is_packed,
                        TypeBlockData::StructAnon(ty_struct) => ty_struct.is_packed,
                        _ => return Err(WError::DataNotAllowed(
                            "AggStruct to var",
                            format!("{:?}", ty)
                        ))
                    };
                    Var::Struct(is_packed, vs)
                },
                DataSingle::String(ty, val) if
                    let TypeBlockData::Array(ty_arr) = ty.as_ref()
                    && val.len() == ty_arr.num => {
                    Var::String(false, val.clone())
                },
                DataSingle::CString(ty, val) if
                    let TypeBlockData::Array(ty_arr) = ty.as_ref()
                    && (val.len() + 1) == ty_arr.num => {
                    Var::String(true, val.clone())
                },
                DataSingle::Cast(ty, instr) => {
                    let ty_res = Ty::try_from(ty.as_ref())?;
                    let op = InstrCalcCastOp::from(instr.op.clone());
                    let val = parse_idx_to_var(
                        instr.val,
                        func_name,
                        value_list,
                        meta_records,
                        ids_global,
                        vars_global,
                        data_idx_var
                    )?;
                    let ty = Ty::try_from(instr.ty_res.as_ref())?;
                    if !matches!(ty_res, Ty::Pointer(None)) && ty != ty_res {
                        return Err(WError::DataNotAllowed(
                            "cast res ty",
                            format!("{:?} {:?}", ty, instr)
                        ))
                    }
                    Var::InstrCast(VarInstrCast {
                        op,
                        ty,
                        val: Box::new(val)
                    })
                },
                DataSingle::Gep(ty, instr) => {
                    let ty_res = Ty::try_from(ty.as_ref())?;
                    if instr.elts.len() != 2 {
                        return Err(WError::DataNotAllowed(
                            "gep num",
                            format!("{:?}", instr.elts)
                        ))
                    }
                    let base = parse_idx_to_var(
                        instr.elts[0].1,
                        func_name,
                        value_list,
                        meta_records,
                        ids_global,
                        vars_global,
                        data_idx_var
                    )?;
                    let idx = parse_idx_to_var(
                        instr.elts[1].1,
                        func_name,
                        value_list,
                        meta_records,
                        ids_global,
                        vars_global,
                        data_idx_var
                    )?;
                    let ty = Ty::try_from(instr.ty.as_ref())?;
                    if !matches!(ty_res, Ty::Pointer(None)) && ty != ty_res {
                        return Err(WError::DataNotAllowed(
                            "gep res ty",
                            format!("{:?} {:?}", ty, ty_res)
                        ))
                    }
                    Var::InstrGEP(VarInstrGEP {
                        in_bounds: instr.flag.in_bounds,
                        nusw: instr.flag.nusw,
                        nuw: instr.flag.nuw,
                        ty,
                        base: Box::new(base),
                        idx: Box::new(idx),
                    })
                },
                DataSingle::BinOp(ty, instr) => {
                    let ty_res = Ty::try_from(ty.as_ref())?;
                    let op = InstrCalcBinOPOp::from(instr.op.clone());
                    let ty = Ty::try_from(instr.ty.as_ref())?;
                    if !matches!(ty_res, Ty::Pointer(None)) && ty != ty_res {
                        return Err(WError::DataNotAllowed(
                            "binop res ty",
                            format!("{:?} {:?}", ty, ty_res)
                        ))
                    }
                    let left = parse_idx_to_var(
                        instr.left,
                        func_name,
                        value_list,
                        meta_records,
                        ids_global,
                        vars_global,
                        data_idx_var
                    )?;
                    let right = parse_idx_to_var(
                        instr.right,
                        func_name,
                        value_list,
                        meta_records,
                        ids_global,
                        vars_global,
                        data_idx_var
                    )?;
                    Var::InstrBinOP(VarInstrBinOP {
                        op,
                        ty,
                        left: Box::new(left),
                        right: Box::new(right),
                    })
                },
                data => return Err(WError::DataNotAllowed(
                    "value_to_var",
                    format!("{:?}", data)
                ))
            }
        },
        GlobalValueUnit::Meta(v) => {
            let k = format!("MetaDataRaw@{func_name}@{v}");
            let name = meta_records.get(&k)
                .ok_or(WError::DataNotAllowed(
                    "meta record name",
                    k
                ))?;
            let id = ids_global.idx.get(
                ID_STATE_META,
                name
            )?;
            Var::MetaData(Some(id))
        }
        GlobalValueUnit::GlobalVar(var) => {
            let var = var.as_ref();
            if let Ok(name) = ids_global.idx.get(
                ID_STATE_GLOBAL_VAR,
                &var.name
            ) {
                return Ok(Var::Ptr(name));
            }
            let name = ids_global.idx.generate(
                ID_STATE_GLOBAL_VAR,
                &var.name
            );
            let val = if var.init_id == 0 {
                let ty = Ty::try_from(var.ty.as_ref())?;
                Var::Null(ty)
            } else {
                parse_idx_to_var(
                    var.init_id - 1,
                    func_name,
                    value_list,
                    meta_records,
                    ids_global,
                    vars_global,
                    data_idx_var,
                )?
            };
            if !vars_global.contains_key(&name) {
                let link_age = ids_global
                    .idx
                    .generate(ID_STATE_ATTR, &var.link_age.to_string());
                let visibility = ids_global
                    .idx
                    .generate(ID_STATE_ATTR, &var.visibility.to_string());
                let unnamed_addr = ids_global
                    .idx
                    .generate(ID_STATE_ATTR, &var.unnamed_addr.to_string());
                let dll_storage_class = ids_global
                    .idx
                    .generate(ID_STATE_ATTR, &var.dll_storage_class.to_string());
                let thread_local = ids_global
                    .idx
                    .generate(ID_STATE_ATTR, &var.thread_local.to_string());

                let var = GlobalVarMeta {
                    is_const: var.is_const,
                    align: var.align,
                    link_age,
                    visibility,
                    dll_storage_class,
                    thread_local,
                    unnamed_addr,
                    var: val,
                };
                vars_global.insert(name.clone(), var);
            }
            Var::Ptr(name)
        }
        GlobalValueUnit::Function(func) => {
            let func = ids_global.idx.get(
                ID_STATE_FUNC,
                func.name.as_str()
            )?;
            Var::Ptr(func)
        }
        _ => todo!("convert value to val: {:?}", data),
    };
    Ok(r)
}


fn parse_idx_to_var(
    idx: usize,
    func_name: &str,
    value_list: &GlobalValueList,
    meta_records: &HashMap<String, String>,
    ids_global: &mut IDGeneraterNow,
    vars_global: &mut GlobalVars,
    data_idx_var: &HashMap<(usize, usize), usize>,
) -> WResult<Var> {
    let data = value_list.get(idx)
        .ok_or(WError::DataNotFound("idx to var"))?;
    // eprintln!("parse_idx_to_var {} {:?}", idx, data);
    parse_value_to_var(data, func_name, value_list, meta_records, ids_global, vars_global, data_idx_var)
}

enum_convert! { "type_to_op", InstrBinOPType, InstrCalcBinOPOp;
    FAdd, FSub, FMul, Urem, Srem, And, Xor,;
    Sdiv, Udiv, Lshr, Ashr, Or, Fdiv, Frem,;
    Add, Sub, Mul, Shl,
}

enum_convert! { "type_to_op", InstrCastType, InstrCalcCastOp;
    SExt, FpToUi, FpToSi, SiToFp, PtrToInt, IntToPtr, BitCast, AddrSpaceCast, PtrToAddr, ;
    ZExt, UiToFp, FpTrunc, FpExt, ;
    Trunc, 
    
}

enum_convert! { "type_to_op", InstrCmpFloatType, InstrCalcCmpFloatOp;
    False, Oeq, Ogt, Oge, Olt, Ole, One, Ord, Uno, Ueq, Ugt, Uge, Ult, Ule, Une, True, ; ; 
}

enum_convert! { "type_to_op", InstrCmpIntType, InstrCalcCmpIntOp;
    Eq, Ne, Ugt, Uge, Ult, Ule, Sgt, Sge, Slt, Sle, ; ; 
}

enum_convert! { "type_to_op", InstrAtomicOrdering, InstrCalcAtomicOrdering;
    NotAtomic, Unordered, Monotonic, Acquire, Release, AcquireRelease,
    SequentiallyConsistent,; ; 
}

enum_convert! { "type_to_op", InstrAtomicRMWOp, InstrCalcAtomicRMWOp;
    Xchg, Add, Sub, And, Nand, Or, Xor, Max, Min,
    Umax, Umin, Fadd, Fsub, Fmax, Fmin, UincWrap,
    UdecWrap, UsubCond, UsubSat, Fmaximum, Fminimum,; ; 
}


enum_convert! { "type_to_op", InstrLandingPadClauseType, InstrCalcLandingPadClauseOp;
    Catch, Filter, ; ; 
}

enum_convert! { "type_to_op", InstrUnOpType, InstrCalcUnOpOp;
    ; FNeg, ;
}

enum_convert! { "type_to_op", ConstantInlineASMDialect, InstrFuncAsmDialect;
    Intel, Att, ; ;
}

impl From<InstrCmpType> for InstrCalcCmpOp {
    fn from(value: InstrCmpType) -> Self {
        match value {
            InstrCmpType::Float(v1, v2) => Self::Float(InstrCalcCmpFloatOp::from(v1), v2),
            InstrCmpType::Int(v1, v2) => Self::Int(InstrCalcCmpIntOp::from(v1), v2),
        }
    }
}

struct InstrDataExtra<'a> {
    func_name: &'a str,
    value_list: &'a GlobalValueList,
    data_idx_var: &'a HashMap<(usize, usize), usize>,
    block_names: &'a HashMap<usize, ID>,
    meta_records: &'a HashMap<String, String>,
}

enum_convert! { "op_to_str", InstrCallTailTy;
    None = ""
    Tail = "tail"
    Must = "musttail"
    No = "notail"
    ;
}

enum_convert! { "op_to_str", FunctionCallingConv;
    C = ""
    Fast = "fastcc"
    Cold = "coldcc"
    WebKitJs = "webkit_jscc"
    AnyReg = "anyregcc"
    PreserveMost = "preserve_mostcc"
    PreserveAll = "preserve_allcc"
    PreserveNone = "preserve_nonecc"
    CxxFastTls = "cxx_fast_tlscc"
    Ghc = "ghccc"
    Tail = "tailcc"
    GRAAL = "graalcc"
    CFGuardCheck = "cfguard_checkcc"
    X86StdCall = "x86_stdcallcc"
    X86FastCall = "x86_fastcallcc"
    X86ThisCall = "x86_thiscallcc"
    X86RegCall = "x86_regcallcc"
    X86VectorCall = "x86_vectorcallcc"
    IntelOCLBI = "intel_ocl_bicc"
    ARMAPCS = "arm_apcscc"
    ARMAAPCS = "arm_aapcscc"
    ARMAAPCSVFP = "arm_aapcs_vfpcc"
    AArch64VectorCall = "aarch64_vector_pcs"
    AArch64SVEVectorCall = "aarch64_sve_vector_pcs"
    AArch64SMEABISupportRoutinesPreserveMostFromX0 = "aarch64_sme_preservemost_from_x0"
    AArch64SMEABISupportRoutinesPreserveMostFromX1 = "aarch64_sme_preservemost_from_x1"
    AArch64SMEABISupportRoutinesPreserveMostFromX2 = "aarch64_sme_preservemost_from_x2"
    MSP430INTR = "msp430_intrcc"
    AVRINTR = "avr_intrcc "
    AVRSIGNAL = "avr_signalcc "
    PTXKernel = "ptx_kernel"
    PTXDevice = "ptx_device"
    X8664SysV = "x86_64_sysvcc"
    Win64 = "win64cc"
    SPIRFUNC = "spir_func"
    SPIRKERNEL = "spir_kernel"
    Swift = "swiftcc"
    SwiftTail = "swifttailcc"
    X86INTR = "x86_intrcc"
    DUMMYHHVM = "hhvmcc"
    DUMMYHHVMC = "hhvm_ccc"
    AMDGPUVS = "amdgpu_vs"
    AMDGPULS = "amdgpu_ls"
    AMDGPUHS = "amdgpu_hs"
    AMDGPUES = "amdgpu_es"
    AMDGPUGS = "amdgpu_gs"
    AMDGPUPS = "amdgpu_ps"
    AMDGPUCS = "amdgpu_cs"
    AMDGPUCSChain = "amdgpu_cs_chain"
    AMDGPUCSChainPreserve = "amdgpu_cs_chain_preserve"
    AMDGPUKERNEL = "amdgpu_kernel"
    AMDGPUGfx = "amdgpu_gfx"
    AMDGPUGfxWholeWave = "amdgpu_gfx_whole_wave"
    M68kRTD = "m68k_rtdcc"
    RISCVVectorCall = "riscv_vector_cc"
    RISCVVLSCall32 = "riscv_vls_cc(32)"
    RISCVVLSCall64 = "riscv_vls_cc(64)"
    RISCVVLSCall128 = "riscv_vls_cc(128)"
    RISCVVLSCall256 = "riscv_vls_cc(256)"
    RISCVVLSCall512 = "riscv_vls_cc(512)"
    RISCVVLSCall1024 = "riscv_vls_cc(1024)"
    RISCVVLSCall2048 = "riscv_vls_cc(2048)"
    RISCVVLSCall4096 = "riscv_vls_cc(4096)"
    RISCVVLSCall8192 = "riscv_vls_cc(8192)"
    RISCVVLSCall16384 = "riscv_vls_cc(16384)"
    RISCVVLSCall32768 = "riscv_vls_cc(32768)"
    RISCVVLSCall65536 = "riscv_vls_cc(65536)"
    CHERIoTCompartmentCall = "cheriot_compartmentcallcc"
    CHERIoTCompartmentCallee = "cheriot_compartmentcalleecc"
    CHERIoTLibraryCall = "cheriot_librarycallcc"

    HiPE = "cc 11"
    AVRBUILTIN = "cc 35"
    MSP430BUILTIN = "cc 43"
    WASMEmscriptenInvoke = "cc 48"
    M68kINTR = "cc 50"
    ARM64ECThunkX64 = "cc 57"
    ARM64ECThunkNative = "cc 58"
    ;
}

enum_convert! { "op_to_str", InstrCalcAtomicOrdering;
    NotAtomic = ""
    Unordered = "unordered"
    Monotonic = "monotonic"
    Acquire = "acquire"
    Release = "release"
    AcquireRelease = "acq_rel"
    SequentiallyConsistent = "seq_cst"
    ;
}


enum_convert! { "op_to_str", InstrCalcAtomicRMWOp;
    Xchg = "xchg"
    Add = "add"
    Sub = "sub"
    And = "and"
    Nand = "nand"
    Or = "or"
    Xor = "xor"
    Max = "max"
    Min = "min"
    Umax = "umax"
    Umin = "umin"
    Fadd = "fadd"
    Fsub = "fsub"
    Fmax = "fmax"
    Fmin = "fmin"
    UincWrap = "uincwrap"
    UdecWrap = "udecwrap"
    UsubCond = "usubcond"
    UsubSat = "usubsat"
    Fmaximum = "fmaximum"
    Fminimum = "fminimum"
    ;
}

enum_convert! { "op_to_str", InstrCalcLandingPadClauseOp;
    Catch = "catch"
    Filter = "filter"
    ;
}

enum_convert! { "op_to_str", InstrCalcUnOpOp;
    ;
    FNeg = "fneg"
}

enum_convert! { "op_to_str", InstrFuncAsmDialect;
    Intel = "inteldialect"
    Att = ""
    ;
}
enum_convert! { "op_to_str", GlobalVarLinkAge;
    External = ""
    AvailableExternally = "available_externally"
    LinkonceOdr = "linkonce_odr"
    ExternWeak = "extern_weak"
    WeakOdr = "weak_odr"
    Weak = "weak"
    Appending = "appending"
    Internal = "internal"
    Linkonce = "linkonce"
    DllImport = "dllimport"
    DllExport = "dllexport"
    Common = "common"
    Private = "private"
    ;
}

enum_convert! { "op_to_str", GlobalVarVisibility;
    Default = ""
    Hidden = "hidden"
    Protected = "protected"
    ;
}

enum_convert! { "op_to_str", GlobalVarUnnamedAddr;
    None = ""
    Global = "unnamed_addr"
    Local = "local_unnamed_addr"
    ;
}

enum_convert! { "op_to_str", GlobalVarThreadLocal;
    NotThreadLocal = ""
    ThreadLocal = "thread_local"
    LocalDynamic = "thread_local(localdynamic)"
    InitialExec = "thread_local(initialexec)"
    LocalExec = "thread_local(locexec)"
    ;
}

enum_convert! { "op_to_str", GlobalVarDllStorageClass;
    Default = ""
    DllImport = "dllimport"
    DllExport = "dllexport"
    ;
}


fn parse_attrs(attr: &ParamAttrUnit, idx: u32, ids_global: &mut IDGeneraterNow) -> Vec<ID> {
    attr.get(&idx)
        .map(|v| {
            v.iter()
                .map(|v| v.to_string())
                .filter(|v| !v.is_empty())
                .map(|vv| ids_global.idx.generate(ID_STATE_ATTR, &vv))
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| Vec::with_capacity(0))
}

fn insert_attr(attr_list: Vec<ID>, attrs: &mut AttrDatas) -> Option<u16> {
    if attr_list.is_empty() {
        return None;
    }
    let r = attrs
        .iter()
        .find(|&(_, v)| v == &attr_list)
        .map(|(v, _)| *v)
        .unwrap_or_else(|| {
            let t = attrs.len() as u16;
            attrs.insert(t, attr_list);
            t
        });
    Some(r)
}

impl InstrFunc {
    fn parse(func: InstrCallFunc, func_name: &str, data_idx_var: &HashMap<(usize, usize), usize>, ids_global: &mut IDGeneraterNow) -> WResult<Self> {
        let func = match func {
            InstrCallFunc::Func(v) => {
                let n = ids_global.idx.get(ID_STATE_FUNC, v.as_str())?;
                Self::Func(n)
            },
            InstrCallFunc::VirtualFunc(idx_block, idx_var) if idx_block == usize::MAX => {
                let n = ids_global.get_var(
                    idx_var,
                    func_name,
                    None,
                    None
                )?;
                Self::VirtualFunc(n)
            },
            InstrCallFunc::VirtualFunc(idx_block, idx_instr) => {
                let idx_var = data_idx_var
                    .get(&(idx_block, idx_instr))
                    .cloned()
                    .ok_or(WError::DataNotFound("call virtual func instr idx"))?;
                let n = ids_global.get_var(
                    idx_var,
                    func_name,
                    None,
                    None
                )?;
                Self::VirtualFunc(n)
            },
            InstrCallFunc::Asm(asm) => {
                let ret = Ty::try_from(asm.ty.retty.as_ref())?.into();
                let args = asm.ty.paramty
                    .iter()
                    .map(|v| Ty::try_from(v.as_ref()))
                    .collect::<WResult<Vec<_>>>()?
                    .into_iter()
                    .map(| v | v.into())
                    .collect();
                let asm_dialect = InstrFuncAsmDialect::from(asm.asm_dialect.clone());
                Self::Asm(InstrFuncAsm {
                    ret,
                    args,
                    asm: Rc::new(asm.asm.clone()),
                    constr: Rc::new(asm.constr.clone()),
                    has_side_effects: asm.has_side_effects,
                    is_align_stack: asm.is_align_stack,
                    can_throw: asm.can_throw,
                    asm_dialect
                })
            }
        };
        Ok(func)
    }
}

impl Instr {
    fn parse<'a>(
        instr: FunctionInstr,
        name_res: ID,
        vars: &mut Vars,
        ids_global: &mut IDGeneraterNow,
        attrs: &mut AttrDatas,
        vars_global: &mut GlobalVars,
        data_extra: &InstrDataExtra<'a>,
    ) -> WResult<Self> {
        let mut get_var = |data: GlobalValue| {
            // eprintln!("Data: {} {:?}", data.idx, data.data);
            let (name, val) = match &data.data {
                GlobalValueUnit::Instr(_, idx_block, idx_instr) if *idx_block != usize::MAX => {
                    let idx_var = data_extra.data_idx_var
                        .get(&(*idx_block, *idx_instr))
                        .cloned()
                        .ok_or(WError::DataNotFound("value to val instr idx"))?;
                    let name = ids_global.get_var(
                        idx_var,
                        data_extra.func_name,
                        None,
                        None
                    )?;
                    // eprintln!("instr res var: {} {} {} {}", idx_block, idx_instr, idx_var, name);
                    (Some(name), None)
                },
                GlobalValueUnit::Instr(ty, _, idx_var) | GlobalValueUnit::Future(idx_var, ty) => {
                    // func arg, future var
                    let val = Ty::try_from(ty.as_ref())?.try_into()?;
                    let name = ids_global.get_var(
                        *idx_var,
                        data_extra.func_name,
                        Some(&val),
                        None
                    )?;
                    (Some(name), Some(val))
                },
                data => {
                    let var = parse_value_to_var(
                        data,
                        data_extra.func_name,
                        data_extra.value_list,
                        data_extra.meta_records,
                        ids_global,
                        vars_global,
                        data_extra.data_idx_var
                    )?;
                    if let GlobalValueUnit::GlobalVar(_) = data
                    && let Var::Ptr(name) = var {
                        (Some(name), None)
                    } else {
                        (None, Some(var))
                    }
                }
            };

            // eprintln!("Val raw: {} {:?} {:?}", data.idx, name, val);
            let name = if let Some(name) = name {
                name
            } else {
                ids_global.get_var(
                    data.idx,
                    data_extra.func_name,
                    val.as_ref(),
                    None
                )?
            };
            // eprintln!("Val: {} {:?}", name, val);
            if let Some(val) = val
                && !vars.contains_key(&name)
            {
                vars.insert(name.clone(), val);
            }
            Ok::<_, WError>(name)
        };
        // eprintln!("Convert instr: {:?}", instr);
        let r = match instr {
            FunctionInstr::BinOP(instr) => {
                let op = InstrCalcBinOPOp::from(instr.op);
                let left = get_var(instr.left)?;
                let right = get_var(instr.right)?;
                Self::Calc(Box::new(InstrCalc::BinOp(InstrCalcBinOP {
                    res: name_res,
                    op,
                    left,
                    right,
                })))
            },
            FunctionInstr::Gep(instr) => {
                let base = get_var(instr.base)?;
                let idx = get_var(instr.idx)?;
                if let Some(val) = vars.remove(&name_res) {
                    vars.insert(name_res.clone(), val);
                }
                Self::Calc(Box::new(InstrCalc::Gep(InstrCalcGEP {
                    in_bounds: instr.no_wrap_flags.in_bounds,
                    nusw: instr.no_wrap_flags.nusw,
                    nuw: instr.no_wrap_flags.nuw,
                    res: name_res,
                    base,
                    idx,
                })))
            },
            FunctionInstr::Cast(instr) => {
                let op = InstrCalcCastOp::from(instr.op);
                let val = get_var(instr.val)?;
                Self::Calc(Box::new(InstrCalc::Cast(InstrCalcCast {
                    res: name_res,
                    op,
                    val,
                })))
            },
            FunctionInstr::Cmp(instr) => {
                let op = InstrCalcCmpOp::from(instr.op);
                let left = get_var(instr.left)?;
                let right = get_var(instr.right)?;
                Self::Calc(Box::new(InstrCalc::Cmp(InstrCalcCmp {
                    res: name_res,
                    op,
                    left,
                    right,
                })))
            },
            FunctionInstr::ExtractElt(instr) => {
                let src = get_var(instr.src)?;
                let idx = get_var(instr.idx)?;
                Self::Calc(Box::new(InstrCalc::ExtractElt(InstrCalcExtractElt {
                    res: name_res,
                    src,
                    idx,
                })))
            },
            FunctionInstr::InsertElt(instr) => {
                let src = get_var(instr.src)?;
                let elt = get_var(instr.elt)?;
                let idx = get_var(instr.idx)?;
                Self::Calc(Box::new(InstrCalc::InsertElt(InstrCalcInsertElt {
                    res: name_res,
                    src,
                    elt,
                    idx,
                })))
            },
            FunctionInstr::ShuffleVec(instr) => {
                let vec1 = get_var(instr.vec1)?;
                let vec2 = get_var(instr.vec2)?;
                let mask = get_var(instr.mask)?;
                Self::Calc(Box::new(InstrCalc::ShuffleVec(InstrCalcShuffleVec {
                    res: name_res,
                    vec1,
                    vec2,
                    mask,
                })))
            },
            FunctionInstr::VSelect(instr) => {
                let cond = get_var(instr.cond)?;
                let val_true = get_var(instr.val_true)?;
                let val_false = get_var(instr.val_false)?;
                Self::Calc(Box::new(InstrCalc::VSelect(InstrCalcVSelect {
                    res: name_res,
                    cond,
                    val_true,
                    val_false,
                    fmf: instr.fmf
                })))
            },
            FunctionInstr::Phi(instr) => {
                // eprintln!("Phi raw: {:?}", instr);
                let srcs = instr
                    .srcs
                    .into_iter()
                    .map(|(var, idx_block)| {
                        let var = get_var(var)?;
                        Ok::<_, WError>((var, idx_block))
                    })
                    .collect::<WResult<Vec<_>>>()?
                    .into_iter()
                    .map(| (var, idx_block) | {
                        let block = data_extra.block_names
                            .get(&idx_block).cloned()
                            .ok_or(WError::DataNotFound("block name"))?;
                        Ok(InstrPhiSrc { block, var })
                    })
                    .collect::<WResult<Vec<_>>>()?;
                // eprintln!("Phi srcs: {:?}", srcs);
                Self::Phi(InstrPhi {
                    res: name_res,
                    srcs,
                })
            },
            FunctionInstr::Load(instr) => {
                let src = get_var(instr.op)?;
                Self::Calc(Box::new(InstrCalc::Load(InstrCalcLoad {
                    res: name_res,
                    src,
                    align: instr.align,
                })))
            },
            FunctionInstr::Store(instr) => {
                let dst = get_var(instr.ptr)?;
                let data = get_var(instr.val)?;
                Self::Calc(Box::new(InstrCalc::Store(InstrCalcStore {
                    data,
                    dst,
                    is_volatile: instr.is_volatile,
                    align: instr.align,
                })))
            },
            FunctionInstr::Br(InstrBr::CondBr(idx_block_true, idx_block_false, cond)) => {
                let cond = get_var(cond)?;
                let block_true = data_extra.block_names
                    .get(&idx_block_true).cloned()
                    .ok_or(WError::DataNotFound("block name"))?;
                let block_false = data_extra.block_names
                    .get(&idx_block_false).cloned()
                    .ok_or(WError::DataNotFound("block name"))?;
                Self::Term(InstrTerm::CondBr(InstrTermCondBr {
                    cond,
                    block_true,
                    block_false
                }))
            },
            FunctionInstr::Br(InstrBr::DirectBr(idx_block)) => {
                let block = data_extra.block_names
                    .get(&idx_block).cloned()
                    .ok_or(WError::DataNotFound("block name"))?;
                Self::Term(InstrTerm::DirectBr(InstrTermDirectBr { block }))
            },
            FunctionInstr::Alloca(instr) => {
                if let Some(val) = vars.remove(&name_res) {
                    vars.insert(name_res.clone(), val);
                }
                Self::Calc(Box::new(InstrCalc::Alloca(InstrCalcAlloca {
                    res: name_res,
                    align: instr.align,
                })))
            },
            FunctionInstr::Call(instr) => {
                let inps = instr.args.iter()
                    .map(|v| {
                        v.get_ty(" convert instr call arg ty")
                            .and_then(|v| Ty::try_from(v.as_ref()))
                    })
                    .collect::<WResult<Vec<_>>>()?;
                let args = instr.args.into_iter()
                    .map(&mut get_var)
                    .collect::<WResult<Vec<_>>>()?;
                let func = InstrFunc::parse(
                    instr.func_name,
                    data_extra.func_name,
                    data_extra.data_idx_var,
                    ids_global
                )?;
                let attr_temp = parse_attrs(
                    &instr.attr_list,
                    u32::MAX,
                    ids_global
                );
                let func_attr = insert_attr(
                    attr_temp,
                    attrs
                );
                let ret = {
                    let ty = Ty::try_from(instr.res_ty.as_ref())?;
                    let attr_temp = parse_attrs(
                        &instr.attr_list,
                        0,
                        ids_global
                    );
                    Arg {
                        ty,
                        attr: attr_temp,
                    }
                };
                let inps = inps
                    .into_iter()
                    .enumerate()
                    .map(|(i, ty)| Arg {
                        ty,
                        attr: parse_attrs(
                            &instr.attr_list,
                            i as u32 + 1,
                            ids_global
                        ),
                    })
                    .collect::<Vec<_>>();
                let tail = ids_global.idx.generate(
                    ID_STATE_ATTR,
                    &instr.call_tail.to_string()
                );
                let conv = ids_global.idx.generate(
                    ID_STATE_ATTR,
                    &instr.call_conv.to_string()
                );

                let res = if name_res.is_empty() {
                    None
                } else {
                    Some(name_res)
                };
                Self::Calc(Box::new(InstrCalc::Call(InstrCalcCall {
                    func,
                    attr: func_attr,
                    ret,
                    args: inps,
                    tail,
                    conv,
                    res,
                    inps: args,
                    fmf: instr.fmf
                })))
            },
            FunctionInstr::Ret(instr) => {
                let var = if let Some(val) = instr.val {
                    let r = get_var(val)?;
                    Some(r)
                } else {
                    None
                };
                Self::Term(InstrTerm::Ret(InstrTermRet {
                    var
                }))
            },
            FunctionInstr::LoadAtomic(instr) => {
                let src = get_var(instr.op)?;
                let ordering = InstrCalcAtomicOrdering::from(instr.ordering);
                let ssid = ids_global.idx.generate(ID_STATE_ATTR, &instr.ssid);
                Self::Calc(Box::new(InstrCalc::LoadAtomic(InstrCalcLoadAtomic {
                    res: name_res,
                    src,
                    ordering,
                    ssid,
                    is_volatile: instr.is_volatile,
                    align: instr.align,
                })))
            },
            FunctionInstr::StoreAtomic(instr) => {
                let data = get_var(instr.val)?;
                let dst = get_var(instr.ptr)?;
                let ordering = InstrCalcAtomicOrdering::from(instr.ordering);
                let ssid = ids_global.idx.generate(ID_STATE_ATTR, &instr.ssid);
                Self::Calc(Box::new(InstrCalc::StoreAtomic(InstrCalcStoreAtomic {
                    data,
                    dst,
                    ordering,
                    ssid,
                    is_volatile: instr.is_volatile,
                    align: instr.align,
                })))
            },
            FunctionInstr::Cmpxchg(instr) => {
                let ptr = get_var(instr.ptr)?;
                let cmp = get_var(instr.cmp)?;
                let val = get_var(instr.val)?;
                let success_ordering = InstrCalcAtomicOrdering::from(instr.success_ordering);
                let failure_ordering = InstrCalcAtomicOrdering::from(instr.failure_ordering);
                let ssid = ids_global.idx.generate(ID_STATE_ATTR, &instr.ssid);
                Self::Calc(Box::new(InstrCalc::Cmpxchg(InstrCalcCmpxchg {
                    res: name_res,
                    ptr,
                    cmp,
                    val,
                    success_ordering,
                    failure_ordering,
                    ssid,
                    is_volatile: instr.is_volatile,
                    is_weak: instr.is_weak,
                    align: instr.align,
                })))
            },
            FunctionInstr::AtomicRMW(instr) => {
                let ptr = get_var(instr.ptr)?;
                let val = get_var(instr.val)?;
                let op = InstrCalcAtomicRMWOp::from(instr.op);
                let ordering = InstrCalcAtomicOrdering::from(instr.ordering);
                let ssid = ids_global.idx.generate(ID_STATE_ATTR, &instr.ssid);
                Self::Calc(Box::new(InstrCalc::AtomicRMW(InstrCalcAtomicRMW {
                    res: name_res,
                    ptr,
                    val,
                    op,
                    ordering,
                    ssid,
                    is_volatile: instr.is_volatile,
                    align: instr.align,
                })))
            },
            FunctionInstr::ExtractVal(instr) => {
                let src = get_var(instr.src)?;
                Self::Calc(Box::new(InstrCalc::ExtractVal(InstrCalcExtractVal {
                    res: name_res,
                    src,
                    idxs: instr.idxs
                })))
            },
            FunctionInstr::Fence(instr) => {
                let ordering = InstrCalcAtomicOrdering::from(instr.ordering);
                let ssid = ids_global.idx.generate(ID_STATE_ATTR, &instr.ssid);
                Self::Calc(Box::new(InstrCalc::Fence(InstrCalcFence {
                    ordering,
                    ssid
                })))
            },
            FunctionInstr::Freeze(instr) => {
                let op = get_var(instr.op)?;
                Self::Calc(Box::new(InstrCalc::Freeze(InstrCalcFreeze {
                    res: name_res,
                    op
                })))
            },
            FunctionInstr::InsertVal(instr) => {
                let src = get_var(instr.src)?;
                let val = get_var(instr.val)?;
                Self::Calc(Box::new(InstrCalc::InsertVal(InstrCalcInsertVal {
                    res: name_res,
                    src,
                    val,
                    idxs: instr.idxs
                })))
            },
            FunctionInstr::LandingPad(instr) => {
                let clauses = instr.clauses.into_iter()
                    .map(| clause | {
                        let ty = InstrCalcLandingPadClauseOp::from(clause.ty);
                        let val = get_var(clause.val)?;
                        Ok(InstrCalcLandingPadClause { ty, val })
                    })
                    .collect::<WResult<Vec<_>>>()?;
                Self::Calc(Box::new(InstrCalc::LandingPad(InstrCalcLandingPad {
                    res: name_res,
                    is_cleanup: instr.is_cleanup,
                    clauses
                })))
            },
            FunctionInstr::UnOp(instr) => {
                let op = InstrCalcUnOpOp::from(instr.op);
                let val = get_var(instr.val)?;
                Self::Calc(Box::new(InstrCalc::UnOp(InstrCalcUnOp {
                    res: name_res,
                    val,
                    op
                })))
            },
            FunctionInstr::Invoke(instr) => {
                let inps = instr.args.iter()
                    .map(|v| {
                        v.get_ty(" convert instr invoke arg ty")
                            .and_then(|v| Ty::try_from(v.as_ref()))
                    })
                    .collect::<WResult<Vec<_>>>()?;
                let args = instr.args.into_iter()
                    .map(&mut get_var)
                    .collect::<WResult<Vec<_>>>()?;
                let func = InstrFunc::parse(
                    instr.func_name,
                    data_extra.func_name,
                    data_extra.data_idx_var,
                    ids_global
                )?;
                let attr_temp = parse_attrs(&instr.attr_list, u32::MAX, ids_global);
                let func_attr = insert_attr(attr_temp, attrs);
                let ret = {
                    let ty = Ty::try_from(instr.res_ty.as_ref())?;
                    let attr_temp = parse_attrs(&instr.attr_list, 0, ids_global);
                    Arg {
                        ty,
                        attr: attr_temp,
                    }
                };
                let inps = inps
                    .into_iter()
                    .enumerate()
                    .map(|(i, ty)| Arg {
                        ty,
                        attr: parse_attrs(&instr.attr_list, i as u32 + 1, ids_global),
                    })
                    .collect::<Vec<_>>();
                let block_normal = data_extra.block_names
                    .get(&instr.block_normal).cloned()
                    .ok_or(WError::DataNotFound("block name"))?;
                let block_unwind = data_extra.block_names
                    .get(&instr.block_unwind).cloned()
                    .ok_or(WError::DataNotFound("block name"))?;
                let conv = ids_global.idx
                    .generate(ID_STATE_ATTR, &instr.call_conv.to_string());

                let res = if name_res.is_empty() {
                    None
                } else {
                    Some(name_res)
                };
                Self::Term(InstrTerm::Invoke(Box::new(InstrTermInvoke {
                    func,
                    attr: func_attr,
                    ret,
                    args: inps,
                    block_normal,
                    block_unwind,
                    conv,
                    res,
                    inps: args,
                })))            },
            FunctionInstr::Resume(instr) => {
                let var = get_var(instr.op)?;
                Self::Term(InstrTerm::Resume(InstrTermResume {
                    var
                }))
            },
            FunctionInstr::Switch(instr) => {
                let cond = get_var(instr.cond)?;
                let block_default = data_extra.block_names
                    .get(&instr.default).cloned()
                    .ok_or(WError::DataNotFound("block name"))?;
                let cases = instr.cases.into_iter()
                    .map(| (idx_block, val) | {
                        let block = data_extra.block_names
                            .get(&idx_block).cloned()
                            .ok_or(WError::DataNotFound("block name"))?;
                        Ok(InstrTermSwitchCase { val, block })
                    })
                    .collect::<WResult<Vec<_>>>()?;
                Self::Term(InstrTerm::Switch(InstrTermSwitch {
                    cond,
                    block_default,
                    cases
                }))
            },
            FunctionInstr::Unreachable(_) => {
                Self::Term(InstrTerm::Unreachable)
            }
            // i => todo!("Convert Instr: {:?}", i),
        };
        // eprintln!("Convert Instr: {:?}", r);
        // if let Self::Calc(Box::new(InstrCalc::Store(v)) = r {
        //     let var_data = vars.get(&v.data);
        //     eprintln!("store: {:?} {:?}", v, var_data);
        // }
        Ok(r)
    }
}


struct MetaDataExtra<'a> {
    func_name: &'a str,
    value_list: &'a GlobalValueList,
    meta_records: &'a HashMap<String, String>,
    data_idx_var: &'a HashMap<(usize, usize), usize>,
}

impl MetaData {
    fn parse(
        meta: MetaDataValue,
        data_extra: &MetaDataExtra,
        ids_global: &mut IDGeneraterNow,
        vars_global: &mut GlobalVars,
    ) -> WResult<Self> {
        let r = match meta {
            MetaDataValue::Named(v) => {
                let id = ids_global.idx.get(
                    ID_STATE_META,
                    &v
                )?;
                Self::Meta(id)
            }
            MetaDataValue::Node(dis, vs) => {
                let vs = vs
                    .into_iter()
                    .map(|v| Self::parse(
                        v,
                        data_extra,
                        ids_global,
                        vars_global,
                    ))
                    .collect::<WResult<Vec<_>>>()?;
                Self::Nodes(dis, vs)
            }
            MetaDataValue::String(v) => Self::String(v),
            MetaDataValue::Value(MetaDataValueSingle::Data(idx_var)) => {
                let r = parse_idx_to_var(
                    idx_var,
                    data_extra.func_name,
                    data_extra.value_list,
                    data_extra.meta_records,
                    ids_global,
                    vars_global,
                    data_extra.data_idx_var
                )?;
                Self::Value(r)
            }
            MetaDataValue::Value(MetaDataValueSingle::Func(v)) => Self::Func(v),
        };
        Ok(r)
    }
}

impl FunctionProto {
    fn parse(
        func: &Function,
        value_list: &GlobalValueList,
        ids_global: &mut IDGeneraterNow,
        attrs: &mut AttrDatas,
    ) -> WResult<Self> {
        // eprintln!("Func proto: {:?}", func.ty);
        let ret = {
            let ty = Ty::try_from(func.ty.retty.as_ref())?;
            let attr_temp = parse_attrs(&func.param_attr, 0, ids_global);
            Arg {
                ty,
                attr: attr_temp,
            }
        };
        let inps = func.ty.paramty.iter()
            .map(|v| Ty::try_from(v.as_ref()))
            .collect::<WResult<Vec<_>>>()?
            .into_iter()
            .enumerate()
            .map(|(i, ty)| Arg {
                ty,
                attr: parse_attrs(&func.param_attr, i as u32 + 1, ids_global),
            })
            .collect::<Vec<_>>();
        let conv = ids_global.idx.generate(
            ID_STATE_ATTR,
            &func.calling_conv.to_string()
        );
        let link_age = ids_global.idx.generate(
            ID_STATE_ATTR,
            &func.link_age.to_string()
        );
        let visibility = ids_global.idx.generate(
            ID_STATE_ATTR,
            &func.visibility.to_string()
        );
        let unnamed_addr = ids_global.idx.generate(
            ID_STATE_ATTR,
            &func.unnamed_addr.to_string()
        );
        let dll_storage_class = ids_global.idx.generate(
            ID_STATE_ATTR,
            &func.dll_storage_class.to_string()
        );

        let attr_temp = parse_attrs(&func.param_attr, u32::MAX, ids_global);
        let attr = insert_attr(attr_temp, attrs);

        let personality = if func.personality_fn == 0 {
            None
        } else {
            let r = value_list
                .get(func.personality_fn-1)
                .and_then(| v | if let GlobalValueUnit::Function(f) = v {
                    Some(ids_global.idx.generate(
                        ID_STATE_FUNC,
                        &f.name
                    ))
                } else { None })
                .ok_or(WError::DataNotFound("FunctionProto personality"))?;
            Some(r)
        };

        Ok(Self {
            ret,
            args: inps,
            conv,
            link_age,
            visibility,
            dll_storage_class,
            unnamed_addr,
            attr,
            personality
        })
    }
}


fn parse_metadata(
    metadata: HashMap<String, MetaDataValue>,
    data_extra: &MetaDataExtra,
    metas: &mut MetaDatas,
    ids_global: &mut IDGeneraterNow,
    vars_global: &mut GlobalVars,
) -> WResult<()> {
    for name in metadata.keys() {
        ids_global.idx.generate(
            ID_STATE_META,
            name
        );
    }
    for (name, meta_temp) in metadata {
        let id = ids_global.idx.get(
            ID_STATE_META,
            &name
        )?;
        // eprintln!("meta: {} {}", name, id);
        let v = MetaData::parse(
            meta_temp,
            data_extra,
            ids_global,
            vars_global
        )?;
        metas.insert(id, v);
    }
    Ok(())
}


impl FunctionNative {
    fn parse(
        func_name: &str,
        func: FunctionBlock,
        base: FunctionProto,
        ids_global: &mut IDGeneraterNow,
        attrs: &mut AttrDatas,
        vars_global: &mut GlobalVars,
        metas: &mut MetaDatas,
    ) -> WResult<Self> {
        let mut data_idx_var = HashMap::with_capacity(func.vars.len());
        for (idx_block, idxs) in func.idxs_instr.into_iter().enumerate() {
            for (idx_instr, idx_var) in idxs {
                data_idx_var.insert((idx_block, idx_instr), idx_var);
            }
        }
        let mut inps = Vec::with_capacity(base.args.len());
        for idx_arg in func.idx_start..(func.idx_start + base.args.len()) {
            let inp_name = ids_global.get_var(
                idx_arg,
                func_name,
                None,
                Some(&func.symtab.value)
            )?;
            inps.push(inp_name);
        }

        let name_empty = ID::empty();
        let mut vars = HashMap::with_capacity(1024);
        let mut name_res_all = HashMap::with_capacity(data_idx_var.len());
        let mut block_names = func.symtab.basic_block.into_iter()
            .map(|(k, v)| (
                k,
                ids_global.idx.generate_with_name(
                    ID_TAG_BLOCK,
                    func_name,
                    v.as_str()
                ),
            ))
            .collect::<HashMap<_, _>>();
        for (idx_block, block) in func.blocks.iter().enumerate() {
            block_names.entry(idx_block).or_insert_with(|| {
                ids_global.get_block(func_name, idx_block)
            });
            for (idx_instr, instr_raw) in block.iter().enumerate() {
                // eprintln!("name res all: {} {} {:?}", idx_block, idx_instr, instr_raw);
                let idx_union = (idx_block, idx_instr);
                let name_res = if let Some(ty) = instr_raw.res() {
                    let val = Ty::try_from(ty.as_ref())?.try_into()?;
                    let idx_var = data_idx_var
                        .get(&idx_union)
                        .cloned()
                        .ok_or(WError::DataNotFound("value instr idx"))?;
                    // eprintln!("instr res: {} {:?}", idx_var, func.symtab.value.get(&idx_var));
                    let name = ids_global.get_var(
                        idx_var,
                        func_name,
                        Some(&val),
                        Some(&func.symtab.value)
                    )?;
                    vars.insert(name.clone(), val);
                    name
                } else {
                    name_empty.clone()
                };
                name_res_all.insert(idx_union, name_res);
            }
        }
        name_res_all.shrink_to_fit();

        let data_extra = MetaDataExtra {  
            func_name,
            value_list: &func.vars,
            meta_records: &func.metadata.record,
            data_idx_var: &data_idx_var,
        };
        parse_metadata(
            func.metadata.named,
            &data_extra,
            metas,
            ids_global,
            vars_global
        )?;
        // drop(func.symtab.value);
        // drop(func.base);
        
        let mut blocks = Vec::with_capacity(func.blocks.len());
        let instr_data_extra = InstrDataExtra {
            func_name,
            value_list: &func.vars,
            data_idx_var: &data_idx_var,
            block_names: &block_names,
            meta_records: &func.metadata.record,
        };
        for (idx_block, block) in func.blocks.into_iter().enumerate() {
            let block_name = block_names.get(&idx_block).cloned()
                .ok_or(WError::DataNotFound("block name"))?;
            let mut instrs_calc = Vec::with_capacity(block.len());
            let mut instrs_phi = Vec::with_capacity(16);
            let mut instr_term = None;
            for (idx_instr, instr_raw) in block.into_iter().enumerate() {
                let idx_union = (idx_block, idx_instr);
                // eprintln!("parse: {} {} {}", func_name, idx_block, idx_instr);
                let name_res = name_res_all
                    .remove(&idx_union)
                    .ok_or(WError::DataNotFound("name res"))?;
                let instr = Instr::parse(
                    instr_raw,
                    name_res,
                    &mut vars,
                    ids_global,
                    attrs,
                    vars_global,
                    &instr_data_extra,
                )?;
                match instr {
                    Instr::Calc(v) => {
                        let metas_temp = func
                            .idxs_instr_global
                            .get(&idx_union)
                            .and_then(|idx_instr| func.metadata_attachment.instr.get(idx_instr));
                        let metas_temp = if let Some(metas_temp) = metas_temp {
                            let mut res = Vec::with_capacity(metas_temp.len());
                            for meta_temp in metas_temp {
                                let name = ids_global.idx.get(
                                    ID_STATE_META,
                                    &meta_temp.name
                                )?;
                                let k = meta_temp.kind.to_string();
                                let kind = ids_global.idx.generate(
                                    ID_STATE_META_KIND,
                                    &k
                                );
                                let t = InstrMeta { name, kind };
                                res.push(t);
                            }
                            res
                        } else {
                            Vec::with_capacity(0)
                        };
                        instrs_calc.push(InstrWithMeta {
                            instr: *v,
                            metas: metas_temp,
                        });
                    }
                    Instr::Phi(v) => {
                        instrs_phi.push(v);
                    }
                    Instr::Term(v) => {
                        if instr_term.is_some() {
                            return Err(WError::DataSetAlready("instr term"));
                        }
                        instr_term.replace(v);
                    }
                }
            }
            instrs_phi.shrink_to_fit();
            instrs_calc.shrink_to_fit();
            let instr_term = instr_term.ok_or(WError::DataNotFound("instr term"))?;

            let b = Block {
                name: block_name,
                instrs_phi,
                instrs_calc,
                instr_term,
            };
            blocks.push(b);
        }
        // drop(func.idxs_instr_global);
        // drop(func.metadata.record);
        // drop(func.metadata_attachment);
        // drop(func.vars);

        blocks.shrink_to_fit();
        inps.shrink_to_fit();
        vars.shrink_to_fit();
        Ok(Self {
            base,
            inps,
            vars,
            blocks,
        })
    }
}

impl TryFrom<BitCode> for LLVM {
    type Error = WError;
    fn try_from(bc: BitCode) -> WResult<Self> {
        let mut ids_global = IDGeneraterNow::default();

        let mut metas = HashMap::with_capacity(128);
        let mut attrs = HashMap::with_capacity(32);
        let mut vars_global = HashMap::with_capacity(bc.module_block.global_var.len());

        let mut funcs_proto = HashMap::with_capacity(16);
        for func in bc.module_block.function.iter() {
            let func = func.as_ref();
            let name = ids_global.idx.generate(ID_STATE_FUNC, &func.name);
            let func = FunctionProto::parse(
                func,
                &bc.module_block.vars,
                &mut ids_global,
                &mut attrs
            )?;
            funcs_proto.insert(name, func);
        }

        let mut funcs_native = HashMap::with_capacity(16);
        for func in bc.module_block.funcs {
            let name = ids_global.idx.get(ID_STATE_FUNC, func.base.name.as_str())?;
            let base = funcs_proto
                .remove(&name)
                .ok_or(WError::DataNotFound("func base"))?;
            ids_global.reset_var_idx();
            let func_name = func.base.name.clone();
            let func_res = FunctionNative::parse(
                &func_name,
                func,
                base,
                &mut ids_global,
                &mut attrs,
                &mut vars_global,
                &mut metas
            )?;
            funcs_native.insert(name, func_res);
        }

        let data_extra = MetaDataExtra {  
            func_name: "",
            value_list: &bc.module_block.vars,
            meta_records: &bc.module_block.metadata.record,
            data_idx_var: &HashMap::with_capacity(0),
        };
        parse_metadata(
            bc.module_block.metadata.named,
            &data_extra,
            &mut metas,
            &mut ids_global,
            &mut vars_global
        )?;

        funcs_native.shrink_to_fit();
        funcs_proto.shrink_to_fit();
        metas.shrink_to_fit();

        let version = ids_global.idx.generate(
            ID_STATE_BASIC,
            &bc.info
        );
        let source_filename = ids_global.idx.generate(
            ID_STATE_BASIC,
            &bc.module_block.basic.source_filename
        );
        let triple = ids_global.idx.generate(
            ID_STATE_BASIC,
            &bc.module_block.basic.triple
        );
        let data_layout = ids_global.idx.generate(
            ID_STATE_BASIC,
            &bc.module_block.basic.data_layout.src
        );

        vars_global.shrink_to_fit();

        let res = Self {
            id: ids_global.idx,
            version,
            source_filename,
            triple,
            data_layout,
            metas,
            attrs,
            vars_global,
            funcs_native,
            funcs_proto,
        };
        Ok(res)
    }
}

impl LLVM {
    pub fn from_file_bc<P: AsRef<Path>>(p: P) -> WResult<Self> {
        let bc = BitCode::from_file(p)?;
        Self::try_from(bc)
    }
}

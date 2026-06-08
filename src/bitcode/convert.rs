//! LLVM from BitCode

use std::{collections::HashMap, fmt::Display, path::Path, rc::Rc, sync::RwLock};

use crate::{
    BitCode, LLVM, WError, WResult,
    bitcode::blocks::{
        constants::{DataSingle, ValueSingle},
        function_block::{
            inst_binop::InstrBinOPType,
            inst_br::InstrBr,
            inst_call::InstrCallTailTy,
            inst_cast::InstrCastType,
            inst_cmp::{InstrCmpFloatType, InstrCmpIntType, InstrCmpType},
            utils::{FunctionBlock, FunctionInstr, GlobalValue},
        },
        function_record::{Function, FunctionCallingConv},
        globalvar::{
            GlobalVarDllStorageClass, GlobalVarLinkAge, GlobalVarThreadLocal, GlobalVarUnnamedAddr,
            GlobalVarVisibility,
        },
        metadata::MetaDataValue,
        moduleblock::{GlobalValueList, GlobalValueUnit},
        param_attr_group::ParamAttrUnit,
        typeblock::TypeBlockData,
        value_symtab::ValueSymTabUnit,
    },
    meta::{
        Arg, AttrDatas, Block, FunctionNative, FunctionProto, GlobalVar, GlobalVars, ID, IDGenerater, InstrCalc, InstrCalcAlloca, InstrCalcBinOP, InstrCalcBinOPOp, InstrCalcCall, InstrCalcCast, InstrCalcCastOp, InstrCalcCmp, InstrCalcCmpFloatOp, InstrCalcCmpIntOp, InstrCalcCmpOp, InstrCalcExtractElt, InstrCalcGEP, InstrCalcInsertElt, InstrCalcLoad, InstrCalcShuffleVec, InstrCalcStore, InstrCalcVSelect, InstrMeta, InstrPhi, InstrTerm, InstrWithMeta, MetaData, Name, Ty, TyBasic, ValBasic, Var, Vars
    }, utils::{ID_STATE_ATTR, ID_STATE_FUNC, ID_STATE_GLOBAL_VAR, ID_STATE_META, ID_STATE_META_KIND, ID_TAG_BLOCK, ID_TAG_VAR},
};

impl ID {
    fn empty() -> Self {
        let name = RwLock::new(String::with_capacity(0));
        let id = 0;
        Self { id, name }
    }

    fn is_empty(&self) -> bool {
        self.id == 0
    }
}

#[derive(Default)]
struct IDGeneraterNow {
    idx: IDGenerater,
    idx_var_unknown: usize,
}

impl IDGeneraterNow {
    fn reset_var_idx(&mut self) {
        self.idx_var_unknown = 0;
    }

    /// idx in value list
    fn get_var(
        &mut self,
        idx_var: usize,
        var: Option<&Var>,
        symtab: Option<&ValueSymTabUnit>,
    ) -> WResult<Name> {
        let name = format!("var_id@@@{idx_var}");
        let r = if let Some(Var::MetaData(Some(name))) = var {
            name.clone()
        } else if let Some(v) = var
            && v.is_const()
        {
            self.idx.generate_const(format!("{:?}", v))
        } else if let Ok(name) = self.idx.get(
            ID_TAG_VAR,
            &name
        ) {
            name
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
            TypeBlockData::Integer(v) if *v <= 65535 => Self::Basic(TyBasic::Int(*v as u16)),
            TypeBlockData::Pointer(v) => {
                let ty = Self::try_from(v.ty.as_ref())?;
                Self::Pointer(Some(Box::new(ty)))
            }
            TypeBlockData::OpaquePointer(_i) => {
                // Self::Pointer(Box::new(ty))
                // todo!()
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
                let tys = v
                    .tys
                    .iter()
                    .map(|v| Self::try_from(v.as_ref()))
                    .collect::<WResult<Vec<_>>>()?;
                Self::Struct(tys)
            }
            TypeBlockData::StructNamed(v) => {
                let tys = v
                    .tys
                    .iter()
                    .map(|v| Self::try_from(v.as_ref()))
                    .collect::<WResult<Vec<_>>>()?;
                Self::Struct(tys)
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
    Calc(InstrCalc),
    Term(InstrTerm),
    Phi(InstrPhi),
}

macro_rules! enum_convert {
    (
        "data_to_var";
        // basic with val
        $($name11:ident = $name12:ident,)*;
        // arr/vec basic with val
        $($name21:ident = $name22:ident = $name23:ident,)*
    ) => {
        fn data_to_var(data: &DataSingle, value_list: &GlobalValueList) -> WResult<Var> {
            let ty = Ty::try_from(data.ty().as_ref())?;
            let r = match (data.value_inner(), ty) {
                $(
                    (ValueSingle::$name11(val), Ty::Basic(ty)) => {
                        Var::Basic(ty, Some(ValBasic::$name12(val)))
                    },
                )*
                $(
                    (ValueSingle::$name21(val), Ty::$name23(ty_single, num)) if
                        let Ty::Basic(ty_basic) = ty_single.as_ref().clone() && (val.len() == num) => {
                        let vs = val
                            .into_iter()
                            .map(| v | Var::Basic(ty_basic.clone(), Some(ValBasic::$name22(v))))
                            .collect();
                        Var::$name23(vs)
                    },
                )*
                (ValueSingle::Null, Ty::Basic(TyBasic::Int(1))) => {
                    Var::Basic(TyBasic::Int(1), Some(ValBasic::Bool(false)))
                },
                (ValueSingle::Null, Ty::Basic(ty)) => {
                    Var::Basic(ty, Some(ValBasic::Null))
                },
                (ValueSingle::Null, Ty::Array(ty_single, num)) if
                    let Ty::Basic(ty_basic) = ty_single.as_ref().clone() => {
                    let vs = (0..num)
                        .map(| _ | Var::Basic(ty_basic.clone(), Some(ValBasic::Null)))
                        .collect();
                    Var::Array(vs)
                },
                (ValueSingle::Null, Ty::Vector(ty_single, num)) if
                    let Ty::Basic(ty_basic) = ty_single.as_ref().clone() => {
                    let vs = (0..num)
                        .map(| _ | Var::Basic(ty_basic.clone(), Some(ValBasic::Null)))
                        .collect();
                    Var::Vector(vs)
                },
                (ValueSingle::Poison, ty) => {
                    Var::Poison(ty)
                },
                (ValueSingle::AggVector(idxs), Ty::Vector(ty_single, num)) if idxs.len() == num => {
                    let ty_single = ty_single.as_ref();
                    let vs = idxs
                        .into_iter()
                        .map(| idx | {
                            let value = value_list
                                .get(idx)
                                .and_then(| v | if let GlobalValueUnit::Data(vv) = v {
                                    Some(vv)
                                } else {
                                    None
                                })
                                .ok_or(WError::DataNotFound("Agg value"))?;
                            let r = data_to_var(value, value_list)?;
                            let ty_temp = r.ty();
                            if &ty_temp != ty_single {
                                return Err(WError::DataNotAllowed(
                                    "agg value ty",
                                    format!("{:?} {:?}", ty_temp, ty_single)
                                ))
                            }
                            Ok::<_, WError>(r)
                        })
                        .collect::<WResult<Vec<_>>>()?;
                    Var::Vector(vs)
                },
                (ValueSingle::CString(val) | ValueSingle::String(val), _) => {
                    Var::Blob(val)
                },
                (data, ty) => return Err(WError::DataNotAllowed(
                    "data_to_var",
                    format!("data: {:?}, ty {:?}", data, ty)
                ))
            };
            Ok(r)
        }
    };


    (
        "type_to_op", $name1:ident, $name2:ident;
        // no related data
        $($var1:ident,)*;
        // one data
        $($var2:ident,)*;
        // two data
        $($var3:ident,)*;
        // no need one data
        $($var4:ident,)*
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
                    $(
                        $name1::$var4(_) => Self::$var4,
                    )*
                }
            }
        }
    };

    ("op_to_str", $name:ident; $($id:ident = $val:expr)*) => {
        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$id => write!(f, $val),
                    )*
                }
            }
        }
    };
}

enum_convert! { "data_to_var";
    Integer1 = Bool, Wide1 = Bool, Integer8 = I8, Wide8 = I8, Integer16 = I16, Wide16 = I16,
    Integer32 = I32, Wide32 = I32, Integer64 = I64, Wide64 = I64, Integer128 = I128,
    Wide128 = I128, BF16 = I16, F16 = I16, F32 = F32, F64 = F64,
    ;
    ArrayWide1 = Bool = Array, VectorWide1 = Bool = Vector,
    ArrayU8 = I8 = Array, ArrayWide8 = I8 = Array, VectorU8 = I8 = Vector, VectorWide8 = I8 = Vector,
    ArrayU16 = I16 = Array, ArrayWide16 = I16 = Array, VectorU16 = I16 = Vector, VectorWide16 = I16 = Vector,
    ArrayU32 = I32 = Array, ArrayWide32 = I32 = Array, VectorU32 = I32 = Vector, VectorWide32 = I32 = Vector,
    ArrayU64 = I64 = Array, ArrayWide64 = I64 = Array, VectorU64 = I64 = Vector, VectorWide64 = I64 = Vector,
    ArrayU128 = I128 = Array, ArrayWide128 = I128 = Array, VectorU128 = I128 = Vector, VectorWide128 = I128 = Vector,
    ArrayF16 = I16 = Array, VectorF16 = I16 = Vector, ArrayBF16 = I16 = Array, VectorBF16 = I16 = Vector,
    ArrayF32 = F32 = Array, VectorF32 = F32 = Vector, ArrayF64 = F64 = Array, VectorF64 = F64 = Vector,

    String = I8 = Array, String = I8 = Vector, CString = I8 = Array, CString = I8 = Vector,
}

enum_convert! { "type_to_op", InstrBinOPType, InstrCalcBinOPOp;
    FAdd, FSub, FMul, Urem, Srem, And, Xor,;
    Sdiv, Udiv, Lshr, Ashr, Or, ;
    Add, Sub, Mul, Shl,;
    Fdiv, Frem,
}

enum_convert! { "type_to_op", InstrCastType, InstrCalcCastOp;
    SExt, FpToUi, FpToSi, SiToFp, PtrToInt, IntToPtr, BitCast, AddrSpaceCast, PtrToAddr, ;
    ZExt, UiToFp, ;
    Trunc, ;
    FpTrunc, FpExt,
}

enum_convert! { "type_to_op", InstrCmpFloatType, InstrCalcCmpFloatOp;
    False, Oeq, Ogt, Oge, Olt, Ole, One, Ord, Uno, Ueq, Ugt, Uge, Ult, Ule, Une, True, ; ; ;
}

enum_convert! { "type_to_op", InstrCmpIntType, InstrCalcCmpIntOp;
    Eq, Ne, Ugt, Uge, Ult, Ule, Sgt, Sge, Slt, Sle, ; ; ;
}

impl From<InstrCmpType> for InstrCalcCmpOp {
    fn from(value: InstrCmpType) -> Self {
        match value {
            InstrCmpType::Float(v, _) => Self::Float(InstrCalcCmpFloatOp::from(v)),
            InstrCmpType::Int(v1, v2) => Self::Int(InstrCalcCmpIntOp::from(v1), v2),
        }
    }
}

fn parse_value_to_val(
    data: GlobalValueUnit,
    value_list: &GlobalValueList,
    meta_records: &HashMap<String, String>,
    ids_global: &mut IDGeneraterNow,
    vars_global: &mut GlobalVars,
    data_idx_var: &HashMap<(usize, usize), usize>,
) -> WResult<(Option<Name>, Option<Var>)> {
    let r = match data {
        GlobalValueUnit::Data(v) => {
            let v = data_to_var(v.as_ref(), value_list)?;
            (None, Some(v))
        }
        GlobalValueUnit::Meta(v) => {
            let k = format!("MetaDataRaw_{v}");
            let name = meta_records
                .get(&k)
                .ok_or(WError::DataNotFound("meta record name"))?;
            let id = ids_global.idx.get("meta", name)?;
            let v = Var::MetaData(Some(id));
            (None, Some(v))
        }
        GlobalValueUnit::Instr(_, idx_block, idx_instr) if idx_block != usize::MAX => {
            let idx_var = data_idx_var
                .get(&(idx_block, idx_instr))
                .cloned()
                .ok_or(WError::DataNotFound("instr idx"))?;
            let name = ids_global.get_var(idx_var, None, None)?;
            // eprintln!("instr res var: {} {} {} {}", idx_block, idx_instr, idx_var, name);
            (Some(name), None)
        }
        GlobalValueUnit::Instr(ty, _, idx_var) | GlobalValueUnit::Future(idx_var, ty) => {
            // func arg, future var
            let val = Ty::try_from(ty.as_ref())?.into();
            let name = ids_global.get_var(idx_var, Some(&val), None)?;
            (Some(name), Some(val))
        }
        GlobalValueUnit::GlobalVar(var) => {
            let var = var.as_ref();
            let name = ids_global.idx.generate(ID_STATE_GLOBAL_VAR, &var.name);
            let data = value_list
                .get(var.init_id - 1)
                .cloned()
                .ok_or(WError::DataNotFound("global var"))?;
            let (_, val) = parse_value_to_val(
                data,
                value_list,
                meta_records,
                ids_global,
                vars_global,
                data_idx_var
            )?;
            if let Some(val) = val && (!vars_global.contains_key(&name)) {
                let link_age = ids_global.idx.generate(
                    ID_STATE_ATTR,
                    &var.link_age.to_string(),
                );
                let visibility = ids_global.idx.generate(
                    ID_STATE_ATTR,
                    &var.visibility.to_string(),
                );
                let unnamed_addr = ids_global.idx.generate(
                    ID_STATE_ATTR,
                    &var.unnamed_addr.to_string(),
                );
                let dll_storage_class = ids_global.idx.generate(
                    ID_STATE_ATTR,
                    &var.dll_storage_class.to_string(),
                );
                let thread_local = ids_global.idx.generate(
                    ID_STATE_ATTR,
                    &var.thread_local.to_string(),
                );
                let var = GlobalVar {
                    is_const: var.is_const,
                    align: var.alignment,
                    link_age,
                    visibility,
                    dll_storage_class,
                    thread_local,
                    unnamed_addr,
                    var: val,
                };
                vars_global.insert(name.clone(), var);
            }
            (Some(name), None)
        }
        _ => todo!("convert value to val: {:?}", data),
    };
    Ok(r)
}

struct InstrDataExtra<'a> {
    value_list: &'a GlobalValueList,
    data_idx_var: &'a HashMap<(usize, usize), usize>,
    block_names: &'a HashMap<usize, Name>,
    meta_records: &'a HashMap<String, String>,
}

enum_convert! { "op_to_str", InstrCallTailTy;
    None = ""
    Tail = "tail"
    Must = "musttail"
    No = "notail"
}

enum_convert! { "op_to_str", FunctionCallingConv;
    Ccc = ""
    Fastcc = "fastcc"
    Coldcc = "coldcc"
    Ghccc = "ghccc"
    WebkitJscc = "webkit_jscc"
    Anyregcc = "anyregcc"
    PreserveMostcc = "preserve_mostcc"
    PreserveAllcc = "preserve_allcc"
    Swiftcc = "swiftcc"
    CxxFastTlscc = "cxx_fast_tlscc"
    Tailcc = "tailcc"
    CfguardCheckcc = "cfguard_checkcc"
    Swifttailcc = "swifttailcc"
    X86Stdcallcc = "x86_stdcallcc"
    X86Fastcallcc = "x86_fastcallcc"
    ArmApcscc = "arm_apcscc"
    ArmAapcscc = "arm_aapcscc"
    ArmAapcsVfpcc = "arm_aapcs_vfpcc"
}

fn parse_attrs(attr: &ParamAttrUnit, idx: u32, ids_gloabl: &mut IDGeneraterNow) -> Vec<Name> {
    attr.get(&idx)
        .map(|v| {
            v.iter()
                .map(|v| v.to_string())
                .filter(|v| !v.is_empty())
                .map(|vv| ids_gloabl.idx.generate(
                    ID_STATE_ATTR,
                    &vv
                ))
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| Vec::with_capacity(0))
}

fn insert_attr(attr_list: Vec<Name>, attrs: &mut AttrDatas) -> Option<u16> {
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

impl Instr {
    fn parse<'a>(
        instr: FunctionInstr,
        name_res: Name,
        vars: &mut Vars,
        ids_global: &mut IDGeneraterNow,
        attrs: &mut AttrDatas,
        vars_global: &mut GlobalVars,
        data_extra: &InstrDataExtra<'a>,
    ) -> WResult<Self> {
        let mut get_var = |data: GlobalValue| {
            // eprintln!("Data: {} {:?}", data.idx, data.data);
            let (name, val) = parse_value_to_val(
                data.data,
                data_extra.value_list,
                data_extra.meta_records,
                ids_global,
                vars_global,
                data_extra.data_idx_var,
            )?;
            // eprintln!("Val raw: {} {:?} {:?}", data.idx, name, val);
            let name = if let Some(name) = name {
                name
            } else {
                ids_global.get_var(data.idx, val.as_ref(), None)?
            };
            // eprintln!("Val: {} {:?}", name, val);
            if let Some(val) = val
                && !vars.contains_key(&name)
            {
                vars.insert(name.clone(), val);
            }
            Ok::<_, WError>(name)
        };

        let r = match instr {
            FunctionInstr::BinOP(instr) => {
                let op = InstrCalcBinOPOp::from(instr.op);
                let left = get_var(instr.left)?;
                let right = get_var(instr.right)?;
                Self::Calc(InstrCalc::BinOp(InstrCalcBinOP {
                    res: name_res,
                    op,
                    left,
                    right,
                }))
            }
            FunctionInstr::Gep(instr) => {
                let base = get_var(instr.base)?;
                let idx = get_var(instr.idx)?;
                if let Some(val) = vars.remove(&name_res) {
                    vars.insert(name_res.clone(), Var::Pointer(Some(Box::new(val))));
                }
                Self::Calc(InstrCalc::Gep(InstrCalcGEP {
                    in_bounds: instr.no_wrap_flags.in_bounds,
                    nusw: instr.no_wrap_flags.nusw,
                    nuw: instr.no_wrap_flags.nuw,
                    res: name_res,
                    base,
                    idx,
                }))
            }
            FunctionInstr::Cast(instr) => {
                let op = InstrCalcCastOp::from(instr.op);
                let val = get_var(instr.val)?;
                Self::Calc(InstrCalc::Cast(InstrCalcCast {
                    res: name_res,
                    op,
                    val,
                }))
            }
            FunctionInstr::Cmp(instr) => {
                let op = InstrCalcCmpOp::from(instr.op);
                let left = get_var(instr.left)?;
                let right = get_var(instr.right)?;
                Self::Calc(InstrCalc::Cmp(InstrCalcCmp {
                    res: name_res,
                    op,
                    left,
                    right,
                }))
            }
            FunctionInstr::ExtractElt(instr) => {
                let src = get_var(instr.src)?;
                let idx = get_var(instr.idx)?;
                Self::Calc(InstrCalc::ExtractElt(InstrCalcExtractElt {
                    res: name_res,
                    src,
                    idx,
                }))
            }
            FunctionInstr::InsertElt(instr) => {
                let src = get_var(instr.src)?;
                let elt = get_var(instr.elt)?;
                let idx = get_var(instr.idx)?;
                Self::Calc(InstrCalc::InsertElt(InstrCalcInsertElt {
                    res: name_res,
                    src,
                    elt,
                    idx,
                }))
            }
            FunctionInstr::ShuffleVec(instr) => {
                let vec1 = get_var(instr.vec1)?;
                let vec2 = get_var(instr.vec2)?;
                let mask = get_var(instr.mask)?;
                Self::Calc(InstrCalc::ShuffleVec(InstrCalcShuffleVec {
                    res: name_res,
                    vec1,
                    vec2,
                    mask,
                }))
            }
            FunctionInstr::VSelect(instr) => {
                let cond = get_var(instr.cond)?;
                let val_true = get_var(instr.val_true)?;
                let val_false = get_var(instr.val_false)?;
                Self::Calc(InstrCalc::VSelect(InstrCalcVSelect {
                    res: name_res,
                    cond,
                    val_true,
                    val_false,
                }))
            }
            FunctionInstr::Phi(instr) => {
                // eprintln!("Phi raw: {:?}", instr);
                let srcs = instr
                    .srcs
                    .into_iter()
                    .map(|(val, idx_block)| {
                        let val = get_var(val)?;
                        let block = data_extra
                            .block_names
                            .get(&idx_block)
                            .cloned()
                            .ok_or(WError::DataNotFound("block name"))?;
                        Ok::<_, WError>((block, val))
                    })
                    .collect::<WResult<Vec<_>>>()?;
                // eprintln!("Phi srcs: {:?}", srcs);
                Self::Phi(InstrPhi {
                    res: name_res,
                    srcs,
                })
            }
            FunctionInstr::Load(instr) => {
                let src = get_var(instr.op)?;
                Self::Calc(InstrCalc::Load(InstrCalcLoad {
                    res: name_res,
                    src,
                    align: instr.align,
                }))
            }
            FunctionInstr::Store(instr) => {
                let data = get_var(instr.val)?;
                let dst = get_var(instr.ptr)?;
                Self::Calc(InstrCalc::Store(InstrCalcStore {
                    data,
                    dst,
                    align: instr.align,
                }))
            }
            FunctionInstr::Br(InstrBr::CondBr(idx_block_true, idx_block_false, cond)) => {
                let block_true = data_extra
                    .block_names
                    .get(&idx_block_true)
                    .cloned()
                    .ok_or(WError::DataNotFound("block name"))?;
                let block_false = data_extra
                    .block_names
                    .get(&idx_block_false)
                    .cloned()
                    .ok_or(WError::DataNotFound("block name"))?;
                let cond = get_var(cond)?;
                Self::Term(InstrTerm::CondBr(cond, block_true, block_false))
            }
            FunctionInstr::Br(InstrBr::DirectBr(idx_block)) => {
                let block = data_extra
                    .block_names
                    .get(&idx_block)
                    .cloned()
                    .ok_or(WError::DataNotFound("block name"))?;
                Self::Term(InstrTerm::DirectBr(block))
            }
            FunctionInstr::Alloca(instr) => {
                if let Some(val) = vars.remove(&name_res) {
                    vars.insert(name_res.clone(), Var::Pointer(Some(Box::new(val))));
                }
                Self::Calc(InstrCalc::Alloca(InstrCalcAlloca {
                    res: name_res,
                    align: instr.align,
                }))
            }
            FunctionInstr::Call(instr) => {
                let inps = instr
                    .args
                    .iter()
                    .map(|v| {
                        v.get_ty(" convert instr call arg ty")
                            .and_then(|v| Ty::try_from(v.as_ref()))
                    })
                    .collect::<WResult<Vec<_>>>()?;
                let args = instr
                    .args
                    .into_iter()
                    .map(&mut get_var)
                    .collect::<WResult<Vec<_>>>()?;

                let func = ids_global.idx
                    .get(
                        ID_STATE_FUNC,
                        instr.func_name.as_str()
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
                let tail = ids_global.idx.generate(
                    ID_STATE_ATTR,
                    &instr.call_tail.to_string(),
                );
                let conv = ids_global.idx.generate(
                    ID_STATE_ATTR,
                    &instr.call_conv.to_string(),
                );

                let res = if name_res.is_empty() {
                    None
                } else {
                    Some(name_res)
                };
                Self::Calc(InstrCalc::Call(InstrCalcCall {
                    func,
                    attr: func_attr,
                    ret,
                    args: inps,
                    tail,
                    conv,
                    res,
                    inps: args,
                }))
            }
            FunctionInstr::Ret(instr) => {
                let val = if let Some(val) = instr.val {
                    let r = get_var(val)?;
                    Some(r)
                } else {
                    None
                };
                Self::Term(InstrTerm::Ret(val))
            }
        };
        // eprintln!("Convert Instr: {:?}", r);
        // if let Self::Calc(InstrCalc::Store(v)) = r {
        //     let var_data = vars.get(&v.data);
        //     eprintln!("store: {:?} {:?}", v, var_data);
        // }
        Ok(r)
    }
}

impl MetaData {
    fn parse(
        meta: MetaDataValue,
        ids_global: &IDGeneraterNow,
        value_list: &GlobalValueList,
    ) -> WResult<Self> {
        let r = match meta {
            MetaDataValue::Named(v) => {
                let id = ids_global.idx.get("meta", &v)?;
                Self::Meta(id)
            }
            MetaDataValue::Node(dis, vs) => {
                let vs = vs
                    .into_iter()
                    .map(|v| Self::parse(v, ids_global, value_list))
                    .collect::<WResult<Vec<_>>>()?;
                Self::Nodes(dis, vs)
            }
            MetaDataValue::String(v) => Self::String(v),
            MetaDataValue::Value(v) => {
                let v = data_to_var(v.as_ref(), value_list)?;
                Self::Value(v)
            }
        };
        Ok(r)
    }
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
    Dllimport = "dllimport"
    Dllexport = "dllexport"
    Common = "common"
    Private = "private"
}

enum_convert! { "op_to_str", GlobalVarVisibility;
    Default = ""
    Hidden = "hidden"
    Protected = "protected"
}

enum_convert! { "op_to_str", GlobalVarUnnamedAddr;
    Not = ""
    Is = "unnamed_addr"
    Local = "local_unnamed_addr"
}

enum_convert! { "op_to_str", GlobalVarThreadLocal;
    NotThreadLocal = ""
    ThreadLocal = "thread_local"
    LocalDynamic = "thread_local(localdynamic)"
    InitialExec = "thread_local(initialexec)"
    LocalExec = "thread_local(locexec)"
}

enum_convert! { "op_to_str", GlobalVarDllStorageClass;
    Default = ""
    DllImport = "dllimport"
    DllExport = "dllexport"
}

impl FunctionProto {
    fn parse(
        func: &Function,
        ids_gloabl: &mut IDGeneraterNow,
        attrs: &mut AttrDatas,
    ) -> WResult<Self> {
        // eprintln!("Func proto: {:?}", func.ty);
        let ret = {
            let ty = Ty::try_from(func.ty.retty.as_ref())?;
            let attr_temp = parse_attrs(&func.param_attr, 0, ids_gloabl);
            Arg {
                ty,
                attr: attr_temp,
            }
        };
        let inps = func
            .ty
            .paramty
            .iter()
            .map(|v| Ty::try_from(v.as_ref()))
            .collect::<WResult<Vec<_>>>()?
            .into_iter()
            .enumerate()
            .map(|(i, ty)| Arg {
                ty,
                attr: parse_attrs(&func.param_attr, i as u32 + 1, ids_gloabl),
            })
            .collect::<Vec<_>>();
        let link_age = ids_gloabl.idx.generate(
            ID_STATE_ATTR,
            &func.link_age.to_string(),
        );
        let visibility = ids_gloabl.idx.generate(
            ID_STATE_ATTR,
            &func.visibility.to_string(),
        );
        let unnamed_addr = ids_gloabl.idx.generate(
            ID_STATE_ATTR,
            &func.unnamed_addr.to_string(),
        );
        let dll_storage_class = ids_gloabl.idx.generate(
            ID_STATE_ATTR,
            &func.dll_storage_class.to_string(),
        );

        let attr_temp = parse_attrs(&func.param_attr, u32::MAX, ids_gloabl);
        let attr = insert_attr(attr_temp, attrs);

        Ok(Self {
            ret,
            args: inps,
            link_age,
            visibility,
            dll_storage_class,
            unnamed_addr,
            attr,
        })
    }
}

impl FunctionNative {
    fn parse(
        func_name: &str,
        func: FunctionBlock,
        base: FunctionProto,
        ids_gloabl: &mut IDGeneraterNow,
        attrs: &mut AttrDatas,
        vars_global: &mut GlobalVars,
        meta_records: &HashMap<String, String>,
    ) -> WResult<Self> {
        let mut inps = Vec::with_capacity(8);
        let mut blocks = Vec::with_capacity(func.blocks.len());
        let name_empty = Rc::new(ID::empty());
        let mut vars = HashMap::with_capacity(1024);

        let mut data_idx_var = HashMap::with_capacity(func.vars.len());
        for (idx_block, idxs) in func.idxs_instr.into_iter().enumerate() {
            for (idx_instr, idx_var) in idxs {
                data_idx_var.insert((idx_block, idx_instr), idx_var);
            }
        }
        for idx_arg in func.idx_start..(func.idx_start + func.base.ty.paramty.len()) {
            let inp_name = ids_gloabl.get_var(idx_arg, None, Some(&func.symtab.value))?;
            inps.push(inp_name);
        }
        let mut name_res_all = HashMap::with_capacity(data_idx_var.len());
        for (idx_block, block) in func.blocks.iter().enumerate() {
            for (idx_instr, instr_raw) in block.iter().enumerate() {
                let idx_union = (idx_block, idx_instr);
                let name_res = if let Some(ty) = instr_raw.res() {
                    let val = Ty::try_from(ty.as_ref())?.into();
                    let idx_var = data_idx_var
                        .get(&idx_union)
                        .cloned()
                        .ok_or(WError::DataNotFound("value instr idx"))?;
                    // eprintln!("instr res: {} {:?}", idx_var, func.symtab.value.get(&idx_var));
                    let name = ids_gloabl.get_var(idx_var, Some(&val), Some(&func.symtab.value))?;
                    vars.insert(name.clone(), val);
                    name
                } else {
                    name_empty.clone()
                };
                name_res_all.insert(idx_union, name_res);
            }
        }
        name_res_all.shrink_to_fit();

        // for arg in func.symtab.value
        let block_names = func
            .symtab
            .basic_block
            .into_iter()
            .map(|(k, v)| (
                k,
                ids_gloabl.idx.generate_with_name(
                    ID_TAG_BLOCK,
                    func_name,
                    v.as_str()
                )
            ))
            .collect::<HashMap<_, _>>();
        let instr_data_extra = InstrDataExtra {
            value_list: &func.vars,
            data_idx_var: &data_idx_var,
            block_names: &block_names,
            meta_records,
        };
        for (idx_block, block) in func.blocks.into_iter().enumerate() {
            let block_name = block_names
                .get(&idx_block)
                .cloned()
                .ok_or(WError::DataNotFound("basic block name"))?;
            let mut instrs_calc = Vec::with_capacity(block.len());
            let mut instrs_phi = Vec::with_capacity(16);
            let mut instr_term = None;
            for (idx_instr, instr_raw) in block.into_iter().enumerate() {
                // eprintln!("instr convert: {} {} {:?}", idx_block, idx_instr, instr_raw);
                let idx_union = (idx_block, idx_instr);
                let name_res = name_res_all
                    .remove(&idx_union)
                    .ok_or(WError::DataNotFound("name res"))?;
                let instr = Instr::parse(
                    instr_raw,
                    name_res,
                    &mut vars,
                    ids_gloabl,
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
                                let name = ids_gloabl.idx.get(
                                    ID_STATE_META,
                                    &meta_temp.name
                                )?;
                                let k = meta_temp.kind.to_string();
                                let kind = ids_gloabl.idx.generate(
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
                            instr: v,
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

        // 解析meta
        for name in bc.module_block.metadata.named.keys() {
            ids_global.idx.generate(ID_STATE_META, name);
        }
        for (name, meta_temp) in bc.module_block.metadata.named {
            let id = ids_global.idx.generate(ID_STATE_META, &name);
            // eprintln!("parse meta: {} {}", name, id);
            let v = MetaData::parse(meta_temp, &ids_global, &bc.module_block.vars)?;
            metas.insert(id, v);
        }

        let mut funcs_proto = HashMap::with_capacity(16);
        for func in bc.module_block.function.iter() {
            let func = func.as_ref();
            let name = ids_global.idx.generate(ID_STATE_FUNC, &func.name);
            let func = FunctionProto::parse(func, &mut ids_global, &mut attrs)?;
            funcs_proto.insert(name, func);
        }

        let mut funcs_native = HashMap::with_capacity(16);
        for func in bc.module_block.funcs {
            let name = ids_global.idx.get(
                ID_STATE_FUNC,
                func.base.name.as_str()
            )?;
            // eprintln!("Function: {}", name);
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
                &bc.module_block.metadata.record,
            )?;
            funcs_native.insert(name, func_res);
        }
        funcs_native.shrink_to_fit();
        funcs_proto.shrink_to_fit();
        metas.shrink_to_fit();

        let version = ids_global.idx.generate(
            "version",
            &bc.info,
        );
        let source_filename = ids_global.idx.generate(
            "source_filename",
            &bc.module_block.basic.source_filename,
        );
        let triple = ids_global.idx.generate(
            "triple",
            &bc.module_block.basic.triple,
        );
        let data_layout = ids_global.idx.generate(
            "data_layout",
            &bc.module_block.basic.data_layout.src,
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

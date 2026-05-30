//! LLVM from BitCode file

use std::{collections::HashMap, path::Path, rc::Rc};

use crate::{
    BitCode, LLVM, WError, WResult,
    bitcode::blocks::{
        constants::{DataSingle, ValueSingle}, function_block::{inst_binop::InstrBinOPType, inst_br::InstrBr, inst_cast::InstrCastType, inst_cmp::{InstrCmpFloatType, InstrCmpIntType, InstrCmpType}, utils::{FunctionBlock, FunctionInstr, GlobalValue}}, function_record::Function, moduleblock::{GlobalValueList, GlobalValueUnit}, typeblock::TypeBlockData
    },
    meta::{Block, FunctionNative, FunctionProto, InstrCalc, InstrCalcAlloca, InstrCalcBinOP, InstrCalcBinOPOp, InstrCalcCall, InstrCalcCast, InstrCalcCastOp, InstrCalcCmp, InstrCalcCmpFloatOp, InstrCalcCmpIntOp, InstrCalcCmpOp, InstrCalcExtractElt, InstrCalcGEP, InstrCalcInsertElt, InstrCalcLoad, InstrCalcShuffleVec, InstrCalcStore, InstrCalcVSelect, InstrPhi, InstrTerm, Name, Ty, TyBasic, ValBasic, Var, Vars},
};

impl TryFrom<&TypeBlockData> for Ty {
    type Error = WError;
    fn try_from(data: &TypeBlockData) -> WResult<Self> {
        let r = match data {
            TypeBlockData::Void => Self::Basic(Rc::new(TyBasic::Void)),
            TypeBlockData::Float => Self::Basic(Rc::new(TyBasic::Float)),
            TypeBlockData::Double => Self::Basic(Rc::new(TyBasic::Double)),
            TypeBlockData::Label => Self::Basic(Rc::new(TyBasic::Label)),
            TypeBlockData::Half => Self::Basic(Rc::new(TyBasic::Half)),
            TypeBlockData::MetaData => Self::MetaData,
            TypeBlockData::Integer(v) if *v <= 65535 => {
                Self::Basic(Rc::new(TyBasic::Int(*v as u16)))
            },
            TypeBlockData::Pointer(v) => {
                let ty = Self::try_from(v.ty.as_ref())?;
                Self::Pointer(Rc::new(ty))
            },
            TypeBlockData::OpaquePointer(_i) => {
                // Self::Pointer(Rc::new(ty))
                // todo!()
                Self::Basic(Rc::new(TyBasic::Label))
            },
            TypeBlockData::Array(v) => {
                let ty = Self::try_from(v.ty.as_ref())?;
                let num = v.num;
                Self::Array(Rc::new(ty), num)
            },
            TypeBlockData::Vector(v) => {
                let ty = Self::try_from(v.ty.as_ref())?;
                let num = v.num;
                Self::Vector(Rc::new(ty), num)
            },
            TypeBlockData::StructAnon(v) => {
                let tys = v
                    .tys
                    .iter()
                    .map(|v| Self::try_from(v.as_ref()))
                    .collect::<WResult<Vec<_>>>()?;
                Self::Struct(tys)
            },
            TypeBlockData::StructNamed(v) => {
                let tys = v
                    .tys
                    .iter()
                    .map(|v| Self::try_from(v.as_ref()))
                    .collect::<WResult<Vec<_>>>()?;
                Self::Struct(tys)
            },
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


fn convert_name(data: &str, prefix: &'static str) -> Name {
    let mut data = format!("{}{}", prefix, data);
    data.shrink_to_fit();
    Rc::new(data)
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
                (ValueSingle::Null, Ty::Basic(ty)) if let TyBasic::Int(1) = ty.as_ref() => {
                    Var::Basic(ty, Some(ValBasic::Bool(false)))
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
                    Var::Poison(Rc::new(ty))
                },
                (ValueSingle::CString(val) | ValueSingle::String(val), Ty::Basic(ty)) => {
                    Var::Basic(ty, Some(ValBasic::Blob(Rc::new(val))))
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
}

enum_convert!{ "data_to_var";
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

enum_convert!{ "type_to_op", InstrBinOPType, InstrCalcBinOPOp;
    FAdd, FSub, FMul, Urem, Srem, And, Xor,;
    ;
    Add, Sub, Mul, Shl,;
    Sdiv, Udiv, Lshr, Ashr, Or, Fdiv, Frem,
}

enum_convert!{ "type_to_op", InstrCastType, InstrCalcCastOp;
    SExt, FpToUi, FpToSi, SiToFp, PtrToInt, IntToPtr, BitCast, AddrSpaceCast, PtrToAddr, ;
    ZExt, UiToFp, ;
    Trunc, ;
    FpTrunc, FpExt,
}

enum_convert!{ "type_to_op", InstrCmpFloatType, InstrCalcCmpFloatOp;
    False, Oeq, Ogt, Oge, Olt, Ole, One, Ord, Uno, Ueq, Ugt, Uge, Ult, Ule, Une, True, ; ; ;
}

enum_convert!{ "type_to_op", InstrCmpIntType, InstrCalcCmpIntOp;
    Eq, Ne, Ugt, Uge, Ult, Ule, Sgt, Sge, Slt, Sle, ; ; ;
}

impl From<InstrCmpType> for InstrCalcCmpOp {
    fn from(value: InstrCmpType) -> Self {
        match value {
            InstrCmpType::Float(v, _) => Self::Float(InstrCalcCmpFloatOp::from(v)),
            InstrCmpType::Int(v1, v2) => Self::Int(InstrCalcCmpIntOp::from(v1), v2)
        }
    }
}



fn parse_value_to_val<F: FnMut(usize, Option<&Var>)->Name>(
    data: GlobalValueUnit,
    value_list: &GlobalValueList,
    data_idx_var: &HashMap<(usize, usize), usize>,
    mut get_name: F,
) -> WResult<(Option<Name>, Option<Var>)> {
    let r = match data {
        GlobalValueUnit::Data(v) => {
            let v = data_to_var(
                v.as_ref(),
                value_list
            )?;
            (None, Some(v))
        },
        GlobalValueUnit::Meta(v) => {
            let v = Var::MetaData(Some(v));
            (None, Some(v))
        },
        GlobalValueUnit::Instr(_, idx_block, idx_instr) if idx_block != usize::MAX => {
            let idx_var = data_idx_var
                .get(&(idx_block, idx_instr))
                .cloned()
                .ok_or(WError::DataNotFound("instr idx"))?;
            let name = get_name(idx_var, None);
            // eprintln!("instr res var: {} {} {} {}", idx_block, idx_instr, idx_var, name);
            (Some(name), None)
        },
        GlobalValueUnit::Instr(ty, _, idx_var) | GlobalValueUnit::Future(idx_var, ty) => {
            // func arg, future var
            let val = Ty::try_from(ty.as_ref())?.into();
            let name = get_name(idx_var, None);
            (Some(name), Some(val))
        },
        GlobalValueUnit::GlobalVar(var) => {
            let var = var.as_ref();
            let name = convert_name(var.name.as_str(), "GlobalVar_");
            let data = value_list
                .get(var.init_id - 1)
                .cloned()
                .ok_or(WError::DataNotFound("global var"))?;
            let (_, val) = parse_value_to_val(
                data,
                value_list,
                data_idx_var,
                get_name
            )?;
            (Some(name), val)
        },
        _ => todo!("convert value to val: {:?}", data)
    };
    Ok(r)
}


impl Instr {
    fn parse<F: FnMut(usize, Option<&Var>)->Name>(
        instr: FunctionInstr,
        name_res: Name,
        mut get_name: F,
        vars: &mut Vars,
        value_list: &GlobalValueList,
        data_idx_var: &HashMap<(usize, usize), usize>,
        block_names: &HashMap<usize, Name>
    ) -> WResult<Self> {
        let mut get_var = | data: GlobalValue | {
            // eprintln!("Data: {} {:?}", data.idx, data.data);
            let (name, val) = parse_value_to_val(
                data.data,
                value_list,
                data_idx_var,
                &mut get_name
            )?;
            // eprintln!("Val raw: {} {:?} {:?}", data.idx, name, val);
            let name = name.unwrap_or_else(|| get_name(
                data.idx,
                val.as_ref()
            ));
            // eprintln!("Val: {} {:?}", name, val);
            if let Some(val) = val && !vars.contains_key(&name) {
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
                    right
                }))
            },
            FunctionInstr::Gep(instr) => {
                let base = get_var(instr.base)?;
                let idx = get_var(instr.idx)?;
                Self::Calc(InstrCalc::Gep(InstrCalcGEP {
                    res: name_res,
                    base,
                    idx
                }))
            },
            FunctionInstr::Cast(instr) => {
                let op = InstrCalcCastOp::from(instr.op);
                let val = get_var(instr.val)?;
                Self::Calc(InstrCalc::Cast(InstrCalcCast {
                    res: name_res,
                    op,
                    val
                }))
            },
            FunctionInstr::Cmp(instr) => {
                let op = InstrCalcCmpOp::from(instr.op);
                let left = get_var(instr.left)?;
                let right = get_var(instr.right)?;
                Self::Calc(InstrCalc::Cmp(InstrCalcCmp {
                    res: name_res,
                    op,
                    left,
                    right
                }))
            },
            FunctionInstr::ExtractElt(instr) => {
                let src = get_var(instr.src)?;
                let idx = get_var(instr.idx)?;
                Self::Calc(InstrCalc::ExtractElt(InstrCalcExtractElt {
                    res: name_res,
                    src,
                    idx
                }))
            },
            FunctionInstr::InsertElt(instr) => {
                let src = get_var(instr.src)?;
                let elt = get_var(instr.elt)?;
                let idx = get_var(instr.idx)?;
                Self::Calc(InstrCalc::InsertElt(InstrCalcInsertElt {
                    res: name_res,
                    src,
                    elt,
                    idx
                }))
            },
            FunctionInstr::ShuffleVec(instr) => {
                let vec1 = get_var(instr.vec1)?;
                let vec2 = get_var(instr.vec2)?;
                let mask = get_var(instr.mask)?;
                Self::Calc(InstrCalc::ShuffleVec(InstrCalcShuffleVec {
                    res: name_res,
                    vec1,
                    vec2,
                    mask
                }))
            },
            FunctionInstr::VSelect(instr) => {
                let cond = get_var(instr.cond)?;
                let val_true = get_var(instr.val_true)?;
                let val_false = get_var(instr.val_false)?;
                Self::Calc(InstrCalc::VSelect(InstrCalcVSelect {
                    res: name_res,
                    cond,
                    val_true,
                    val_false
                }))
            },
            FunctionInstr::Phi(instr) => {
                let srcs = instr.srcs
                    .into_iter()
                    .map(| (val, idx_block) | {
                        let val = get_var(val)?;
                        let block = block_names
                            .get(&idx_block)
                            .cloned()
                            .ok_or(WError::DataNotFound("block name"))?;
                        Ok::<_, WError>((block, val))
                    })
                    .collect::<WResult<Vec<_>>>()?;
                Self::Phi(InstrPhi {
                    res: name_res,
                    srcs
                })
            },
            FunctionInstr::Load(instr) => {
                let src = get_var(instr.op)?;
                Self::Calc(InstrCalc::Load(InstrCalcLoad {
                    res: name_res,
                    src
                }))
            },
            FunctionInstr::Store(instr) => {
                let data = get_var(instr.val)?;
                let dst = get_var(instr.ptr)?;
                Self::Calc(InstrCalc::Store(InstrCalcStore {
                    data,
                    dst 
                }))
            },
            FunctionInstr::Br(InstrBr::CondBr(idx_block_true, idx_block_false, cond)) => {
                let block_true = block_names
                    .get(&idx_block_true)
                    .cloned()
                    .ok_or(WError::DataNotFound("block name"))?;
                let block_false = block_names
                    .get(&idx_block_false)
                    .cloned()
                    .ok_or(WError::DataNotFound("block name"))?;
                let cond = get_var(cond)?;
                Self::Term(InstrTerm::CondBr(cond, block_true, block_false))
            },
            FunctionInstr::Br(InstrBr::DirectBr(idx_block)) => {
                let block = block_names
                    .get(&idx_block)
                    .cloned()
                    .ok_or(WError::DataNotFound("block name"))?;
                Self::Term(InstrTerm::DirectBr(block))
            },
            FunctionInstr::Alloca(instr) => {
                let val = Ty::try_from(instr.ty_inst.as_ref())?.into();
                vars.insert(name_res.clone(), val);
                Self::Calc(InstrCalc::Alloca(InstrCalcAlloca {
                    res: name_res
                }))
            },
            FunctionInstr::Call(instr) => {
                let func = convert_name(instr.func_name.as_str(), "Func_");
                let args = instr.args
                    .into_iter()
                    .map(&mut get_var)
                    .collect::<WResult<Vec<_>>>()?;
                let res = if name_res.is_empty() {
                    None
                } else {
                    Some(name_res)
                };
                Self::Calc(InstrCalc::Call(InstrCalcCall {
                    res,
                    func,
                    args
                }))
            },
            FunctionInstr::Ret(instr) => {
                let val = if let Some(val) = instr.val {
                    let r = get_var(val)?;
                    Some(r)
                } else {
                    None
                };
                Self::Term(InstrTerm::Ret(val))
            },
        };
        // eprintln!("Convert Instr: {:?}", r);
        // if let Self::Calc(InstrCalc::Store(v)) = r {
        //     let var_data = vars.get(&v.data);
        //     eprintln!("store: {:?} {:?}", v, var_data);
        // }
        Ok(r)
    }
}


impl TryFrom<&Function> for FunctionProto {
    type Error = WError;
    fn try_from(func: &Function) -> WResult<Self> {
        // eprintln!("Func proto: {:?}", func.ty);
        let ret = Ty::try_from(func.ty.retty.as_ref())?;
        let inps = func.ty.paramty
            .iter()
            .map(| v | Ty::try_from(v.as_ref()).map(| v | v.into()))
            .collect::<WResult<Vec<_>>>()?;
        Ok(Self { ret, inps })
    }
}

impl TryFrom<FunctionBlock> for FunctionNative {
    type Error = WError;
    fn try_from(func: FunctionBlock) -> WResult<Self> {
        let base = FunctionProto::try_from(func.base.as_ref())?;
        let mut inps = Vec::with_capacity(8);
        let mut blocks = Vec::with_capacity(func.blocks.len());
        let mut idx_var_unknown = 0;
        let mut idx_var_const = 0;
        let mut idx_var_metadata = 0;
        let name_empty = Rc::new(String::with_capacity(0));
        let mut vars = HashMap::with_capacity(1024);
        let mut vars_already: HashMap<usize, Rc<String>> = HashMap::with_capacity(func.vars.len());
        let mut get_name = | idx_var, var: Option<&Var> | {
            let r = if let Some(Var::MetaData(_)) = var {
                let r = format!("MetaData_{idx_var_metadata}");
                idx_var_metadata += 1;
                Rc::new(r)
            } else if let Some(v) = var && v.is_const() {
                // eprintln!("Const id: {}", idx_var_const);
                let r = format!("Const_{idx_var_const}");
                idx_var_const += 1;
                Rc::new(r)
            } else if let Some(r) = vars_already.get(&idx_var) {
                return r.clone();
            } else {
                func.symtab.value
                    .get(&idx_var)
                    .map(| v | convert_name(v, "Var_"))
                    .unwrap_or_else(|| {
                        let r = format!("Var_{idx_var_unknown}");
                        idx_var_unknown += 1;
                        Rc::new(r)
                    })
            };
            vars_already.insert(idx_var, r.clone());
            r
        };
        let mut data_idx_var = HashMap::with_capacity(func.vars.len());
        for (idx_block, idxs) in func.idxs_instr.into_iter().enumerate() {
            for (idx_instr, idx_var) in idxs {
                data_idx_var.insert((idx_block, idx_instr), idx_var);
            }
        }
        // for arg in func.symtab.value
        let block_names = func.symtab.basic_block
            .into_iter()
            .map(| (k, v) | {
                (
                    k,
                    convert_name(v.as_str(), "Block_")
                )
            })
            .collect::<HashMap<_, _>>();
        for (idx_block, block) in func.blocks.into_iter().enumerate() {
            let block_name = block_names
                .get(&idx_block)
                .cloned()
                .ok_or(WError::DataNotFound("basic block name"))?;
            let mut instrs_calc = Vec::with_capacity(block.len());
            let mut instrs_phi = Vec::with_capacity(16);
            let mut instr_term = None;
            for (idx_instr, instr_raw) in block.into_iter().enumerate() {
                let name_res = if let Some(ty) = instr_raw.res() {
                    let idx_var = data_idx_var
                        .get(&(idx_block, idx_instr))
                        .cloned()
                        .ok_or(WError::DataNotFound("value instr idx"))?;
                    // eprintln!("instr res: {} {:?}", idx_var, func.symtab.value.get(&idx_var));
                    let name = get_name(idx_var, None);
                    let val = Ty::try_from(ty.as_ref())?.into();
                    vars.insert(name.clone(), val);
                    name
                } else {
                    name_empty.clone()
                };
                // eprintln!("instr posi: {} {}", idx_block, idx_instr);
                let instr = Instr::parse(
                    instr_raw,
                    name_res,
                    &mut get_name,
                    &mut vars,
                    &func.vars,
                    &data_idx_var,
                    &block_names
                )?;
                match instr {
                    Instr::Calc(v) => {
                        instrs_calc.push(v);
                    },
                    Instr::Phi(v) => {
                        instrs_phi.push(v);
                    },
                    Instr::Term(v) => {
                        if instr_term.is_some() {
                            return Err(WError::DataSetAlready("instr term"));
                        }
                        instr_term.replace(v);
                    },
                }
            }
            instrs_phi.shrink_to_fit();
            instrs_calc.shrink_to_fit();
            let instr_term = instr_term.ok_or(WError::DataNotFound("instr term"))?;

            let b = Block {
                name: block_name,
                instrs_phi,
                instrs_calc,
                instr_term
            };
            blocks.push(b);
        }
        
        for idx_arg in (func.idx_start..(func.idx_start + func.base.ty.paramty.len())) {
            let inp_name = get_name(idx_arg, None);
            inps.push(inp_name);
        }
        
        blocks.shrink_to_fit();
        inps.shrink_to_fit();
        vars.shrink_to_fit();
        Ok(Self { base, inps, vars, blocks })
    }
}

impl TryFrom<BitCode> for LLVM {
    type Error = WError;
    fn try_from(bc: BitCode) -> WResult<Self> {
        let mut funcs_native = HashMap::with_capacity(16);
        let mut funcs_proto = HashMap::with_capacity(16);

        for func in bc.module_block.function {
            if !func.is_proto {
                continue;
            }
            let func = func.as_ref();
            let name = convert_name(func.name.as_str(), "Func_");
            let func = FunctionProto::try_from(func)?;
            funcs_proto.insert(name, func);
        }
        for func in bc.module_block.funcs {
            let name = convert_name(func.base.name.as_str(), "Func_");
            // eprintln!("Function: {}", name);
            let func = FunctionNative::try_from(func)?;
            funcs_native.insert(name, func);
        }
        
        funcs_native.shrink_to_fit();
        funcs_proto.shrink_to_fit();
        let res = Self {
            version: Rc::new(bc.info),
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

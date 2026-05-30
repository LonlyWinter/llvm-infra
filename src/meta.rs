//! LLVM Meta

use std::{collections::HashMap, rc::Rc};

pub type Name = Rc<String>;
pub type Blob = Rc<Vec<u8>>;

/// 基本类型
#[derive(Debug, Clone, PartialEq)]
pub enum TyBasic {
    Label,
    Blob,
    Void,
    Float,
    Double,
    Half,
    Int(u16),
}

/// 基本数据
#[derive(Debug)]
pub enum ValBasic {
    Null,
    Poison,
    Label(Name),
    Blob(Blob),
    Bool(bool),
    I8(u8),
    I16(u16),
    I32(u32),
    I64(u64),
    I128(u128),
    F32(f32),
    F64(f64),
}

/// 所有类型
#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Basic(Rc<TyBasic>),
    Pointer(Rc<Self>),
    Array(Rc<Self>, usize),
    Vector(Rc<Self>, usize),
    Struct(Vec<Self>),
    MetaData,
}

/// 所有变量：内含类型和数据
#[derive(Debug)]
pub enum Var {
    Basic(Rc<TyBasic>, Option<ValBasic>),
    Pointer(Rc<Self>),
    Array(Vec<Self>),
    Vector(Vec<Self>),
    Struct(Vec<Self>),
    MetaData(Option<usize>),
    Poison(Rc<Ty>),
}

impl From<Ty> for Var {
    fn from(ty: Ty) -> Self {
        match ty {
            Ty::Basic(v) => Self::Basic(v, None),
            Ty::Pointer(v) => Self::Pointer(Rc::new(Var::from(v.as_ref().clone()))),
            Ty::Array(v, n) => Self::Array((0..n).map(| _ | Var::from(v.as_ref().clone())).collect()),
            Ty::Vector(v, n) => Self::Vector((0..n).map(| _ | Var::from(v.as_ref().clone())).collect()),
            Ty::Struct(vs) => Self::Vector(vs.into_iter().map(Var::from).collect()),
            Ty::MetaData => Self::MetaData(None),
        }
    }
}

impl Var {
    pub fn ty(&self) -> Ty {
        match self {
            Self::Basic(v, _) => Ty::Basic(v.clone()),
            Self::Pointer(v) => v.ty(),
            Self::Array(vs) | Self::Vector(vs) => {
                let ty = vs[0].ty();
                let n = vs.len();
                Ty::Array(Rc::new(ty), n)
            }
            Self::Struct(vs) => {
                let tys = vs.iter().map(|v| v.ty()).collect::<Vec<_>>();
                Ty::Struct(tys)
            },
            Self::MetaData(_) => Ty::MetaData,
            Self::Poison(v) => v.as_ref().clone(),
        }
    }

    pub fn is_const(&self) -> bool {
        match self {
            Self::Basic(_, v) => v.is_some(),
            Self::Pointer(v) => v.is_const(),
            Self::Array(vs) | Self::Vector(vs) | Self::Struct(vs) => vs.iter()
                .any(| v | v.is_const()),
            Self::MetaData(_) | Self::Poison(_) => true,
        }
    }
}

pub type Vars = HashMap<Name, Var>;

#[derive(Debug)]
pub struct Arg {
    pub ty: Ty,
}

impl From<Ty> for Arg {
    fn from(ty: Ty) -> Self {
        Self { ty }
    }
}

/// 变量输入指令
#[derive(Debug)]
pub struct InstrPhi {
    /// res var name
    pub res: Name,
    /// 来源：block_name, var_name
    pub srcs: Vec<(Name, Name)>,
}

#[derive(Debug)]
pub struct InstrCalcAlloca {
    /// res var name
    pub res: Name,
}

#[derive(Debug)]
pub enum InstrCalcBinOPOp {
    FAdd,
    FSub,
    FMul,
    Urem,
    Srem,
    And,
    Xor,
    /// no signed wrap, no unsigned wrap
    Add(bool, bool),
    /// no signed wrap, no unsigned wrap
    Sub(bool, bool),
    /// no signed wrap, no unsigned wrap
    Mul(bool, bool),
    /// no signed wrap, no unsigned wrap
    Shl(bool, bool),
    Sdiv,
    Udiv,
    Lshr,
    Ashr,
    Or,
    Fdiv,
    Frem,
}

#[derive(Debug)]
pub struct InstrCalcBinOP {
    /// res var name
    pub res: Name,
    /// op
    pub op: InstrCalcBinOPOp,
    /// left var name
    pub left: Name,
    /// right var name
    pub right: Name,
}

#[derive(Debug)]
pub struct InstrCalcCall {
    /// res var name
    pub res: Option<Name>,
    /// ty
    pub func: Name,
    /// args var name
    pub args: Vec<Name>,
}

#[derive(Debug)]
pub enum InstrCalcCastOp {
    SExt,
    FpToUi,
    FpToSi,
    SiToFp,
    PtrToInt,
    IntToPtr,
    BitCast,
    AddrSpaceCast,
    PtrToAddr,
    /// non neg
    ZExt(bool),
    /// non neg
    UiToFp(bool),
    /// no signed wrap, no unsigned wrap
    Trunc(bool, bool),
    FpTrunc,
    FpExt,
}

#[derive(Debug)]
pub struct InstrCalcCast {
    /// res var name
    pub res: Name,
    /// op
    pub op: InstrCalcCastOp,
    /// inp var name
    pub val: Name,
}

#[derive(Debug)]
pub enum InstrCalcCmpFloatOp {
    // Always false (always folded)
    False,
    // True if ordered and equal
    Oeq,
    // True if ordered and greater than
    Ogt,
    // True if ordered and greater than or equal
    Oge,
    // True if ordered and less than
    Olt,
    // True if ordered and less than or equal
    Ole,
    // True if ordered and operands are unequal
    One,
    // True if ordered (no nans)
    Ord,
    // True if unordered: isnan(X) | isnan(Y)
    Uno,
    // True if unordered or equal
    Ueq,
    // True if unordered or greater than
    Ugt,
    // True if unordered, greater than, or equal
    Uge,
    // True if unordered or less than
    Ult,
    // True if unordered, less than, or equal
    Ule,
    // True if unordered or not equal
    Une,
    // Always true (always folded)
    True,
}

#[derive(Debug)]
pub enum InstrCalcCmpIntOp {
    // equal
    Eq,
    // not equal
    Ne,
    // unsigned greater than
    Ugt,
    // unsigned greater or equal
    Uge,
    // unsigned less than
    Ult,
    // unsigned less or equal
    Ule,
    // signed greater than
    Sgt,
    // signed greater or equal
    Sge,
    // signed less than
    Slt,
    // signed less or equal
    Sle,
}

#[derive(Debug)]
pub enum InstrCalcCmpOp {
    Float(InstrCalcCmpFloatOp),
    Int(InstrCalcCmpIntOp, bool),
}

#[derive(Debug)]
pub struct InstrCalcCmp {
    /// res var name
    pub res: Name,
    /// op
    pub op: InstrCalcCmpOp,
    /// left var name
    pub left: Name,
    /// right var name
    pub right: Name,
}

#[derive(Debug)]
pub struct InstrCalcExtractElt {
    /// res var name
    pub res: Name,
    /// src var name
    pub src: Name,
    /// src var name
    pub idx: Name,
}

#[derive(Debug)]
pub struct InstrCalcGEP {
    /// res var name
    pub res: Name,
    /// base var name
    pub base: Name,
    /// idx var name
    pub idx: Name,
}

#[derive(Debug)]
pub struct InstrCalcInsertElt {
    /// res var name
    pub res: Name,
    /// src var name
    pub src: Name,
    /// elt var name
    pub elt: Name,
    /// idx var name
    pub idx: Name,
}

#[derive(Debug)]
pub struct InstrCalcLoad {
    /// res var name
    pub res: Name,
    /// src var name
    pub src: Name,
}

#[derive(Debug)]
pub struct InstrCalcShuffleVec {
    /// res var name
    pub res: Name,
    /// vec1 var name
    pub vec1: Name,
    /// vec2 var name
    pub vec2: Name,
    /// mask var name
    pub mask: Name,
}

#[derive(Debug)]
pub struct InstrCalcStore {
    /// data var name
    pub data: Name,
    /// dst var name
    pub dst: Name,
}

#[derive(Debug)]
pub struct InstrCalcVSelect {
    /// data var name
    pub res: Name,
    /// cond var name
    pub cond: Name,
    /// val_true var name
    pub val_true: Name,
    /// val_false var name
    pub val_false: Name,
}

/// 计算指令
#[derive(Debug)]
pub enum InstrCalc {
    Alloca(InstrCalcAlloca),
    BinOp(InstrCalcBinOP),
    Call(InstrCalcCall),
    Cast(InstrCalcCast),
    Cmp(InstrCalcCmp),
    ExtractElt(InstrCalcExtractElt),
    Gep(InstrCalcGEP),
    InsertElt(InstrCalcInsertElt),
    Load(InstrCalcLoad),
    ShuffleVec(InstrCalcShuffleVec),
    Store(InstrCalcStore),
    VSelect(InstrCalcVSelect),
}

/// 终止指令
#[derive(Debug)]
pub enum InstrTerm {
    /// 返回某个变量
    Ret(Option<Name>),
    /// 直接跳转到block：block_name
    DirectBr(Name),
    /// 条件跳转：cond var name、block_name1、block_name2
    CondBr(Name, Name, Name),
}

#[derive(Debug)]
pub struct Block {
    /// 名字
    pub name: Name,
    /// 输入指令
    pub instrs_phi: Vec<InstrPhi>,
    /// 计算指令
    pub instrs_calc: Vec<InstrCalc>,
    /// 终止指令
    pub instr_term: InstrTerm,
}

#[derive(Debug)]
pub struct FunctionProto {
    pub ret: Ty,
    pub inps: Vec<Arg>,
}

#[derive(Debug)]
pub struct FunctionNative {
    /// 基本信息
    pub base: FunctionProto,
    /// 输入名字
    pub inps: Vec<Name>,
    /// 所有变量
    pub vars: Vars,
    /// 基本块
    pub blocks: Vec<Block>,
}


#[derive(Debug)]
pub struct LLVM {
    /// 版本信息
    pub version: Name,
    /// 本地函数
    pub funcs_native: HashMap<Name, FunctionNative>,
    /// 外联函数
    pub funcs_proto: HashMap<Name, FunctionProto>,
}

//! LLVM Meta

use std::{
    collections::HashMap,
    fmt::Debug,
    rc::Rc,
    sync::RwLock,
};

pub struct ID {
    pub(super) id: usize,
    pub name: RwLock<String>,
}

#[derive(Debug)]
pub struct IDGenerater {
    pub(super) idx: usize,
    pub(super) idx_var_const: usize,
    pub(super) record: HashMap<String, Name>,
}

pub type Name = Rc<ID>;

/// 基本类型
#[derive(Debug, Clone, PartialEq)]
pub enum TyBasic {
    Label,
    Void,
    Float,
    Double,
    Half,
    Int(u16),
}

/// 基本数据
#[derive(Debug, PartialEq)]
pub enum ValBasic {
    Null,
    Poison,
    Label(Name),
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
    Basic(TyBasic),
    Pointer(Option<Box<Self>>),
    Array(Box<Self>, usize),
    Vector(Box<Self>, usize),
    Struct(Vec<Self>),
    MetaData,
    Blob,
}

/// 所有变量：内含类型和数据
#[derive(Debug, PartialEq)]
pub enum Var {
    Basic(TyBasic, Option<ValBasic>),
    Pointer(Option<Box<Self>>),
    Array(Vec<Self>),
    Vector(Vec<Self>),
    Struct(Vec<Self>),
    MetaData(Option<Name>),
    Poison(Ty),
    Blob(Vec<u8>),
}

pub type Vars = HashMap<Name, Var>;

#[derive(Debug, Clone)]
pub struct Arg {
    pub ty: Ty,
    pub attr: Vec<Name>,
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
    pub align: u16,
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
    /// wrap: 不会从MAX+1到MIN
    Add(bool, bool),
    /// no signed wrap, no unsigned wrap
    Sub(bool, bool),
    /// no signed wrap, no unsigned wrap
    Mul(bool, bool),
    /// no signed wrap, no unsigned wrap
    Shl(bool, bool),
    /// is exact
    Sdiv(bool),
    /// is exact
    Udiv(bool),
    /// is exact
    Lshr(bool),
    /// is exact
    Ashr(bool),
    /// is disjoint
    Or(bool),
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
    /// func name
    pub func: Name,
    /// func attr
    pub attr: Option<u16>,
    /// ret ty/attr
    pub ret: Arg,
    /// arg ty/attr
    pub args: Vec<Arg>,

    pub tail: Name,
    pub conv: Name,

    /// res var name
    pub res: Option<Name>,
    /// inps var name
    pub inps: Vec<Name>,
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
    /// same_sign
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
    // no wrap flags
    pub in_bounds: bool,
    pub nusw: bool,
    pub nuw: bool,
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
    pub align: u16,
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
    pub align: u16,
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
pub enum MetaData {
    Value(Var),
    String(String),
    Meta(Name),
    Nodes(bool, Vec<Self>),
}

pub type MetaDatas = HashMap<Name, MetaData>;

#[derive(Debug)]
pub struct InstrMeta {
    pub name: Name,
    pub kind: Name,
}

#[derive(Debug)]
pub struct InstrWithMeta<T> {
    pub instr: T,
    pub metas: Vec<InstrMeta>,
}

#[derive(Debug)]
pub struct Block {
    /// 名字
    pub name: Name,
    /// 输入指令
    pub instrs_phi: Vec<InstrPhi>,
    /// 计算指令
    pub instrs_calc: Vec<InstrWithMeta<InstrCalc>>,
    /// 终止指令
    pub instr_term: InstrTerm,
}

#[derive(Debug, Clone)]
pub struct FunctionProto {
    pub ret: Arg,
    pub args: Vec<Arg>,
    pub link_age: Name,
    pub visibility: Name,
    pub dll_storage_class: Name,
    pub unnamed_addr: Name,
    pub attr: Option<u16>,
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
pub struct GlobalVar {
    pub is_const: bool,
    pub align: u16,
    pub link_age: Name,
    pub visibility: Name,
    pub dll_storage_class: Name,
    pub thread_local: Name,
    pub unnamed_addr: Name,
    pub var: Var
}

pub type AttrDatas = HashMap<u16, Vec<Name>>;
pub type GlobalVars = HashMap<Name, GlobalVar>;

#[derive(Debug)]
pub struct LLVM {
    /// ID all
    pub id: IDGenerater,
    // 基本信息
    pub version: Name,
    pub source_filename: Name,
    pub triple: Name,
    pub data_layout: Name,
    /// meta
    pub metas: MetaDatas,
    /// attrs
    pub attrs: AttrDatas,
    /// global var
    pub vars_global: GlobalVars,
    /// 本地函数
    pub funcs_native: HashMap<Name, FunctionNative>,
    /// 外联函数
    pub funcs_proto: HashMap<Name, FunctionProto>,
}

//! LLVM Meta

use std::{collections::HashMap, fmt::Debug, rc::Rc, sync::RwLock};

pub(super) struct IDInner {
    pub(super) id: usize,
    pub name: RwLock<String>,
}

#[derive(Clone)]
pub struct ID {
    pub(super) inner: Rc<IDInner>,
}

#[derive(Debug)]
pub struct IDGenerater {
    pub(super) idx: usize,
    pub(super) idx_var_const: usize,
    pub(super) record: HashMap<String, ID>,
}

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
#[derive(Debug, Clone, PartialEq)]
pub enum ValBasic {
    Poison,
    Label(ID),
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    F16(u16),
    F32(f32),
    F64(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FastMathFlags {
    pub allow_reassoc: bool,
    pub no_nans: bool,
    pub no_infs: bool,
    pub no_signed_zeros: bool,
    pub allow_reciprocal: bool,
    pub allow_contract: bool,
    pub approx_func: bool,
    pub all: bool,
    pub any: bool,
}


#[derive(Debug, Clone, PartialEq)]
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
    FpTrunc(Option<FastMathFlags>),
    FpExt(Option<FastMathFlags>),
}

#[derive(Debug, Clone, PartialEq)]
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
    Fdiv(Option<FastMathFlags>),
    Frem(Option<FastMathFlags>),
}


/// 所有类型
#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Basic(TyBasic),
    Pointer(Option<Box<Self>>),
    Array(Box<Self>, usize),
    Vector(Box<Self>, usize),
    /// packed
    Struct(Vec<Self>, bool),
    MetaData,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarInstrCast {
    pub op: InstrCalcCastOp,
    pub ty: Ty,
    pub val: Box<Var>
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarInstrBinOP {
    pub op: InstrCalcBinOPOp,
    pub ty: Ty,
    pub left: Box<Var>,
    pub right: Box<Var>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarInstrGEP {
    // no wrap flags
    pub in_bounds: bool,
    pub nusw: bool,
    pub nuw: bool,
    pub ty: Ty,
    pub base: Box<Var>,
    pub idx: Box<Var>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionProto {
    pub ret: Arg,
    pub args: Vec<Arg>,
    pub conv: ID,
    pub link_age: ID,
    pub visibility: ID,
    pub dll_storage_class: ID,
    pub unnamed_addr: ID,
    pub attr: Option<u16>,
    /// personality fn
    pub personality: Option<ID>,
}


/// 所有变量：内含类型和数据
#[derive(Debug, Clone, PartialEq)]
pub enum Var {
    Basic(TyBasic, Option<ValBasic>),
    Pointer(Option<Box<Self>>),
    Array(Ty, Vec<Self>),
    Vector(Ty, Vec<Self>),
    /// packed
    Struct(bool, Vec<Self>),
    MetaData(Option<ID>),
    Poison(Ty),
    /// has_null, data
    String(bool, Rc<Vec<u8>>),
    Null(Ty),
    Undef(Ty),
    InstrCast(VarInstrCast),
    InstrBinOP(VarInstrBinOP),
    InstrGEP(VarInstrGEP),
    /// ptr to function/global var
    Ptr(ID),
}

pub type Vars = HashMap<ID, Var>;

#[derive(Debug, Clone, PartialEq)]
pub struct Arg {
    pub ty: Ty,
    pub attr: Vec<ID>,
}

#[derive(Debug)]
pub struct InstrPhiSrc {
    pub block: ID,
    pub var: ID,
}

/// 变量输入指令
#[derive(Debug)]
pub struct InstrPhi {
    /// res var name
    pub res: ID,
    /// 来源：block_name, var_name
    pub srcs: Vec<InstrPhiSrc>,
}

#[derive(Debug)]
pub struct InstrCalcAlloca {
    /// res var name
    pub res: ID,
    pub align: u16,
}


#[derive(Debug)]
pub struct InstrCalcBinOP {
    /// res var name
    pub res: ID,
    /// op
    pub op: InstrCalcBinOPOp,
    /// left var name
    pub left: ID,
    /// right var name
    pub right: ID,
}

#[derive(Debug)]
pub enum InstrFuncAsmDialect {
    Att,
    Intel,
}

#[derive(Debug)]
pub struct InstrFuncAsm {
    pub ret: Arg,
    pub args: Vec<Arg>,
    pub asm: Rc<String>,
    pub constr: Rc<String>,
    pub has_side_effects: bool,
    pub is_align_stack: bool,
    pub can_throw: bool,
    pub asm_dialect: InstrFuncAsmDialect,
}


#[derive(Debug)]
pub enum InstrFunc {
    /// 普通函数
    Func(ID),
    /// 虚拟函数，idx_block, idx_instr
    VirtualFunc(ID),
    /// asm
    Asm(InstrFuncAsm),
}


#[derive(Debug)]
pub struct InstrCalcCall {
    /// func name
    pub func: InstrFunc,
    /// func attr
    pub attr: Option<u16>,
    /// ret ty/attr
    pub ret: Arg,
    /// arg ty/attr
    pub args: Vec<Arg>,

    pub tail: ID,
    pub conv: ID,
    pub fmf: Option<FastMathFlags>,

    /// res var name
    pub res: Option<ID>,
    /// inps var name
    pub inps: Vec<ID>,
}

#[derive(Debug)]
pub struct InstrCalcCast {
    /// res var name
    pub res: ID,
    /// op
    pub op: InstrCalcCastOp,
    /// inp var name
    pub val: ID,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum InstrCalcCmpOp {
    Float(InstrCalcCmpFloatOp, Option<FastMathFlags>),
    /// same_sign
    Int(InstrCalcCmpIntOp, bool),
}

#[derive(Debug)]
pub struct InstrCalcCmp {
    /// res var name
    pub res: ID,
    /// op
    pub op: InstrCalcCmpOp,
    /// left var name
    pub left: ID,
    /// right var name
    pub right: ID,
}

#[derive(Debug)]
pub struct InstrCalcExtractElt {
    /// res var name
    pub res: ID,
    /// src var name
    pub src: ID,
    /// idx var name
    pub idx: ID,
}

#[derive(Debug)]
pub struct InstrCalcGEP {
    // no wrap flags
    pub in_bounds: bool,
    pub nusw: bool,
    pub nuw: bool,
    /// res var name
    pub res: ID,
    /// base var name
    pub base: ID,
    /// idx var name
    pub idx: ID,
}

#[derive(Debug, Clone)]
pub struct InstrCalcInsertElt {
    /// res var name
    pub res: ID,
    /// src var name
    pub src: ID,
    /// elt var name
    pub elt: ID,
    /// idx var name
    pub idx: ID,
}

#[derive(Debug)]
pub struct InstrCalcLoad {
    /// res var name
    pub res: ID,
    /// src var name
    pub src: ID,
    pub align: u16,
}

#[derive(Debug)]
pub struct InstrCalcShuffleVec {
    /// res var name
    pub res: ID,
    /// vec1 var name
    pub vec1: ID,
    /// vec2 var name
    pub vec2: ID,
    /// mask var name
    pub mask: ID,
}

#[derive(Debug)]
pub struct InstrCalcStore {
    /// data var name
    pub data: ID,
    /// dst var name
    pub dst: ID,

    pub is_volatile: bool,
    pub align: u16,
}

#[derive(Debug)]
pub struct InstrCalcVSelect {
    /// data var name
    pub res: ID,
    /// cond var name
    pub cond: ID,
    /// val_true var name
    pub val_true: ID,
    /// val_false var name
    pub val_false: ID,
    
    pub fmf: Option<FastMathFlags>
}

#[derive(Debug, Clone)]
pub enum InstrCalcAtomicOrdering {
    NotAtomic,
    Unordered,
    Monotonic,
    Acquire,
    Release,
    AcquireRelease,
    SequentiallyConsistent,
}

#[derive(Debug)]
pub struct InstrCalcLoadAtomic {
    pub res: ID,
    pub src: ID,
    pub ordering: InstrCalcAtomicOrdering,
    pub ssid: ID,
    pub is_volatile: bool,
    pub align: u16,
}

#[derive(Debug)]
pub struct InstrCalcStoreAtomic {
    pub data: ID,
    pub dst: ID,
    pub ordering: InstrCalcAtomicOrdering,
    pub ssid: ID,
    pub is_volatile: bool,
    pub align: u16,
}

#[derive(Debug)]
pub struct InstrCalcCmpxchg {
    pub res: ID,
    pub ptr: ID,
    pub cmp: ID,
    pub val: ID,
    pub success_ordering: InstrCalcAtomicOrdering,
    pub failure_ordering: InstrCalcAtomicOrdering,
    pub ssid: ID,
    pub is_volatile: bool,
    pub is_weak: bool,
    pub align: u16,
}

#[derive(Debug)]
pub struct InstrCalcExtractVal {
    /// res var name
    pub res: ID,
    /// src var name
    pub src: ID,
    /// idxs
    pub idxs: Vec<usize>,
}

#[derive(Debug)]
pub struct InstrCalcInsertVal {
    /// res var name
    pub res: ID,
    /// src var name
    pub src: ID,
    /// val var name
    pub val: ID,
    /// idxs
    pub idxs: Vec<usize>,
}


#[derive(Debug)]
pub struct InstrCalcFence {
    pub ordering: InstrCalcAtomicOrdering,
    pub ssid: ID,
}

#[derive(Debug)]
pub enum InstrCalcLandingPadClauseOp {
    Catch,
    Filter
}

#[derive(Debug)]
pub struct InstrCalcLandingPadClause {
    pub ty: InstrCalcLandingPadClauseOp,
    pub val: ID,
}

#[derive(Debug)]
pub struct InstrCalcLandingPad {
    pub res: ID,
    pub is_cleanup: bool,
    pub clauses: Vec<InstrCalcLandingPadClause>,
}

#[derive(Debug)]
pub struct InstrCalcFreeze {
    pub res: ID,
    pub op: ID,
}

#[derive(Debug, Clone)]
pub enum InstrCalcAtomicRMWOp {
    Xchg,
    Add,
    Sub,
    And,
    Nand,
    Or,
    Xor,
    Max,
    Min,
    Umax,
    Umin,
    Fadd,
    Fsub,
    Fmax,
    Fmin,
    UincWrap,
    UdecWrap,
    UsubCond,
    UsubSat,
    Fmaximum,
    Fminimum,
}

#[derive(Debug)]
pub struct InstrCalcAtomicRMW {
    pub res: ID,
    pub ptr: ID,
    pub val: ID,
    pub op: InstrCalcAtomicRMWOp,
    pub ordering: InstrCalcAtomicOrdering,
    pub ssid: ID,
    pub is_volatile: bool,
    pub align: u16,
}

#[derive(Debug, Clone)]
pub enum InstrCalcUnOpOp {
    FNeg(Option<FastMathFlags>)
}

#[derive(Debug)]
pub struct InstrCalcUnOp {
    pub res: ID,
    pub val: ID,
    pub op: InstrCalcUnOpOp,
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
    LoadAtomic(InstrCalcLoadAtomic),
    StoreAtomic(InstrCalcStoreAtomic),
    Cmpxchg(InstrCalcCmpxchg),
    ExtractVal(InstrCalcExtractVal),
    InsertVal(InstrCalcInsertVal),
    Fence(InstrCalcFence),
    LandingPad(InstrCalcLandingPad),
    Freeze(InstrCalcFreeze),
    AtomicRMW(InstrCalcAtomicRMW),
    UnOp(InstrCalcUnOp),
}

#[derive(Debug)]
pub struct InstrTermRet {
    pub var: Option<ID>,
}

#[derive(Debug)]
pub struct InstrTermDirectBr {
    pub block: ID,
}


#[derive(Debug)]
pub struct InstrTermCondBr {
    pub cond: ID,
    pub block_true: ID,
    pub block_false: ID,
}

/// bit_width, val
#[derive(Debug, Clone)]
pub enum InstrTermSwitchValue {
    Int8(u16, i8),
    Int16(u16, i16),
    Int32(u16, i32),
    Int64(u16, i64),
    Int128(u16, i128),
}

#[derive(Debug)]
pub struct InstrTermSwitchCase {
    pub val: InstrTermSwitchValue,
    pub block: ID,
}

#[derive(Debug)]
pub struct InstrTermSwitch {
    pub cond: ID,
    pub block_default: ID,
    pub cases: Vec<InstrTermSwitchCase>
}

#[derive(Debug)]
pub struct InstrTermInvoke {
    /// func name
    pub func: InstrFunc,
    /// func attr
    pub attr: Option<u16>,
    /// ret ty/attr
    pub ret: Arg,
    /// arg ty/attr
    pub args: Vec<Arg>,

    pub block_normal: ID,
    pub block_unwind: ID,
    
    pub conv: ID,

    /// res var name
    pub res: Option<ID>,
    /// inps var name
    pub inps: Vec<ID>,
}

#[derive(Debug)]
pub struct InstrTermResume {
    pub var: ID,
}

/// 终止指令
#[derive(Debug)]
pub enum InstrTerm {
    /// 返回某个变量
    Ret(InstrTermRet),
    /// 直接跳转到block：block_name
    DirectBr(InstrTermDirectBr),
    /// 条件跳转：cond var name、block_name1、block_name2
    CondBr(InstrTermCondBr),
    /// Switch: cond var name, default block_name, [block_name, value]
    Switch(InstrTermSwitch),
    /// Invoke
    Invoke(Box<InstrTermInvoke>),
    /// unreachable
    Unreachable,
    /// resume
    Resume(InstrTermResume),
}

#[derive(Debug, Clone)]
pub enum MetaData {
    Value(Var),
    String(String),
    Func(Rc<String>),
    Meta(ID),
    Nodes(bool, Vec<Self>),
}

pub type MetaDatas = HashMap<ID, MetaData>;

#[derive(Debug, Clone)]
pub struct InstrMeta {
    pub name: ID,
    pub kind: ID,
}

#[derive(Debug)]
pub struct InstrWithMeta<T> {
    pub instr: T,
    pub metas: Vec<InstrMeta>,
}

#[derive(Debug)]
pub struct Block {
    /// 名字
    pub name: ID,
    /// 输入指令
    pub instrs_phi: Vec<InstrPhi>,
    /// 计算指令
    pub instrs_calc: Vec<InstrWithMeta<InstrCalc>>,
    /// 终止指令
    pub instr_term: InstrTerm,
}

#[derive(Debug)]
pub struct FunctionNative {
    /// 基本信息
    pub base: FunctionProto,
    /// 输入名字
    pub inps: Vec<ID>,
    /// 所有变量
    pub vars: Vars,
    /// 基本块
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone)]
pub struct GlobalVar {
    pub is_const: bool,
    pub align: Option<u16>,
    pub link_age: ID,
    pub visibility: ID,
    pub dll_storage_class: ID,
    pub thread_local: ID,
    pub unnamed_addr: ID,
    pub var: Var,
}

pub type AttrDatas = HashMap<u16, Vec<ID>>;
pub type GlobalVars = HashMap<ID, GlobalVar>;

#[derive(Debug)]
pub struct LLVM {
    /// ID all
    pub id: IDGenerater,
    // 基本信息
    pub version: ID,
    pub source_filename: ID,
    pub triple: ID,
    pub data_layout: ID,
    /// meta
    pub metas: MetaDatas,
    /// attrs
    pub attrs: AttrDatas,
    /// global var
    pub vars_global: GlobalVars,
    /// 本地函数
    pub funcs_native: HashMap<ID, FunctionNative>,
    /// 外联函数
    pub funcs_proto: HashMap<ID, FunctionProto>,
}

//! utils

use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt::{Debug, Display, Error},
    fs::File,
    hash::Hash,
    io::Write,
    path::Path,
    rc::Rc,
    sync::{RwLock, RwLockWriteGuard},
};

use crate::{
    LLVM, WError, WResult,
    meta::{
        Arg, FunctionNative, FunctionProto, GlobalVar, ID, IDGenerater, IDInner, InstrCalc, InstrCalcCall, InstrFunc, InstrPhi, InstrPhiSrc, InstrTermSwitchCase, InstrWithMeta, Ty, TyBasic, ValBasic, Var
    },
};

pub const ID_STATE_GLOBAL_VAR: &str = "global_var";
pub const ID_TAG_CONST: &str = "const";
pub const ID_STATE_ATTR: &str = "attr";
pub const ID_STATE_META_KIND: &str = "meta_kind";
pub const ID_STATE_FUNC: &str = "func";
pub const ID_STATE_BASIC: &str = "basic";
pub const ID_STATE_META: &str = "meta";
pub const ID_TAG_BLOCK: &str = "block";
pub const ID_TAG_VAR: &str = "var";

pub const ATTR_CALL_TAIL_NONE: &str = "";
pub const ATTR_CALL_TAIL_TAIL: &str = "tail";
pub const ATTR_CALL_TAIL_MUST: &str = "musttail";
pub const ATTR_CALL_TAIL_NO: &str = "notail";

pub const ATTR_LINK_AGE_EXTERNAL: &str = "";
pub const ATTR_LINK_AGE_AVAILABLEEXTERNALLY: &str = "available_externally";
pub const ATTR_LINK_AGE_LINKONCEODR: &str = "linkonce_odr";
pub const ATTR_LINK_AGE_EXTERNWEAK: &str = "extern_weak";
pub const ATTR_LINK_AGE_WEAKODR: &str = "weak_odr";
pub const ATTR_LINK_AGE_WEAK: &str = "weak";
pub const ATTR_LINK_AGE_APPENDING: &str = "appending";
pub const ATTR_LINK_AGE_INTERNAL: &str = "internal";
pub const ATTR_LINK_AGE_LINKONCE: &str = "linkonce";
pub const ATTR_LINK_AGE_DLLIMPORT: &str = "dllimport";
pub const ATTR_LINK_AGE_DLLEXPORT: &str = "dllexport";
pub const ATTR_LINK_AGE_COMMON: &str = "common";
pub const ATTR_LINK_AGE_PRIVATE: &str = "private";

pub const ATTR_VISIBILITY_DEFAULT: &str = "";
pub const ATTR_VISIBILITY_HIDDEN: &str = "hidden";
pub const ATTR_VISIBILITY_PROTECTED: &str = "protected";

pub const ATTR_DLL_STORAGE_CLASS_DEFAULT: &str = "";
pub const ATTR_DLL_STORAGE_CLASS_DLLIMPORT: &str = "dllimport";
pub const ATTR_DLL_STORAGE_CLASS_DLLEXPORT: &str = "dllexport";

pub const ATTR_THREAD_LOCAL_NOTTHREADLOCAL: &str = "";
pub const ATTR_THREAD_LOCAL_THREADLOCAL: &str = "thread_local";
pub const ATTR_THREAD_LOCAL_LOCALDYNAMIC: &str = "thread_local(localdynamic)";
pub const ATTR_THREAD_LOCAL_INITIALEXEC: &str = "thread_local(initialexec)";
pub const ATTR_THREAD_LOCAL_LOCALEXEC: &str = "thread_local(locexec)";

pub const ATTR_UNNAMED_ADDR_NONE: &str = "";
pub const ATTR_UNNAMED_ADDR_GLOBAL: &str = "unnamed_addr";
pub const ATTR_UNNAMED_ADDR_LOCAL: &str = "local_unnamed_addr";

pub const ATTR_CALLING_CONV_CCC: &str = "";
pub const ATTR_CALLING_CONV_FASTCC: &str = "fastcc";
pub const ATTR_CALLING_CONV_COLDCC: &str = "coldcc";
pub const ATTR_CALLING_CONV_GHCCC: &str = "ghccc";
pub const ATTR_CALLING_CONV_WEBKITJSCC: &str = "webkit_jscc";
pub const ATTR_CALLING_CONV_ANYREGCC: &str = "anyregcc";
pub const ATTR_CALLING_CONV_PRESERVEMOSTCC: &str = "preserve_mostcc";
pub const ATTR_CALLING_CONV_PRESERVEALLCC: &str = "preserve_allcc";
pub const ATTR_CALLING_CONV_SWIFTCC: &str = "swiftcc";
pub const ATTR_CALLING_CONV_CXXFASTTLSCC: &str = "cxx_fast_tlscc";
pub const ATTR_CALLING_CONV_TAILCC: &str = "tailcc";
pub const ATTR_CALLING_CONV_CFGUARDCHECKCC: &str = "cfguard_checkcc";
pub const ATTR_CALLING_CONV_SWIFTTAILCC: &str = "swifttailcc";
pub const ATTR_CALLING_CONV_X86STDCALLCC: &str = "x86_stdcallcc";
pub const ATTR_CALLING_CONV_X86FASTCALLCC: &str = "x86_fastcallcc";
pub const ATTR_CALLING_CONV_ARMAPCSCC: &str = "arm_apcscc";
pub const ATTR_CALLING_CONV_ARMAAPCSCC: &str = "arm_aapcscc";
pub const ATTR_CALLING_CONV_ARMAAPCSVFPCC: &str = "arm_aapcs_vfpcc";

pub const ATTR_PARAM_ALLOCALIGN: &str = "allocalign";
pub const ATTR_PARAM_ALLOCATEDPOINTER: &str = "allocatedpointer";
pub const ATTR_PARAM_ALWAYSINLINE: &str = "alwaysinline";
pub const ATTR_PARAM_BUILTIN: &str = "builtin";
pub const ATTR_PARAM_COLD: &str = "cold";
pub const ATTR_PARAM_CONVERGENT: &str = "convergent";
pub const ATTR_PARAM_COROONLYDESTROYWHENCOMPLETE: &str = "coroonlydestroywhencomplete";
pub const ATTR_PARAM_COROELIDESAFE: &str = "coroelidesafe";
pub const ATTR_PARAM_DEADONRETURN: &str = "dead_on_return";
pub const ATTR_PARAM_DEADONUNWIND: &str = "dead_on_unwind";
pub const ATTR_PARAM_DISABLESANITIZERINSTRUMENTATION: &str = "disablesanitizerinstrumentation";
pub const ATTR_PARAM_FNRETTHUNKEXTERN: &str = "fnretthunkextern";
pub const ATTR_PARAM_HOT: &str = "hot";
pub const ATTR_PARAM_HYBRIDPATCHABLE: &str = "hybridpatchable";
pub const ATTR_PARAM_IMMARG: &str = "immarg";
pub const ATTR_PARAM_INREG: &str = "inreg";
pub const ATTR_PARAM_INLINEHINT: &str = "inlinehint";
pub const ATTR_PARAM_JUMPTABLE: &str = "jumptable";
pub const ATTR_PARAM_MINSIZE: &str = "minsize";
pub const ATTR_PARAM_MUSTPROGRESS: &str = "mustprogress";
pub const ATTR_PARAM_NAKED: &str = "naked";
pub const ATTR_PARAM_NEST: &str = "nest";
pub const ATTR_PARAM_NOALIAS: &str = "noalias";
pub const ATTR_PARAM_NOBUILTIN: &str = "nobuiltin";
pub const ATTR_PARAM_NOCALLBACK: &str = "nocallback";
pub const ATTR_PARAM_NOCFCHECK: &str = "nocfcheck";
pub const ATTR_PARAM_NOCREATEUNDEFORPOISON: &str = "nocreateundeforpoison";
pub const ATTR_PARAM_NODIVERGENCESOURCE: &str = "nodivergencesource";
pub const ATTR_PARAM_NODUPLICATE: &str = "noduplicate";
pub const ATTR_PARAM_NOEXT: &str = "noext";
pub const ATTR_PARAM_NOFREE: &str = "nofree";
pub const ATTR_PARAM_NOIMPLICITFLOAT: &str = "noimplicitfloat";
pub const ATTR_PARAM_NOINLINE: &str = "noinline";
pub const ATTR_PARAM_NOMERGE: &str = "nomerge";
pub const ATTR_PARAM_NOPROFILE: &str = "noprofile";
pub const ATTR_PARAM_NORECURSE: &str = "norecurse";
pub const ATTR_PARAM_NOREDZONE: &str = "noredzone";
pub const ATTR_PARAM_NORETURN: &str = "noreturn";
pub const ATTR_PARAM_NOSANITIZEBOUNDS: &str = "nosanitizebounds";
pub const ATTR_PARAM_NOSANITIZECOVERAGE: &str = "nosanitizecoverage";
pub const ATTR_PARAM_NOSYNC: &str = "nosync";
pub const ATTR_PARAM_NOUNDEF: &str = "noundef";
pub const ATTR_PARAM_NOUNWIND: &str = "nounwind";
pub const ATTR_PARAM_NONLAZYBIND: &str = "nonlazybind";
pub const ATTR_PARAM_NONNULL: &str = "nonnull";
pub const ATTR_PARAM_NULLPOINTERISVALID: &str = "nullpointerisvalid";
pub const ATTR_PARAM_OPTFORFUZZING: &str = "optforfuzzing";
pub const ATTR_PARAM_OPTIMIZEFORDEBUGGING: &str = "optimizefordebugging";
pub const ATTR_PARAM_OPTIMIZEFORSIZE: &str = "optimizeforsize";
pub const ATTR_PARAM_OPTIMIZENONE: &str = "optimizenone";
pub const ATTR_PARAM_PRESPLITCOROUTINE: &str = "presplitcoroutine";
pub const ATTR_PARAM_READNONE: &str = "readnone";
pub const ATTR_PARAM_READONLY: &str = "readonly";
pub const ATTR_PARAM_RETURNED: &str = "returned";
pub const ATTR_PARAM_RETURNSTWICE: &str = "returnstwice";
pub const ATTR_PARAM_SEXT: &str = "sext";
pub const ATTR_PARAM_SAFESTACK: &str = "safestack";
pub const ATTR_PARAM_SANITIZEADDRESS: &str = "sanitizeaddress";
pub const ATTR_PARAM_SANITIZEALLOCTOKEN: &str = "sanitizealloctoken";
pub const ATTR_PARAM_SANITIZEHWADDRESS: &str = "sanitizehwaddress";
pub const ATTR_PARAM_SANITIZEMEMTAG: &str = "sanitizememtag";
pub const ATTR_PARAM_SANITIZEMEMORY: &str = "sanitizememory";
pub const ATTR_PARAM_SANITIZENUMERICALSTABILITY: &str = "sanitizenumericalstability";
pub const ATTR_PARAM_SANITIZEREALTIME: &str = "sanitizerealtime";
pub const ATTR_PARAM_SANITIZEREALTIMEBLOCKING: &str = "sanitizerealtimeblocking";
pub const ATTR_PARAM_SANITIZETHREAD: &str = "sanitizethread";
pub const ATTR_PARAM_SANITIZETYPE: &str = "sanitizetype";
pub const ATTR_PARAM_SHADOWCALLSTACK: &str = "shadowcallstack";
pub const ATTR_PARAM_SKIPPROFILE: &str = "skipprofile";
pub const ATTR_PARAM_SPECULATABLE: &str = "speculatable";
pub const ATTR_PARAM_SPECULATIVELOADHARDENING: &str = "speculativeloadhardening";
pub const ATTR_PARAM_STACKPROTECT: &str = "stackprotect";
pub const ATTR_PARAM_STACKPROTECTREQ: &str = "stackprotectreq";
pub const ATTR_PARAM_STACKPROTECTSTRONG: &str = "stackprotectstrong";
pub const ATTR_PARAM_STRICTFP: &str = "strictfp";
pub const ATTR_PARAM_SWIFTASYNC: &str = "swiftasync";
pub const ATTR_PARAM_SWIFTERROR: &str = "swifterror";
pub const ATTR_PARAM_SWIFTSELF: &str = "swiftself";
pub const ATTR_PARAM_WILLRETURN: &str = "willreturn";
pub const ATTR_PARAM_WRITABLE: &str = "writable";
pub const ATTR_PARAM_WRITEONLY: &str = "writeonly";
pub const ATTR_PARAM_ZEXT: &str = "zeroext";

pub type IDState<'a> = HashMap<&'a ID, &'a str>;

impl PartialEq for ID {
    fn eq(&self, other: &Self) -> bool {
        self.inner.id == other.inner.id
    }
}

impl Eq for ID {}

impl PartialOrd for ID {
    fn ge(&self, other: &Self) -> bool {
        self.inner.id >= other.inner.id
    }

    fn gt(&self, other: &Self) -> bool {
        self.inner.id > other.inner.id
    }

    fn le(&self, other: &Self) -> bool {
        self.inner.id <= other.inner.id
    }

    fn lt(&self, other: &Self) -> bool {
        self.inner.id < other.inner.id
    }

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ID {
    fn min(self, other: Self) -> Self {
        if self.inner.id < other.inner.id {
            self
        } else {
            other
        }
    }

    fn max(self, other: Self) -> Self {
        if self.inner.id > other.inner.id {
            self
        } else {
            other
        }
    }

    fn cmp(&self, other: &Self) -> Ordering {
        if self.inner.id > other.inner.id {
            Ordering::Greater
        } else if self.inner.id == other.inner.id {
            Ordering::Equal
        } else {
            Ordering::Less
        }
    }

    fn clamp(self, min: Self, max: Self) -> Self {
        if self.inner.id < min.inner.id {
            min
        } else if self.inner.id > max.inner.id {
            max
        } else {
            self
        }
    }
}

impl ID {
    pub fn with_data<F, R>(&self, f: F) -> WResult<R>
    where
        F: FnOnce(&str) -> R,
    {
        match self.inner.name.read().as_deref() {
            Ok(v) => Ok(f(v.as_str())),
            Err(_) => Err(WError::DataNotFound("get name")),
        }
    }

    pub fn empty() -> Self {
        let name = RwLock::new(String::with_capacity(0));
        let id = 0;
        Self {
            inner: Rc::new(IDInner { id, name }),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.id == 0
    }

    /// 更改内部数据
    pub fn data(&self) -> WResult<RwLockWriteGuard<'_, String>> {
        let r = self.inner.name.write()?;
        Ok(r)
    }

    /// 一共被使用的次数
    pub fn count(&self) -> usize {
        Rc::strong_count(&self.inner)
    }

    /// 在给定IDGenerater内生成相同的ID，需要提供原始IDGenerater的state
    pub fn renew(&self, state: &IDState, id: &mut IDGenerater) -> WResult<Self> {
        let state = state
            .get(&self)
            .ok_or(WError::DataNotFound("state key"))?
            .to_string();
        self.with_data(|n| id.generate_with_id(state, n))
    }
}

impl Hash for ID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.inner.id);
    }
}

impl Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.inner.name.read().as_deref() {
            Ok(v) => write!(f, "{}", v),
            Err(_) => Err(Error),
        }
    }
}

impl Debug for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self, self.inner.id)
    }
}

impl Default for IDGenerater {
    fn default() -> Self {
        Self {
            idx: 1,
            idx_var_const: 0,
            record: HashMap::with_capacity(2048),
        }
    }
}

impl AsRef<HashMap<String, ID>> for IDGenerater {
    fn as_ref(&self) -> &HashMap<String, ID> {
        &self.record
    }
}

impl IDGenerater {
    /// 继承生成一个新的
    /// 可以后续使用renew
    /// 内部ID不会和renew的冲突
    pub fn inherit<'a>(&'a self) -> (Self, IDState<'a>) {
        let state = self
            .record
            .iter()
            .map(|(k, v)| (v, k.as_str()))
            .collect::<HashMap<_, _>>();
        let r = Self {
            idx: self.idx,
            idx_var_const: self.idx_var_const,
            record: HashMap::with_capacity(2048),
        };
        (r, state)
    }

    /// 生成一个id（最基础的生成id函数）
    /// 使用现有str id生成一个id name
    /// 内部id使用给定str id
    /// 如果重复直接返回已有id name
    pub fn generate_with_id(&mut self, id: String, data: &str) -> ID {
        if let Some(name_old) = self.record.get(&id) {
            return name_old.clone();
        }
        let res = ID {
            inner: Rc::new(IDInner {
                id: self.idx,
                name: RwLock::new(data.to_string()),
            }),
        };
        self.record.insert(id, res.clone());
        self.idx += 1;
        res
    }

    /// 生成常量id name
    /// 内部str id为ID_TAG_CONST + _ + value
    pub fn generate_const(&mut self, value: String) -> ID {
        let id = format!("{}_{}", ID_TAG_CONST, value);
        if let Some(name) = self.record.get(&id) {
            return name.clone();
        }

        let r = format!("Const_{}", self.idx_var_const);
        self.idx_var_const += 1;
        // 使用值当作name，相同的const就会被合并
        self.generate_with_id(id, &r)
    }

    /// 生成一个id
    /// 将state和data融合为内部str id
    /// 然后调用generate_with_id
    pub fn generate(&mut self, state: &str, data: &str) -> ID {
        let id = format!("{}_{}", state, data);
        self.generate_with_id(id, data)
    }

    /// 生成一个id
    /// 带有名字，不同名字用来分割不同作用域
    /// 比如名字是function name
    /// 将tag和name融合为state，然后调用generate
    pub fn generate_with_name(&mut self, tag: &'static str, name: &str, data: &str) -> ID {
        let tag = format!("{}_{}", tag, name);
        self.generate(&tag, data)
    }

    /// 清空记录
    pub fn clear(&mut self) {
        self.record.clear();
    }

    /// 清除其他地方不再使用的id
    pub fn clean(&mut self) {
        self.record.retain(|_, v| v.count() != 1);
    }

    /// 从内部取出一个id（最基础的取出id的函数）
    /// 使用给定str id取出id
    pub fn get_with_id(&self, id: &str) -> WResult<ID> {
        self.record
            .get(id)
            .cloned()
            .ok_or(WError::DataNotAllowed("id get", id.to_string()))
    }

    /// 从内部取出一个id
    /// 将state和data融合为内部str id
    /// 然后调用get_with_id
    pub fn get(&self, state: &str, data: &str) -> WResult<ID> {
        let id = format!("{}_{}", state, data);
        self.get_with_id(&id)
    }

    /// 从内部取出一个id
    /// 带有名字，不同名字用来分割不同作用域
    /// 比如名字是function name
    /// 将tag和name融合为state，然后调用get
    pub fn get_with_name(&self, tag: &'static str, name: &str, data: &str) -> WResult<ID> {
        let tag = format!("{}_{}", tag, name);
        self.get(&tag, data)
    }
}

impl From<Ty> for Arg {
    fn from(ty: Ty) -> Self {
        Self {
            ty,
            attr: Vec::with_capacity(0),
        }
    }
}

impl Arg {
    pub fn renew(&self, state: &IDState, id: &mut IDGenerater) -> WResult<Self> {
        let attr = self
            .attr
            .iter()
            .map(|v| v.renew(state, id))
            .collect::<WResult<Vec<_>>>()?;
        Ok(Self {
            ty: self.ty.clone(),
            attr,
        })
    }
}

impl From<TyBasic> for Ty {
    fn from(value: TyBasic) -> Self {
        Self::Basic(value)
    }
}

impl TryFrom<Ty> for Var {
    type Error = WError;
    fn try_from(ty: Ty) -> WResult<Self> {
        let r = match ty {
            Ty::Basic(TyBasic::Void) => {
                return Err(WError::DataNotAllowed("Ty to Var", format!("{}", ty)));
            }
            Ty::Basic(v) => Self::Basic(v, None),
            Ty::Pointer(Some(v)) => {
                Self::Pointer(Some(Box::new(Var::try_from(v.as_ref().clone())?)))
            }
            Ty::Pointer(None) => Self::Pointer(None),
            Ty::Array(v, n) => Self::Array(
                *v.clone(),
                (0..n)
                    .map(|_| Var::try_from(v.as_ref().clone()))
                    .collect::<WResult<Vec<_>>>()?,
            ),
            Ty::Vector(v, n) => Self::Vector(
                *v.clone(),
                (0..n)
                    .map(|_| Var::try_from(v.as_ref().clone()))
                    .collect::<WResult<Vec<_>>>()?,
            ),
            Ty::Struct(vs, p) => Self::Struct(
                p,
                vs.into_iter()
                    .map(Var::try_from)
                    .collect::<WResult<Vec<_>>>()?,
            ),
            Ty::MetaData => Self::MetaData(None),
        };
        Ok(r)
    }
}

impl Var {
    /// 将basic值填充poison
    pub fn fill_basic_poison(&mut self) -> WResult<()> {
        match self {
            Self::Basic(TyBasic::Int(_), v) if v.is_none() => {
                v.replace(ValBasic::Poison);
            }
            Self::Array(_, vs) | Self::Vector(_, vs) | Self::Struct(_, vs) => {
                for v in vs.iter_mut() {
                    v.fill_basic_poison()?;
                }
            }
            Self::Pointer(Some(v)) => {
                v.fill_basic_poison()?;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn ty(&self) -> Ty {
        match self {
            Self::Basic(v, _) => Ty::Basic(v.clone()),
            Self::Pointer(Some(v)) => Ty::Pointer(Some(Box::new(v.ty()))),
            Self::Pointer(None) => Ty::Pointer(None),
            Self::Array(ty, vs) => {
                let n = vs.len();
                Ty::Array(Box::new(ty.clone()), n)
            }
            Self::Vector(ty, vs) => {
                let n = vs.len();
                Ty::Vector(Box::new(ty.clone()), n)
            }
            Self::Struct(p, vs) => {
                let tys = vs.iter().map(|v| v.ty()).collect::<Vec<_>>();
                Ty::Struct(tys, *p)
            }
            Self::MetaData(_) => Ty::MetaData,
            Self::Poison(v) | Self::Null(v) | Self::Undef(v) => v.clone(),
            Self::String(p, v) => Ty::Array(
                Box::new(Ty::Basic(TyBasic::Int(8))),
                v.len() + (*p as usize)
            ),
            Self::InstrCast(instr) => instr.ty.clone(),
            Self::InstrGEP(instr) => Ty::Pointer(Some(Box::new(instr.ty.clone()))),
            Self::InstrBinOP(instr) => instr.ty.clone(),
            Self::Ptr(_) => Ty::Pointer(None)
        }
    }

    pub fn is_const(&self) -> bool {
        match self {
            Self::Basic(_, v) => v.is_some(),
            Self::Pointer(Some(v)) => v.is_const(),
            Self::Pointer(None) => false,
            Self::Array(_, vs) | Self::Vector(_, vs) | Self::Struct(_, vs) => {
                vs.iter().any(|v| v.is_const())
            }
            _ => true,
        }
    }

    pub fn convert_idx(&self) -> WResult<usize> {
        let (ty, val) = if let Var::Basic(ty, Some(v)) = self {
            (ty, v)
        } else if matches!(self,
            Var::Null(Ty::Basic(TyBasic::Int(8 | 32 | 64 | 16)))
            | Var::Poison(Ty::Basic(TyBasic::Int(8 | 32 | 64 | 16)))
        ) {
            return Ok(0)
        } else {
            return Err(WError::DataNotAllowed(
                "var convert idx",
                format!("{self:?}"),
            ));
        };
        let r = match (ty, val) {
            (TyBasic::Int(8), ValBasic::I8(v)) => *v as usize,
            (TyBasic::Int(32), ValBasic::I32(v)) => *v as usize,
            (TyBasic::Int(64), ValBasic::I64(v)) => *v as usize,
            (TyBasic::Int(16), ValBasic::I16(v)) => *v as usize,
            (TyBasic::Int(8 | 32 | 64 | 16), ValBasic::Poison) => 0,
            _ => {
                return Err(WError::DataNotAllowed(
                    "var convert idx",
                    format!("{self:?}"),
                ));
            }
        };
        Ok(r)
    }

    pub fn convert_str_to_array_i8(data: &str) -> Self {
        let vs = data
            .chars()
            .map(|v| Self::Basic(TyBasic::Int(8), Some(ValBasic::I8(v as u8 as i8))))
            .collect();
        Self::Array(Ty::Basic(TyBasic::Int(8)), vs)
    }
}

impl GlobalVar {
    pub fn generate(data: Var, id: &mut IDGenerater) -> Self {
        let link_age = id.generate(ID_STATE_ATTR, ATTR_LINK_AGE_PRIVATE);
        let visibility = id.generate(ID_STATE_ATTR, ATTR_VISIBILITY_DEFAULT);
        let dll_storage_class = id.generate(ID_STATE_ATTR, ATTR_DLL_STORAGE_CLASS_DEFAULT);
        let thread_local = id.generate(ID_STATE_ATTR, ATTR_THREAD_LOCAL_NOTTHREADLOCAL);
        let unnamed_addr = id.generate(ID_STATE_ATTR, ATTR_UNNAMED_ADDR_GLOBAL);
        Self {
            is_const: true,
            align: Some(1),
            link_age,
            visibility,
            dll_storage_class,
            thread_local,
            unnamed_addr,
            var: data,
        }
    }
}


macro_rules! enum_res {
    // 1. res
    // 2. return res
    // 3. return none
    ($($name1:ident,)*; $($name2:ident,)*; $($name3:ident,)*) => {
        impl InstrCalc {
            pub fn res(&self) -> Option<ID> {
                let r = match self {
                    $(
                        Self::$name1(v) => v.res.clone(),
                    )*
                    $(
                        Self::$name2(v) => return v.res.clone(),
                    )*
                    $(
                        Self::$name3(_) => return None,
                    )*
                };
                Some(r)
            }
        }

    };
}

enum_res! {
    Alloca, BinOp, Cast, Cmp, ExtractElt, Gep,
    InsertElt, Load, ShuffleVec, VSelect, LoadAtomic, Cmpxchg,
    ExtractVal, InsertVal, LandingPad, Freeze, AtomicRMW, UnOp,
    ;
    Call,
    ;
    Store, StoreAtomic, Fence,
}


impl InstrPhi {
    pub fn renew(&self, state: &IDState, id: &mut IDGenerater) -> WResult<Self> {
        let res = self.res.renew(state, id)?;
        let srcs = self
            .srcs
            .iter()
            .map(|src| {
                let var = src.var.renew(state, id)?;
                let block = src.block.renew(state, id)?;
                Ok::<_, WError>(InstrPhiSrc { block, var })
            })
            .collect::<WResult<Vec<_>>>()?;
        Ok(Self { res, srcs })
    }
}

impl FunctionProto {
    pub fn generate(ret: Arg, args: Vec<Arg>, id: &mut IDGenerater) -> Self {
        let conv = id.generate(ID_STATE_ATTR, ATTR_CALLING_CONV_CCC);
        let link_age = id.generate(ID_STATE_ATTR, ATTR_LINK_AGE_EXTERNAL);
        let visibility = id.generate(ID_STATE_ATTR, ATTR_VISIBILITY_DEFAULT);
        let dll_storage_class = id.generate(ID_STATE_ATTR, ATTR_DLL_STORAGE_CLASS_DEFAULT);
        let unnamed_addr = id.generate(ID_STATE_ATTR, ATTR_UNNAMED_ADDR_NONE);

        Self {
            ret,
            args,
            conv,
            link_age,
            visibility,
            dll_storage_class,
            unnamed_addr,
            attr: None,
            personality: None,
        }
    }

    pub fn renew(&self, state: &IDState, id: &mut IDGenerater) -> WResult<Self> {
        let ret = self.ret.renew(state, id)?;
        let args = self
            .args
            .iter()
            .map(|v| v.renew(state, id))
            .collect::<WResult<Vec<_>>>()?;
        let conv = self.conv.renew(state, id)?;
        let link_age = self.link_age.renew(state, id)?;
        let visibility = self.visibility.renew(state, id)?;
        let dll_storage_class = self.dll_storage_class.renew(state, id)?;
        let unnamed_addr = self.unnamed_addr.renew(state, id)?;
        let personality = if let Some(v) = &self.personality {
            Some(v.renew(state, id)?)
        } else {
            None
        };
        Ok(Self {
            ret,
            args,
            conv,
            link_age,
            visibility,
            dll_storage_class,
            unnamed_addr,
            attr: self.attr,
            personality
        })
    }
}

impl FunctionNative {
    pub fn generate(
        func_name: &str,
        ret: Arg,
        args: Vec<Arg>,
        inps: Vec<&str>,
        id: &mut IDGenerater,
    ) -> WResult<(ID, Self)> {
        let name = id.generate(ID_STATE_FUNC, func_name);
        if inps.len() != args.len() {
            return Err(WError::DataNotAllowed(
                "func arg/inp num",
                format!("{}/{}", args.len(), inps.len()),
            ));
        }
        let inps = inps
            .into_iter()
            .map(|n| id.generate(&format!("var_{}", func_name), n))
            .collect::<Vec<_>>();
        let mut vars = HashMap::with_capacity(128);
        for (name, inp) in inps.clone().into_iter().zip(args.iter()) {
            let val = Var::try_from(inp.ty.clone())?;
            vars.insert(name, val);
        }
        let base = FunctionProto::generate(ret, args, id);
        let blocks = Vec::with_capacity(4);
        Ok((
            name,
            Self {
                base,
                inps,
                vars,
                blocks,
            },
        ))
    }

    pub fn generate_var(
        &mut self,
        func_name: &str,
        var_name: &str,
        ty: Ty,
        id: &mut IDGenerater,
    ) -> WResult<ID> {
        let res = id.generate_with_name(ID_TAG_VAR, func_name, var_name);
        let var = Var::try_from(ty)?;
        self.vars.insert(res.clone(), var);
        Ok(res)
    }

    pub fn generate_const(&mut self, var: Var, id: &mut IDGenerater) -> ID {
        let res = id.generate_const(format!("{}", var));
        self.vars.insert(res.clone(), var);
        res
    }
}

impl From<InstrCalc> for InstrWithMeta<InstrCalc> {
    fn from(instr: InstrCalc) -> Self {
        Self {
            instr,
            metas: Vec::with_capacity(0),
        }
    }
}

impl InstrCalcCall {
    pub fn generate(
        res: Option<ID>,
        inps: Vec<ID>,
        func_name: ID,
        func_proto: &FunctionProto,
        id: &mut IDGenerater,
    ) -> WResult<Self> {
        if inps.len() != func_proto.args.len() {
            return Err(WError::DataNotAllowed(
                "instr call arg/inp num",
                format!("{}/{}", func_proto.args.len(), inps.len()),
            ));
        }
        let tail = id.generate(ID_STATE_ATTR, ATTR_CALL_TAIL_NONE);
        let conv = id.generate(ID_STATE_ATTR, ATTR_CALLING_CONV_CCC);
        Ok(Self {
            func: InstrFunc::Func(func_name),
            attr: None,
            ret: func_proto.ret.clone(),
            args: func_proto.args.clone(),
            tail: tail.clone(),
            conv: conv.clone(),
            res,
            inps,
            fmf: None
        })
    }
}

impl InstrTermSwitchCase {
    pub fn renew(&self, state: &IDState, id: &mut IDGenerater) -> WResult<Self> {
        let val = self.val.clone();
        let block = self.block.renew(state, id)?;
        Ok(Self { val, block })
    }
}


impl LLVM {
    pub fn save_file_ll<P: AsRef<Path>>(&self, p: P) -> WResult<()> {
        let mut f = File::create(p)?;
        f.write_all(self.to_string().as_bytes())?;
        Ok(())
    }
}

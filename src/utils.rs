//! utils

use std::{cmp::Ordering, collections::HashMap, fmt::{Debug, Display, Error}, fs::File, hash::Hash, io::Write, path::Path, rc::Rc, sync::RwLock};

use crate::{LLVM, WError, WResult, meta::{Arg, FunctionNative, FunctionProto, GlobalVar, ID, IDGenerater, InstrCalc, InstrCalcCall, InstrWithMeta, Name, Ty, TyBasic, ValBasic, Var}};


pub const ID_STATE_GLOBAL_VAR: &str = "global_var";
pub const ID_TAG_CONST: &str = "const";
pub const ID_STATE_ATTR: &str = "attr";
pub const ID_STATE_META: &str = "meta";
pub const ID_STATE_META_KIND: &str = "meta_kind";
pub const ID_STATE_FUNC: &str = "func";
pub const ID_TAG_BLOCK: &str = "block";
pub const ID_TAG_VAR: &str = "var";


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

pub const ATTR_UNNAMED_ADDR_NOT: &str = "";
pub const ATTR_UNNAMED_ADDR_IS: &str = "unnamed_addr";
pub const ATTR_UNNAMED_ADDR_LOCAL: &str = "local_unnamed_addr";

pub const ATTR_CALL_TAIL_NONE: &str = "";
pub const ATTR_CALL_TAIL_TAIL: &str = "tail";
pub const ATTR_CALL_TAIL_MUST: &str = "musttail";
pub const ATTR_CALL_TAIL_NO: &str = "notail";

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




impl Default for IDGenerater {
    fn default() -> Self {
        Self {
            idx: 1,
            idx_var_const: 0,
            record: HashMap::with_capacity(2048)
        }
    }
}

impl AsRef<HashMap<String, Name>> for IDGenerater {
    fn as_ref(&self) -> &HashMap<String, Name> {
        &self.record
    }
}

impl IDGenerater {
    /// 生成一个id（最基础的生成id函数）
    /// 使用现有str id生成一个id name
    /// 内部id使用给定str id
    /// 如果重复直接返回已有id name
    pub fn generate_with_id(&mut self, id: String, data: &str) -> Name {
        if let Some(name_old) = self.record.get(&id) {
            return name_old.clone();
        }
        let res = Rc::new(ID {
            id: self.idx,
            name: RwLock::new(data.to_string()),
        });
        self.record.insert(id, res.clone());
        self.idx += 1;
        res
    }


    /// 生成常量id name
    /// 内部str id为ID_TAG_CONST + _ + value
    pub fn generate_const(&mut self, value: String) -> Name {
        let id = format!("{}_{}", ID_TAG_CONST, value);
        if let Some(name) = self.record.get(&id) {
            return name.clone()
        }

        let r = format!("Const_{}", self.idx_var_const);
        self.idx_var_const += 1;
        // 使用值当作name，相同的const就会被合并
        self.generate_with_id(
            id,
            &r
        )
    }

    /// 生成一个id
    /// 将state和data融合为内部str id
    /// 然后调用generate_with_id
    pub fn generate(&mut self, state: &str, data: &str) -> Name {
        let id = format!("{}_{}", state, data);
        self.generate_with_id(id, data)
    }

    /// 生成一个id
    /// 带有名字，不同名字用来分割不同作用域
    /// 比如名字是function name
    /// 将tag和name融合为state，然后调用generate
    pub fn generate_with_name(&mut self, tag: &'static str, name: &str, data: &str) -> Name {
        let tag = format!("{}_{}", tag, name);
        self.generate(&tag, data)
    }

    /// 清空记录
    pub fn clear(&mut self) {
        self.record.clear();
    }

    /// 从内部取出一个id（最基础的取出id的函数）
    /// 使用给定str id取出id
    pub fn get_with_id(&self, id: &str) -> WResult<Name> {
        self.record
            .get(id)
            .cloned()
            .ok_or(WError::DataNotAllowed("id get", id.to_string()))
    }
    
    /// 从内部取出一个id
    /// 将state和data融合为内部str id
    /// 然后调用get_with_id
    pub fn get(&self, state: &str, data: &str) -> WResult<Name> {
        let id = format!("{}_{}", state, data);
        self.get_with_id(&id)
    }
    
    /// 从内部取出一个id
    /// 带有名字，不同名字用来分割不同作用域
    /// 比如名字是function name
    /// 将tag和name融合为state，然后调用get
    pub fn get_with_name(&self, tag: &'static str, name: &str, data: &str) -> WResult<Name> {
        let tag = format!("{}_{}", tag, name);
        self.get(&tag, data)
    }
}

impl PartialEq for ID {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ID {}

impl PartialOrd for ID {
    fn ge(&self, other: &Self) -> bool {
        self.id >= other.id
    }

    fn gt(&self, other: &Self) -> bool {
        self.id > other.id
    }

    fn le(&self, other: &Self) -> bool {
        self.id <= other.id
    }

    fn lt(&self, other: &Self) -> bool {
        self.id < other.id
    }

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ID {
    fn min(self, other: Self) -> Self {
        if self.id < other.id { self } else { other }
    }

    fn max(self, other: Self) -> Self {
        if self.id > other.id { self } else { other }
    }

    fn cmp(&self, other: &Self) -> Ordering {
        if self.id > other.id {
            Ordering::Greater
        } else if self.id == other.id {
            Ordering::Equal
        } else {
            Ordering::Less
        }
    }

    fn clamp(self, min: Self, max: Self) -> Self {
        if self.id < min.id {
            min
        } else if self.id > max.id {
            max
        } else {
            self
        }
    }
}

impl Hash for ID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.id);
    }
}

impl Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.name.read().as_deref() {
            Ok(v) => write!(f, "{}", v),
            Err(_) => Err(Error),
        }
    }
}

impl Debug for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self, self.id)
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

impl From<TyBasic> for Ty {
    fn from(value: TyBasic) -> Self {
        Self::Basic(value)
    }
}

impl From<Ty> for Var {
    fn from(ty: Ty) -> Self {
        match ty {
            Ty::Basic(v) => Self::Basic(v, None),
            Ty::Pointer(Some(v)) => Self::Pointer(Some(Box::new(Var::from(v.as_ref().clone())))),
            Ty::Pointer(None) => Self::Pointer(None),
            Ty::Array(v, n) => Self::Array((0..n).map(|_| Var::from(v.as_ref().clone())).collect()),
            Ty::Vector(v, n) => {
                Self::Vector((0..n).map(|_| Var::from(v.as_ref().clone())).collect())
            }
            Ty::Struct(vs) => Self::Vector(vs.into_iter().map(Var::from).collect()),
            Ty::MetaData => Self::MetaData(None),
            Ty::Blob => Self::Blob(Vec::with_capacity(0))
        }
    }
}

impl Var {
    pub fn ty(&self) -> Ty {
        match self {
            Self::Basic(v, _) => Ty::Basic(v.clone()),
            Self::Pointer(Some(v)) => Ty::Pointer(Some(Box::new(v.ty()))),
            Self::Pointer(None) => Ty::Pointer(None),
            Self::Array(vs) => {
                let ty = vs[0].ty();
                let n = vs.len();
                Ty::Array(Box::new(ty), n)
            }
            Self::Vector(vs) => {
                let ty = vs[0].ty();
                let n = vs.len();
                Ty::Vector(Box::new(ty), n)
            }
            Self::Struct(vs) => {
                let tys = vs.iter().map(|v| v.ty()).collect::<Vec<_>>();
                Ty::Struct(tys)
            }
            Self::MetaData(_) => Ty::MetaData,
            Self::Poison(v) => v.clone(),
            Self::Blob(_) => Ty::Blob,
        }
    }

    pub fn is_const(&self) -> bool {
        match self {
            Self::Basic(_, v) => v.is_some(),
            Self::Pointer(Some(v)) => v.is_const(),
            Self::Pointer(None) => false,
            Self::Array(vs) | Self::Vector(vs) | Self::Struct(vs) => {
                vs.iter().any(|v| v.is_const())
            }
            Self::MetaData(_) | Self::Poison(_) | Self::Blob(_) => true,
        }
    }
    
    pub fn convert_str_to_array_i8(data: &str) -> Self {
        let vs = data
            .chars()
            .map(| v | Self::Basic(
                TyBasic::Int(8),
                Some(ValBasic::I8(v as u8))
            ))
            .collect();
        Self::Array(vs)
    }
}

impl GlobalVar {
    pub fn generate(data: Var, id: &mut IDGenerater) -> Self {
        let link_age = id.generate(
            ID_STATE_ATTR,
            ATTR_LINK_AGE_PRIVATE,
        );
        let visibility = id.generate(
            ID_STATE_ATTR,
            ATTR_VISIBILITY_DEFAULT,
        );
        let dll_storage_class = id.generate(
            ID_STATE_ATTR,
            ATTR_DLL_STORAGE_CLASS_DEFAULT,
        );
        let thread_local = id.generate(
            ID_STATE_ATTR,
            ATTR_THREAD_LOCAL_NOTTHREADLOCAL,
        );
        let unnamed_addr = id.generate(
            ID_STATE_ATTR,
            ATTR_UNNAMED_ADDR_IS,
        );
        Self {
            is_const: true,
            align: 1,
            link_age,
            visibility,
            dll_storage_class,
            thread_local,
            unnamed_addr,
            var: data
        }
    }
}

impl InstrCalc {
    pub fn res(&self) -> Option<Name> {
        let r = match self {
            Self::Alloca(v) => v.res.clone(),
            Self::BinOp(v) => v.res.clone(),
            Self::Call(v) => return v.res.clone(),
            Self::Cast(v) => v.res.clone(),
            Self::Cmp(v) => v.res.clone(),
            Self::ExtractElt(v) => v.res.clone(),
            Self::Gep(v) => v.res.clone(),
            Self::InsertElt(v) => v.res.clone(),
            Self::Load(v) => v.res.clone(),
            Self::ShuffleVec(v) => v.res.clone(),
            Self::Store(_) => return None,
            Self::VSelect(v) => v.res.clone(),
        };
        Some(r)
    }
}

impl FunctionProto {
    pub fn generate(ret: Arg, args: Vec<Arg>, id: &mut IDGenerater) -> Self {
        let link_age = id.generate(
            ID_STATE_ATTR,
            ATTR_LINK_AGE_EXTERNAL,
        );
        let visibility = id.generate(
            ID_STATE_ATTR,
            ATTR_VISIBILITY_DEFAULT,
        );
        let dll_storage_class = id.generate(
            ID_STATE_ATTR,
            ATTR_DLL_STORAGE_CLASS_DEFAULT,
        );
        let unnamed_addr = id.generate(
            ID_STATE_ATTR,
            ATTR_UNNAMED_ADDR_NOT,
        );

        Self {
            ret,
            args,
            link_age,
            visibility,
            dll_storage_class,
            unnamed_addr,
            attr: None
        }
    }
}


impl FunctionNative {
    pub fn generate(
        func_name: &str,
        ret: Arg,
        args: Vec<Arg>,
        inps: Vec<&str>,
        id: &mut IDGenerater
    ) -> WResult<(Name, Self)> {
        let name = id.generate(
            ID_STATE_FUNC,
            func_name,
        );
        if inps.len() != args.len() {
            return Err(WError::DataNotAllowed(
                "func arg/inp num",
                format!("{}/{}", args.len(), inps.len())
            ))
        }
        let inps = inps
            .into_iter()
            .map(| n | id.generate(
                &format!("var_{}", func_name),
                n,
            ))
            .collect::<Vec<_>>();
        let mut vars = HashMap::with_capacity(128);
        for (name, inp) in inps.clone().into_iter().zip(args.iter()) {
            let val = Var::from(inp.ty.clone());
            vars.insert(name, val);
        }
        let base = FunctionProto::generate(ret, args, id);
        let blocks = Vec::with_capacity(4);
        Ok((name, Self {
            base,
            inps,
            vars,
            blocks
        }))
    }

    
    pub fn generate_var(&mut self, func_name: &str, var_name: &str, ty: Ty, id: &mut IDGenerater) -> Name {
        let res = id.generate_with_name(
            ID_TAG_VAR,
            func_name,
            var_name
        );
        let var = Var::from(ty);
        self.vars.insert(res.clone(), var);
        res
    }
    
    
    pub fn generate_const(&mut self, var: Var, id: &mut IDGenerater) -> Name {
        let res = id.generate_const(format!("{}", var));
        self.vars.insert(res.clone(), var);
        res
    }
}


impl From<InstrCalc> for InstrWithMeta<InstrCalc> {
    fn from(instr: InstrCalc) -> Self {
        Self {
            instr,
            metas: Vec::with_capacity(0)
        }
    }
}


impl InstrCalcCall {
    pub fn generate(res: Option<Name>, inps: Vec<Name>, func_name: Name, func_proto: &FunctionProto, id: &mut IDGenerater) -> WResult<Self> {
        if inps.len() != func_proto.args.len() {
            return Err(WError::DataNotAllowed(
                "instr call arg/inp num",
                format!("{}/{}", func_proto.args.len(), inps.len())
            ))
        }
        let tail = id.generate(
            ID_STATE_ATTR,
            ATTR_CALL_TAIL_NONE
        );
        let conv = id.generate(
            ID_STATE_ATTR,
            ATTR_CALLING_CONV_CCC
        );
        Ok(Self {
            func: func_name,
            attr: None,
            ret: func_proto.ret.clone(),
            args: func_proto.args.clone(),
            tail: tail.clone(),
            conv: conv.clone(),
            res,
            inps
        })
    }
}


impl LLVM {
    pub fn save_file_ll<P: AsRef<Path>>(&self, p: P) -> WResult<()> {
        let mut f = File::create(p)?;
        f.write_all(self.to_string().as_bytes())?;
        Ok(())
    }
}



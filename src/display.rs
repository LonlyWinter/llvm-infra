//! Meta Display

use std::{collections::HashMap, fmt::{Debug, Display}};

use crate::{LLVM, meta::{Arg, FunctionNative, FunctionProto, Name, Ty, TyBasic, ValBasic, Var}};


impl Display for TyBasic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Label => write!(f, "label"),
            Self::Blob => write!(f, "blob"),
            Self::Void => write!(f, "void"),
            Self::Float => write!(f, "float"),
            Self::Double => write!(f, "double"),
            Self::Half => write!(f, "half"),
            Self::Int(n) => write!(f, "i{n}"),
        }
    }
}

impl Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basic(v) => write!(f, "{}", v.as_ref()),
            Self::Pointer(v) => write!(f, "ptr {}", v.as_ref()),
            Self::Array(v, num) => write!(f, "[{} x {}]", num, v.as_ref()),
            Self::Vector(v, num) => write!(f, "<{} x {}>", num, v.as_ref()),
            Self::Struct(vs) => {
                let t = vs
                    .iter()
                    .map(| v | format!("{v}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{{{}}}", t)
            },
            Self::MetaData => write!(f, "metadata"),
        }
    }
}

impl Display for Arg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ty)
    }
}

impl Display for ValBasic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Poison => write!(f, "poison"),
            Self::Label(v) => write!(f, "label {}", v),
            Self::Blob(v) => {
                let v = v.iter().map(| v | *v as char).collect::<String>();
                write!(f, "{:?}", v)
            },
            Self::Bool(v) => write!(f, "{}", v),
            Self::I8(v) => write!(f, "{}", *v as i8),
            Self::I16(v) => write!(f, "{}", *v as i16),
            Self::I32(v) => write!(f, "{}", *v as i32),
            Self::I64(v) => write!(f, "{}", *v as i64),
            Self::I128(v) => write!(f, "{}", *v as i128),
            Self::F32(v) => write!(f, "{}", v),
            Self::F64(v) => write!(f, "{}", v),
        }
    }
}


impl Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basic(ty, Some(ValBasic::Null)) if matches!(
                ty.as_ref(),
                TyBasic::Int(_) | TyBasic::Float | TyBasic::Double | TyBasic::Half
            ) => write!(
                f,
                "{} 0",
                ty,
            ),
            Self::Basic(ty, data) => write!(
                f,
                "{}{}",
                ty,
                data.as_ref().map(| v | format!(" {v}")).unwrap_or_default()
            ),
            Self::Pointer(v) => write!(
                f,
                "ptr {}",
                v.as_ref()
            ),
            Self::Array(vs) | Self::Vector(vs) | Self::Struct(vs) => {
                let (v1, v2) = if let Self::Array(_) = self {
                    ("[", "]")
                } else if let Self::Vector(_) = self {
                    ("<", ">")
                } else {
                    ("{", "}")
                };
                let t = if self.is_const() {
                    let t = vs
                        .iter()
                        .map(| v | format!("{v}"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!(" {}{}{}", v1, t, v2)
                } else {
                    String::with_capacity(0)
                };
                let ty_base = vs[0].ty();
                write!(f, "{}{} x {}{}{}", v1, vs.len(), ty_base, v2, t)
            },
            Self::MetaData(v) => write!(
                f,
                "metadata{}",
                v.map(| v | format!(" {v}")).unwrap_or_default()
            ),
            Self::Poison(ty) => write!(
                f,
                "{} poison",
                ty.as_ref()
            ),
        }
    }
}


impl Display for FunctionProto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n  Ret: {}\n  Inps: {}",
            self.ret,
            self.inps.iter().map(| v | format!("{v}")).collect::<Vec<_>>().join(", ")
        )
    }
}


fn parse_instrs<T: Debug>(instrs: &[T]) -> String {
    instrs
        .iter()
        .map(|v| format!("\n    {v:?}"))
        .collect()
}


impl Display for FunctionNative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inps = self.inps
            .iter()
            .map(| v | format!("{v}"))
            .collect::<Vec<_>>()
            .join(", ");
        let mut vars = self
            .vars
            .iter()
            .map(|(name, ty)| format!("\n    {} = {}", name, ty))
            .collect::<Vec<_>>();
        vars.sort();
        let vars = vars
            .into_iter()
            .collect::<String>();
        let blocks = self.blocks
            .iter()
            .map(|block| {
                let instrs_phi = parse_instrs(&block.instrs_phi);
                let instrs_calc = parse_instrs(&block.instrs_calc);
                format!(
                    "\n  {}: {}{}{}\n    {:?}",
                    block.name,
                    block.instrs_calc.len(),
                    instrs_phi,
                    instrs_calc,
                    block.instr_term
                )
            })
            .collect::<String>();
        write!(
            f,
            "\n  Ret: {}\n  Inps: [{}]\n  Vars:{}{}",
            self.base.ret,
            inps,
            vars,
            blocks
        )
    }
}

fn parse_funcs<T: Display>(tag: &'static str, funcs: &HashMap<Name, T>) -> String {
    let mut funcs = funcs
        .iter()
        .collect::<Vec<_>>();
    funcs.sort_by_key(| v | v.0);
    funcs
        .into_iter()
        .map(|(name, src)| format!("\n{} {}:{}", tag, name, src))
        .collect::<String>()
}


impl Display for LLVM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let funcs_proto = parse_funcs("FunctionProto", &self.funcs_proto);
        let funcs_native = parse_funcs("FunctionNative", &self.funcs_native);
        write!(f, "Version: {}{}{}", self.version, funcs_proto, funcs_native)
    }
}

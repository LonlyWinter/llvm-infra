//! LLVM Display, .ll format

use std::{collections::HashMap, fmt::Display};

use crate::{
    LLVM,
    bitcode::parse_data_to_string,
    meta::{
        Arg, GlobalVar, GlobalVars, InstrCalc, InstrCalcBinOPOp, InstrCalcCastOp, InstrCalcCmpOp, InstrMeta, InstrPhi, InstrTerm, InstrWithMeta, MetaData, Name, Ty, TyBasic, ValBasic, Var, Vars
    },
};

impl Display for TyBasic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Label => write!(f, "label"),
            Self::Void => write!(f, "void"),
            Self::Float => write!(f, "float"),
            Self::Double => write!(f, "double"),
            Self::Half => write!(f, "half"),
            Self::Int(n) => write!(f, "i{n}"),
        }
    }
}

fn parse_vec_join<T: Display>(data: &[T], gap: &'static str) -> String {
    data.iter()
        .map(|v| format!("{v}"))
        .collect::<Vec<_>>()
        .join(gap)
}

fn parse_vec_collect<T: Display>(data: &[T], prefix: &'static str) -> String {
    data.iter()
        .map(|v| format!("{prefix}{v}"))
        .collect::<String>()
}

impl Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basic(v) => write!(f, "{}", v),
            Self::Pointer(_) => write!(f, "ptr"),
            Self::Array(v, num) => write!(f, "[{} x {}]", num, v.as_ref()),
            Self::Vector(v, num) => write!(f, "<{} x {}>", num, v.as_ref()),
            Self::Struct(vs) => {
                let t = parse_vec_join(vs, ", ");
                write!(f, "{{{}}}", t)
            }
            Self::MetaData => write!(f, "metadata"),
            Self::Blob => write!(f, "blob"),
        }
    }
}

impl Display for Arg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.ty, parse_vec_collect(&self.attr, " "))
    }
}

impl Display for ValBasic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Poison => write!(f, "poison"),
            Self::Label(v) => write!(f, "label {}", v),
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

impl Var {
    fn display_val(&self) -> String {
        match self {
            Self::Basic(
                TyBasic::Int(_) | TyBasic::Float | TyBasic::Double | TyBasic::Half,
                Some(ValBasic::Null),
            ) => "0".to_owned(),
            Self::Basic(_, data) => data.as_ref().map(|v| format!("{v}")).unwrap_or_default(),
            Self::Pointer(Some(v)) => format!("{}", v.as_ref()),
            Self::Pointer(None) => String::with_capacity(0),
            Self::Array(vs) | Self::Vector(vs) if vs.iter().all(|v| v.eq(&vs[0])) => {
                let val = vs[0].to_string();
                if val.ends_with(" 0") {
                    return "zeroinitializer".to_owned();
                }
                format!("splat ({})", val)
            }
            Self::Array(vs) | Self::Vector(vs) | Self::Struct(vs) => {
                let (v1, v2) = if let Self::Array(_) = self {
                    ("[", "]")
                } else if let Self::Vector(_) = self {
                    ("<", ">")
                } else {
                    ("{", "}")
                };
                if self.is_const() {
                    let t = parse_vec_join(vs, ", ");
                    format!("{}{}{}", v1, t, v2)
                } else {
                    String::with_capacity(0)
                }
            }
            Self::MetaData(v) => v.as_ref().map(|v| format!("!{v}")).unwrap_or_default(),
            Self::Poison(_) => "poison".to_owned(),
            Self::Blob(v) => parse_data_to_string(v.iter().copied())
        }
    }
}

impl Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = self.display_val();
        match self {
            Self::Basic(ty, _) => write!(f, "{} {}", ty, val),
            Self::Pointer(_) => write!(f, "ptr {}", val),
            Self::Array(vs) | Self::Vector(vs) | Self::Struct(vs) => {
                let (v1, v2) = if let Self::Array(_) = self {
                    ("[", "]")
                } else if let Self::Vector(_) = self {
                    ("<", ">")
                } else {
                    ("{", "}")
                };
                let ty_base = vs[0].ty();
                write!(f, "{}{} x {}{} {}", v1, vs.len(), ty_base, v2, val)
            }
            Self::MetaData(_) => write!(f, "metadata {}", val),
            Self::Poison(ty) => write!(f, "{} {}", ty, val),
            Self::Blob(_) => write!(f, "{}", val),
        }
    }
}

fn display_var_with_val_ty(id: &Name, vars: &DisplayVarData) -> String {
    vars.vars
        .get(id)
        .map(|v| {
            if v.is_const() {
                format!("{v}")
            } else {
                format!("{} %{}", v.ty(), id)
            }
        })
        .or_else(|| if vars.vars_global.contains_key(id) {
            Some(format!("ptr @{}", id))
        } else { None })
        .unwrap_or("display_var_error".to_owned())
}

fn display_var_with_val(id: &Name, vars: &DisplayVarData) -> String {
    vars.vars
        .get(id)
        .map(|v| {
            if v.is_const() {
                v.display_val()
            } else {
                format!("%{}", id)
            }
        })
        .or_else(|| if vars.vars_global.contains_key(id) {
            Some(format!("ptr @{}", id))
        } else { None })
        .unwrap_or("display_var_error".to_owned())
}

fn display_var_ty(id: &Name, vars: &DisplayVarData) -> String {
    vars.vars
        .get(id)
        .map(|v| format!("{}", v.ty()))
        .unwrap_or("display_ty_error".to_owned())
}

fn display_var_ty_pointer(id: &Name, vars: &DisplayVarData) -> String {
    vars.vars
        .get(id)
        .map(|v| {
            if let Var::Pointer(Some(v)) = v {
                v.ty()
            } else {
                v.ty()
            }
        })
        .map(|v| format!("{v}"))
        .unwrap_or_else(|| "display_ty_pointer_error".to_string())
}

struct DisplayVarData<'a> {
    vars: &'a Vars,
    vars_global: &'a GlobalVars,
}

trait DisplayVar {
    fn display<'a>(&self, vars: &DisplayVarData<'a>) -> String;
}

impl DisplayVar for InstrPhi {
    fn display(&self, vars: &DisplayVarData) -> String {
        let srcs = self
            .srcs
            .iter()
            .map(|(block, var)| format!("[ {}, %{} ]", display_var_with_val(var, vars), block))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "%{} = phi {} {}",
            self.res,
            display_var_ty(&self.res, vars),
            srcs
        )
    }
}

impl Display for InstrMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "!{} !{}", self.kind, self.name)
    }
}

impl Display for InstrCalcBinOPOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add(v1, v2) => write!(
                f,
                "add{}{}",
                if *v2 { " nuw" } else { "" },
                if *v1 { " nsw" } else { "" }
            ),
            Self::Sub(v1, v2) => write!(
                f,
                "sub{}{}",
                if *v2 { " nuw" } else { "" },
                if *v1 { " nsw" } else { "" }
            ),
            Self::Mul(v1, v2) => write!(
                f,
                "mul{}{}",
                if *v2 { " nuw" } else { "" },
                if *v1 { " nsw" } else { "" }
            ),
            Self::Shl(v1, v2) => write!(
                f,
                "shl{}{}",
                if *v2 { " nuw" } else { "" },
                if *v1 { " nsw" } else { "" }
            ),
            Self::FAdd => write!(f, "fadd"),
            Self::FSub => write!(f, "fsub"),
            Self::FMul => write!(f, "fmul"),
            Self::Urem => write!(f, "urem"),
            Self::Srem => write!(f, "srem"),
            Self::And => write!(f, "and"),
            Self::Xor => write!(f, "xor"),
            Self::Sdiv(v) => write!(f, "sdiv{}", if *v { " exact" } else { "" }),
            Self::Udiv(v) => write!(f, "udiv{}", if *v { " exact" } else { "" }),
            Self::Lshr(v) => write!(f, "lshr{}", if *v { " exact" } else { "" }),
            Self::Ashr(v) => write!(f, "ashr{}", if *v { " exact" } else { "" }),
            Self::Or(v) => write!(f, "or{}", if *v { " disjoint" } else { "" }),
            Self::Fdiv => write!(f, "fdiv"),
            Self::Frem => write!(f, "frem"),
        }
    }
}

impl Display for InstrCalcCastOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZExt(v) => write!(f, "zext{}", if *v { " nneg" } else { "" }),
            Self::UiToFp(v) => write!(f, "uitofp{}", if *v { " nneg" } else { "" }),
            Self::Trunc(v1, v2) => write!(
                f,
                "trunc{}{}",
                if *v2 { " nuw" } else { "" },
                if *v1 { " nsw" } else { "" }
            ),
            Self::SExt => write!(f, "sext"),
            Self::FpToUi => write!(f, "fptoui"),
            Self::FpToSi => write!(f, "fptosi"),
            Self::SiToFp => write!(f, "sitofp"),
            Self::PtrToInt => write!(f, "ptrtoint"),
            Self::IntToPtr => write!(f, "inttoptr"),
            Self::BitCast => write!(f, "bitcast"),
            Self::AddrSpaceCast => write!(f, "addrspacecast"),
            Self::PtrToAddr => write!(f, "ptrtoaddr"),
            Self::FpTrunc => write!(f, "fptrunc"),
            Self::FpExt => write!(f, "fpext"),
        }
    }
}

impl Display for InstrCalcCmpOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstrCalcCmpOp::Float(v) => write!(f, "fcmp {}", format!("{v:?}").to_lowercase()),
            InstrCalcCmpOp::Int(v, _) => write!(f, "icmp {}", format!("{v:?}").to_lowercase()),
        }
    }
}

impl DisplayVar for InstrCalc {
    fn display(&self, vars: &DisplayVarData) -> String {
        match self {
            InstrCalc::Alloca(instr) => format!(
                "%{} = alloca {}, align {}",
                instr.res,
                display_var_ty_pointer(&instr.res, vars),
                instr.align
            ),
            InstrCalc::BinOp(instr) => format!(
                "%{} = {} {}, {}",
                instr.res,
                instr.op,
                display_var_with_val_ty(&instr.left, vars),
                display_var_with_val(&instr.right, vars),
            ),
            InstrCalc::Call(instr) => {
                let args = instr
                    .inps
                    .iter()
                    .zip(instr.args.iter())
                    .map(|(v, g)| {
                        format!(
                            "{}{} {}",
                            display_var_ty(v, vars),
                            parse_vec_collect(&g.attr, " "),
                            display_var_with_val(v, vars)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let res = instr
                    .res
                    .as_ref()
                    .map(|v| format!("%{v} ="))
                    .unwrap_or_default();
                let tail = instr.tail.to_string();
                let head = parse_vec_join_str(vec![res, tail], " ");
                let tail = instr.attr.map(|v| format!(" #{v}")).unwrap_or_default();
                format!(
                    "{}{}call {} @{}({}){}",
                    head,
                    if head.is_empty() { "" } else { " " },
                    instr.ret,
                    instr.func,
                    args,
                    tail
                )
            }
            InstrCalc::Cast(instr) => format!(
                "%{} = {} {} %{} to {}",
                instr.res,
                instr.op,
                display_var_ty(&instr.val, vars),
                instr.val,
                display_var_ty(&instr.res, vars)
            ),
            InstrCalc::Cmp(instr) => format!(
                "%{} = {} {}, {}",
                instr.res,
                instr.op,
                display_var_with_val_ty(&instr.left, vars),
                display_var_with_val(&instr.right, vars)
            ),
            InstrCalc::ExtractElt(instr) => format!(
                "%{} = extractelement {}, {}",
                instr.res,
                display_var_with_val_ty(&instr.src, vars),
                display_var_with_val_ty(&instr.idx, vars),
            ),
            InstrCalc::Gep(instr) => {
                let mut no_warp_flags = String::with_capacity(18);
                if instr.in_bounds {
                    no_warp_flags.push_str("inbounds ");
                }
                if (!instr.in_bounds) && instr.nusw {
                    // in_bounds隐含nusw，存在in_bounds可以不输出nusw
                    no_warp_flags.push_str("nusw ");
                }
                if instr.nuw {
                    no_warp_flags.push_str("nuw ");
                }
                format!(
                    "%{} = getelementptr {}{}, {}, {}",
                    instr.res,
                    no_warp_flags,
                    display_var_ty_pointer(&instr.res, vars),
                    display_var_with_val_ty(&instr.base, vars),
                    display_var_with_val_ty(&instr.idx, vars),
                )
            }
            InstrCalc::InsertElt(instr) => format!(
                "%{} = insertelement {}, {}, {}",
                instr.res,
                display_var_with_val_ty(&instr.src, vars),
                display_var_with_val_ty(&instr.elt, vars),
                display_var_with_val_ty(&instr.idx, vars),
            ),
            InstrCalc::Load(instr) => format!(
                "%{} = load {}, {}, align {}",
                instr.res,
                display_var_ty(&instr.res, vars),
                display_var_with_val_ty(&instr.src, vars),
                instr.align
            ),
            InstrCalc::ShuffleVec(instr) => format!(
                "%{} = shufflevector {}, {}, {}",
                instr.res,
                display_var_with_val_ty(&instr.vec1, vars),
                display_var_with_val_ty(&instr.vec2, vars),
                display_var_with_val_ty(&instr.mask, vars),
            ),
            InstrCalc::Store(instr) => format!(
                "store {}, {}, align {}",
                display_var_with_val_ty(&instr.data, vars),
                display_var_with_val_ty(&instr.dst, vars),
                instr.align
            ),
            InstrCalc::VSelect(instr) => format!(
                "%{} = select {}, {}, {}",
                instr.res,
                display_var_with_val_ty(&instr.cond, vars),
                display_var_with_val_ty(&instr.val_true, vars),
                display_var_with_val_ty(&instr.val_false, vars),
            ),
        }
    }
}

impl<T: DisplayVar> DisplayVar for InstrWithMeta<T> {
    fn display(&self, vars: &DisplayVarData) -> String {
        format!(
            "{}{}{}",
            self.instr.display(vars),
            if self.metas.is_empty() { "" } else { ", " },
            parse_vec_join(&self.metas, ", ")
        )
    }
}

fn parse_instrs<T: DisplayVar>(instrs: &[T], vars: &DisplayVarData) -> String {
    instrs
        .iter()
        .map(|v| format!("\n  {}", v.display(vars)))
        .collect()
}

fn parse_vars<T: Display>(data: &HashMap<Name, T>, prefix: &'static str) -> String {
    let mut vars = data
        .iter()
        .map(|(name, ty)| format!("\n{}{} = {}", prefix, name, ty))
        .collect::<Vec<_>>();
    vars.sort();
    vars.into_iter().collect::<String>()
}

impl Display for MetaData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetaData::Value(v) => write!(f, "{v}"),
            MetaData::String(v) => write!(f, "!{v:?}"),
            MetaData::Meta(v) => write!(f, "!{v}"),
            MetaData::Nodes(dis, vs) => {
                let vs = parse_vec_join(vs, ", ");
                if *dis {
                    write!(f, "distinct !{{{}}}", vs)
                } else {
                    write!(f, "!{{{}}}", vs)
                }
            }
        }
    }
}

impl DisplayVar for InstrTerm {
    fn display(&self, vars: &DisplayVarData) -> String {
        match self {
            Self::Ret(None) => "ret void".to_owned(),
            Self::Ret(Some(id)) => format!("ret {}", display_var_with_val_ty(id, vars)),
            Self::CondBr(cond, label1, label2) => format!(
                "br {}, label %{}, label %{}",
                display_var_with_val_ty(cond, vars),
                label1,
                label2
            ),
            Self::DirectBr(label) => format!("br label %{}", label),
        }
    }
}

fn parse_vec_join_str(data: Vec<String>, gap: &'static str) -> String {
    data.into_iter()
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>()
        .join(gap)
}

impl Display for GlobalVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let attr_temp = parse_vec_join_str(
            vec![
                format!("{}", self.link_age),
                format!("{}", self.visibility),
                format!("{}", self.dll_storage_class),
                format!("{}", self.thread_local),
                format!("{}", self.unnamed_addr),
            ],
            " ",
        );
        write!(
            f,
            "{} {} {}, align {}",
            attr_temp,
            if self.is_const {
                "constant"
            } else {
                "global"
            },
            self.var,
            self.align
        )
    }
}

impl Display for LLVM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let metas = parse_vars(&self.metas, "!");

        let attrs = {
            let mut vs = self
                .attrs
                .iter()
                .map(|(k, v)| (
                    k,
                    v.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ")
                ))
                .collect::<Vec<_>>();
            vs.sort_by_key(|v| v.0);
            vs.into_iter()
                .map(|(k, v)| format!("\nattributes #{} = {{ {} }}", k, v))
                .collect::<String>()
        };

        // let mut attrs = Ha::with_capacity(16);

        let mut funcs = self.funcs_proto.iter().collect::<Vec<_>>();
        funcs.sort_by_key(|v| v.0);
        let funcs_proto = funcs
            .into_iter()
            .map(|(name, src)| {
                let tail = parse_vec_join_str(
                    vec![
                        format!("{}", src.link_age),
                        format!("{}", src.visibility),
                        format!("{}", src.dll_storage_class),
                        format!("{}", src.unnamed_addr),
                        src.attr.map(|v| format!("#{v}")).unwrap_or_default(),
                    ],
                    " ",
                );
                format!(
                    "\n\ndeclare{} {} @{}({}){}{}",
                    parse_vec_collect(&src.ret.attr, " "),
                    src.ret.ty,
                    name,
                    parse_vec_join(&src.args, ", "),
                    if tail.is_empty() { "" } else { " " },
                    tail,
                )
            })
            .collect::<String>();

        let mut funcs = self.funcs_native.iter().collect::<Vec<_>>();
        funcs.sort_by_key(|v| v.0);
        let funcs_native = funcs
            .into_iter()
            .map(|(name, func)| {
                let vars = DisplayVarData {
                    vars: &func.vars,
                    vars_global: &self.vars_global,
                };
                //
                let inps = func
                    .inps
                    .iter()
                    .zip(func.base.args.iter())
                    .map(|(name, ty)| format!("{ty} %{name}"))
                    .collect::<Vec<_>>();
                let inps = parse_vec_join(&inps, ", ");
                let blocks = func
                    .blocks
                    .iter()
                    .map(|block| {
                        let instrs_phi = parse_instrs(&block.instrs_phi, &vars);
                        let instrs_calc = parse_instrs(&block.instrs_calc, &vars);
                        format!(
                            "\n{}:{}{}\n  {}",
                            block.name,
                            instrs_phi,
                            instrs_calc,
                            block.instr_term.display(&vars)
                        )
                    })
                    .collect::<String>();

                let tail = parse_vec_join_str(
                    vec![
                        format!("{}", func.base.link_age),
                        format!("{}", func.base.visibility),
                        format!("{}", func.base.dll_storage_class),
                        format!("{}", func.base.unnamed_addr),
                        func.base.attr.map(|v| format!("#{v}")).unwrap_or_default(),
                    ],
                    " ",
                );
                format!(
                    "\n\ndefine{} {} @{}({}){}{} {{{}\n}}",
                    parse_vec_collect(&func.base.ret.attr, " "),
                    func.base.ret.ty,
                    name,
                    inps,
                    if tail.is_empty() { "" } else { " " },
                    tail,
                    blocks
                )
            })
            .collect::<String>();
        write!(
            f,
            "; Version = \"{}\"\nsource_filename = \"{}\"\ntarget datalayout = \"{}\"\ntarget triple = \"{}\"\n{}\n{}\n{}\n{}\n{}",
            self.version,
            self.source_filename,
            self.data_layout,
            self.triple,
            parse_vars(&self.vars_global, "@"),
            funcs_native,
            funcs_proto,
            attrs,
            metas,
        )
    }
}

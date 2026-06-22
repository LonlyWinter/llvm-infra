//! LLVM Display, .ll format

use std::{collections::HashMap, fmt::Display};

use crate::{
    LLVM,
    meta::{
        Arg, GlobalVar, GlobalVars, ID, InstrCalc, InstrCalcBinOPOp, InstrCalcCastOp,
        InstrCalcCmpOp, InstrFunc, InstrMeta, InstrPhi, InstrTerm, InstrTermRet,
        InstrTermSwitchValue, InstrWithMeta, MetaData, Ty, TyBasic, ValBasic, Var, Vars
    },
    utils::{
        ATTR_LINK_AGE_AVAILABLEEXTERNALLY, ATTR_LINK_AGE_EXTERNAL, ATTR_LINK_AGE_EXTERNWEAK
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
            Self::Struct(vs, p) => {
                let (v1, v2) = if *p {
                    ("<{", "}>")
                } else {
                    ("{", "}")
                };
                let t = parse_vec_join(vs, ", ");
                write!(f, "{} {} {}", v1, t, v2)
            },
            Self::MetaData => write!(f, "metadata"),
        }
    }
}

impl Arg {
    fn display_ty_last(&self) -> String {
        let head = parse_vec_join(&self.attr, " ");
        format!(
            "{}{}{}",
            head,
            if head.is_empty() { String::with_capacity(0) } else { " ".to_owned() },
            self.ty
        )
    }
}

impl Display for Arg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.ty, parse_vec_collect(&self.attr, " "))
    }
}

fn is_exact_f32(x: f32) -> bool {
    let bits = x.to_bits();
    if bits == 0x00000000 || bits == 0x80000000 {
        return true;
    }
    let exp = (bits >> 23) & 0xFF;
    let frac = bits & 0x007F_FFFF;

    match exp {
        0 | 255 => false, // zero / subnormal / NaN / Inf
        _ => {
            let mantissa = frac | 0x0080_0000; // 隐含 1
            mantissa.is_power_of_two()
        }
    }
}

impl Display for ValBasic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Poison => write!(f, "poison"),
            Self::Label(v) => write!(f, "label {}", v),
            Self::Bool(v) => write!(f, "{}", v),
            Self::I8(v) => write!(f, "{}", v),
            Self::I16(v) => write!(f, "{}", v),
            Self::I32(v) => write!(f, "{}", v),
            Self::I64(v) => write!(f, "{}", v),
            Self::I128(v) => write!(f, "{}", v),
            Self::F16(v) => write!(f, "0x{:04X}", v),
            Self::F32(v) => {
                if is_exact_f32(*v) {
                    write!(f, "{:.8e}", v)
                } else {
                    write!(f, "0x{:016X}", (*v as f64).to_bits())
                }
            },
            Self::F64(v) => write!(f, "0x{:016X}", v.to_bits()),
        }
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
            Self::Fdiv(_) => write!(f, "fdiv"),
            Self::Frem(_) => write!(f, "frem"),
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
            Self::FpTrunc(_) => write!(f, "fptrunc"),
            Self::FpExt(_) => write!(f, "fpext"),
        }
    }
}


impl Var {
    fn display_val(&self) -> String {
        match self {
            Self::Basic(_, data) => data.as_ref()
                .map(|v| format!("{v}"))
                .unwrap_or_default(),
            Self::Pointer(Some(v)) => format!("{}", v.as_ref()),
            Self::Pointer(None) => String::with_capacity(0),
            Self::Array(_, vs) | Self::Vector(_, vs)
                if !vs.is_empty() && vs[0].is_const() && vs.iter().all(|v| v.eq(&vs[0])) =>
            {
                let val = vs[0].to_string();
                if val.ends_with(" 0") {
                    return "zeroinitializer".to_owned();
                }
                format!("splat ({})", val)
            }
            Self::Array(_, vs) | Self::Vector(_, vs) | Self::Struct(_, vs) => {
                let (v1, v2) = if let Self::Array(_, _) = self {
                    ("[", "]")
                } else if let Self::Struct(p, _) = self {
                    if *p {
                        ("<{ ", " }>")
                    } else {
                        ("{ ", " }")
                    }
                } else {
                    ("<", ">")
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
            Self::String(n, v) => format!(
                "c\"{}{}\"",
                v.iter()
                    .map(| vv | match vv {
                            b'\n' => "\\0A".to_owned(),
                            b'\r' => "\\0D".to_owned(),
                            b'\t' => "\\09".to_owned(),
                            b'\0' => "\\00".to_owned(),
                            b'\\' => "\\\\".to_owned(),
                            0x20..=0x7e if vv != &b'"' => format!("{}", *vv as char),
                            i => format!("\\{:02X}", i),
                        })
                    .collect::<String>(),
                if *n { "\\00" } else { "" }
            ),
            Self::Null(Ty::Basic(TyBasic::Int(1))) => "false".to_owned(),
            Self::Null(Ty::Basic(TyBasic::Int(_))) => "0".to_owned(),
            Self::Null(Ty::Basic(
                TyBasic::Float | TyBasic::Double | TyBasic::Half
            )) => "0.0".to_owned(),
            Self::Null(Ty::Array(_, _) | Ty::Vector(_, _)) => "zeroinitializer".to_owned(),
            Self::Null(_) => "null".to_owned(),
            Self::Undef(_) => "undef".to_owned(),
            Self::InstrCast(instr) => format!(
                "{} ({} to {})",
                instr.op,
                instr.val,
                instr.ty,
            ),
            Self::InstrGEP(instr) => {
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
                    "getelementptr {}({}, {}, {})",
                    no_warp_flags,
                    instr.ty,
                    instr.base,
                    instr.idx
                )
            },
            Self::InstrBinOP(instr) => format!(
                "{} ({}, {})",
                instr.op,
                instr.left,
                instr.right,
            ),
            Self::Ptr(func) => format!(
                "@{}",
                func
            )
        }
    }

    fn display_ty_pointer(&self) -> String {
        let ty = self.ty();
        if let Ty::Pointer(Some(v)) = ty {
            v.to_string()
        } else {
            ty.to_string()
        }
    }
}

impl Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = self.display_val();
        match self {
            Self::Basic(ty, _) => write!(f, "{} {}", ty, val),
            Self::Pointer(_) => write!(f, "ptr {}", val),
            Self::Array(ty_base, vs) | Self::Vector(ty_base, vs) => {
                let (v1, v2) = if let Self::Array(_, _) = self {
                    ("[", "]")
                } else {
                    ("<", ">")
                };
                write!(f, "{}{} x {}{} {}", v1, vs.len(), ty_base, v2, val)
            },
            Self::Struct(p, vs) => {
                let tys = vs.iter()
                    .map(| v | format!("{}", v.ty()))
                    .collect::<Vec<_>>()
                    .join(", ");
                let (v1, v2) = if *p {
                    ("<{", "}>")
                } else {
                    ("{", "}")
                };
                write!(f, "{}{}{} {}", v1, tys, v2, val)
            },
            Self::MetaData(_) => write!(f, "metadata {}", val),
            Self::Poison(ty) => write!(f, "{} {}", ty, val),
            Self::String(b, v) => write!(f, "[{} x i8] {}", v.len() + (*b as usize), val),
            Self::Null(v) | Self::Undef(v) => write!(f, "{} {}", v, val),
            Self::InstrCast(i) => write!(f, "{} {}", i.ty, val),
            Self::InstrGEP(_) => write!(f, "ptr {}", val),
            Self::InstrBinOP(i) => write!(f, "{} {}", i.ty, val),
            Self::Ptr(_) => write!(f, "ptr {}", val),
        }
    }
}

fn display_var_with_val_ty(id: &ID, vars: &DisplayVarData) -> String {
    vars.vars
        .get(id)
        .map(|v| {
            if v.is_const() {
                format!("{v}")
            } else {
                format!("{} %{}", v.ty(), id)
            }
        })
        .or_else(|| {
            if vars.vars_global.contains_key(id) {
                Some(format!("ptr @{}", id))
            } else {
                None
            }
        })
        .unwrap_or_else(|| format!("display_error_var_{}", id))
}

fn display_var_with_val(id: &ID, vars: &DisplayVarData) -> String {
    vars.vars
        .get(id)
        .map(|v| {
            if v.is_const() {
                v.display_val()
            } else {
                format!("%{}", id)
            }
        })
        .or_else(|| {
            if vars.vars_global.contains_key(id) {
                Some(format!("@{}", id))
            } else {
                None
            }
        })
        .unwrap_or_else(|| format!("display_error_val_{}", id))
}

fn display_var_ty(id: &ID, vars: &DisplayVarData) -> String {
    vars.vars
        .get(id)
        .map(|v| format!("{}", v.ty()))
        .or_else(|| {
            if vars.vars_global.contains_key(id) {
                Some("ptr".to_owned())
            } else {
                None
            }
        })
        .unwrap_or_else(|| format!("display_error_ty_{}", id))
}

fn display_var_ty_pointer(id: &ID, vars: &DisplayVarData) -> String {
    vars.vars
        .get(id)
        .map(|v| v.display_ty_pointer())
        .unwrap_or_else(|| format!("display_error_ty_pointer_{}", id))
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
            .map(|src| {
                format!(
                    "[ {}, %{} ]",
                    display_var_with_val(&src.var, vars),
                    src.block
                )
            })
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

impl Display for InstrCalcCmpOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstrCalcCmpOp::Float(v, _) => write!(f, "fcmp {}", format!("{v:?}").to_lowercase()),
            InstrCalcCmpOp::Int(v, b) => write!(
                f, "icmp{} {}",
                if *b { " samesign" } else { "" },
                format!("{v:?}").to_lowercase()
            ),
        }
    }
}



impl Display for InstrFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Func(v) => write!(f, "@{}", v),
            Self::VirtualFunc(v) => write!(f, "%{}", v),
            Self::Asm(asm) => write!(
                f, "asm {}{}{}\"{}\", \"{}\"",
                if asm.has_side_effects {
                    "sideeffect "
                } else {
                    ""
                },
                asm.asm_dialect,
                if asm.is_align_stack {
                    "alignstack "
                } else {
                    ""
                },
                asm.asm,
                asm.constr
            )
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
                let args = instr.inps.iter()
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
                let res = instr.res.as_ref()
                    .map(|v| format!("%{v} ="))
                    .unwrap_or_default();
                let tail = instr.tail.to_string();
                let conv = instr.conv.to_string();
                let head = parse_vec_join_str(vec![res, tail, "call".to_owned(), conv], " ");
                let tail = instr.attr.map(|v| format!(" #{v}")).unwrap_or_default();
                format!(
                    "{} {} {}({}){}",
                    head,
                    instr.ret.display_ty_last(),
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
            InstrCalc::LoadAtomic(instr) => format!(
                "%{} = load atomic {}, {} {}, align {}",
                instr.res,
                display_var_ty(&instr.res, vars),
                display_var_with_val_ty(&instr.src, vars),
                instr.ordering,
                instr.align
            ),
            InstrCalc::StoreAtomic(instr) => format!(
                "store atomic {}, {} {}, align {}",
                display_var_with_val_ty(&instr.data, vars),
                display_var_with_val_ty(&instr.dst, vars),
                instr.ordering,
                instr.align
            ),
            InstrCalc::Cmpxchg(instr) => format!(
                "%{} = cmpxchg {}{}, {}, {} {} {}, align {}",
                instr.res,
                if instr.is_weak { "weak " } else { "" },
                display_var_with_val_ty(&instr.ptr, vars),
                display_var_with_val_ty(&instr.cmp, vars),
                display_var_with_val_ty(&instr.val, vars),
                instr.success_ordering,
                instr.failure_ordering,
                instr.align
            ),
            InstrCalc::ExtractVal(instr) => format!(
                "%{} = extractvalue {},{}",
                instr.res,
                display_var_with_val_ty(&instr.src, vars),
                parse_vec_collect(&instr.idxs, " "),
            ),
            InstrCalc::InsertVal(instr) => format!(
                "%{} = insertvalue {}, {},{}",
                instr.res,
                display_var_with_val_ty(&instr.src, vars),
                display_var_with_val_ty(&instr.val, vars),
                parse_vec_collect(&instr.idxs, " "),
            ),
            InstrCalc::Fence(instr) => format!(
                "fence {}",
                instr.ordering,
            ),
            InstrCalc::LandingPad(instr) => {
                let clean = if instr.is_cleanup {
                    vec!["\n          cleanup".to_owned()]
                } else {
                    Vec::with_capacity(0)
                };
                let tail = instr.clauses.iter()
                    .map(| clause | format!(
                        "\n          {} {}",
                        clause.ty,
                        display_var_with_val_ty(&clause.val, vars)
                    ))
                    .chain(clean)
                    .collect::<String>();
                format!(
                    "%{} = landingpad {}{}",
                    instr.res,
                    display_var_ty(&instr.res, vars),
                    tail
                )
            },
            InstrCalc::Freeze(instr) => format!(
                "%{} = freeze {}",
                instr.res,
                display_var_with_val_ty(&instr.op, vars),
            ),
            InstrCalc::AtomicRMW(instr) => format!(
                "%{} = atomicrmw {} {}, {} {}, align {}",
                instr.res,
                instr.op,
                display_var_with_val_ty(&instr.ptr, vars),
                display_var_with_val_ty(&instr.val, vars),
                instr.ordering,
                instr.align
            ),
            InstrCalc::UnOp(instr) => format!(
                "%{} = {} {}",
                instr.res,
                instr.op,
                display_var_with_val_ty(&instr.val, vars),
            )
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

fn parse_vars<T: Display>(data: &HashMap<ID, T>, prefix: &'static str) -> String {
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
            MetaData::Func(v) => write!(f, "ptr @{v}"),
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

impl Display for InstrTermSwitchValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int8(w, v) => write!(f, "i{} {}", w, v),
            Self::Int16(w, v) => write!(f, "i{} {}", w, v),
            Self::Int32(w, v) => write!(f, "i{} {}", w, v),
            Self::Int64(w, v) => write!(f, "i{} {}", w, v),
            Self::Int128(w, v) => write!(f, "i{} {}", w, v),
        }
    }
}

impl DisplayVar for InstrTerm {
    fn display(&self, vars: &DisplayVarData) -> String {
        match self {
            Self::Ret(InstrTermRet { var: None }) => "ret void".to_owned(),
            Self::Ret(InstrTermRet { var: Some(id) }) => format!(
                "ret {}",
                display_var_with_val_ty(id, vars)
            ),
            Self::CondBr(instr) => format!(
                "br {}, label %{}, label %{}",
                display_var_with_val_ty(&instr.cond, vars),
                instr.block_true,
                instr.block_false
            ),
            Self::DirectBr(instr) => format!("br label %{}", instr.block),
            Self::Invoke(instr) => {
                let args = instr.inps.iter()
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
                let res = instr.res.as_ref()
                    .map(|v| format!("%{v} ="))
                    .unwrap_or_default();
                let conv = instr.conv.to_string();
                let head = parse_vec_join_str(vec![res, "invoke".to_owned(), conv], " ");
                let tail = instr.attr.map(|v| format!(" #{v}")).unwrap_or_default();
                format!(
                    "{} {} {}({}){}\n          to label %{} unwind label %{}",
                    head,
                    instr.ret.display_ty_last(),
                    instr.func,
                    args,
                    tail,
                    instr.block_normal,
                    instr.block_unwind
                )
            },
            Self::Resume(instr) => format!(
                "resume {}",
                display_var_with_val_ty(&instr.var, vars),
            ),
            Self::Switch(instr) => {
                let cases = instr.cases.iter()
                    .map(| case | format!(
                        "\n    {}, label %{}",
                        case.val,
                        case.block
                    ))
                    .collect::<String>();
                format!(
                    "switch {}, label %{} [{}\n  ]",
                    display_var_with_val_ty(&instr.cond, vars),
                    instr.block_default,
                    cases
                )
            },
            Self::Unreachable => "unreachable".to_owned(),
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
        let tail = if let Ok(true) = self.link_age.with_data(| v | matches!(
            v, ATTR_LINK_AGE_EXTERNAL
                | ATTR_LINK_AGE_EXTERNWEAK
                | ATTR_LINK_AGE_AVAILABLEEXTERNALLY)) {
            self.var.display_ty_pointer()
        } else {
            format!(
                "{} {}{}",
                self.var.display_ty_pointer(),
                self.var.display_val(),
                self.align.as_ref().map(| v | format!(", align {v}")).unwrap_or_default()
            )
        };
        write!(
            f,
            "{} {} {}",
            attr_temp,
            if self.is_const { "constant" } else { "global" },
            tail
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
                .map(|(k, v)| {
                    (
                        k,
                        v.iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join(" "),
                    )
                })
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
                let head = src.link_age.to_string();
                let tail = parse_vec_join_str(
                    vec![
                        format!("{}", src.visibility),
                        format!("{}", src.dll_storage_class),
                        format!("{}", src.unnamed_addr),
                        src.attr.map(|v| format!("#{v}")).unwrap_or_default(),
                    ],
                    " ",
                );
                format!(
                    "\n\ndeclare{}{}{} {} @{}({}){}{}",
                    if head.is_empty() { "" } else { " " },
                    head,
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
                let inps = func.inps.iter()
                    .zip(func.base.args.iter())
                    .map(|(name, ty)| format!("{ty} %{name}"))
                    .collect::<Vec<_>>();
                let inps = parse_vec_join(&inps, ", ");
                let blocks = func.blocks.iter()
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

                let head = parse_vec_join_str(
                    vec![
                        format!("{}", func.base.link_age),
                        format!("{}", func.base.conv),
                    ],
                    " ",
                );
                let tail = parse_vec_join_str(
                    vec![
                        format!("{}", func.base.visibility),
                        format!("{}", func.base.dll_storage_class),
                        format!("{}", func.base.unnamed_addr),
                        func.base.attr
                            .map(|v| format!("#{v}"))
                            .unwrap_or_default(),
                        func.base.personality.as_ref()
                            .map(| v | format!("personality ptr @{}", v))
                            .unwrap_or_default(),
                    ],
                    " ",
                );
                format!(
                    "\n\ndefine{}{}{} {} @{}({}){}{} {{{}\n}}",
                    if head.is_empty() { "" } else { " " },
                    head,
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

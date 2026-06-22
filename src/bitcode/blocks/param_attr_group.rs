//! ParamAttrGroup
//! https://llvm.org/docs/BitCodeFormat.html#paramattr-group-block-contents

use std::{
    collections::{HashMap, hash_map::Entry},
    fmt::Display,
    io::{Read, Seek},
    rc::Rc,
    vec::IntoIter,
};

use crate::{
    WError, WResult,
    bitcode::{blocks::blockinfo::BlockInfoAll, entry::BitCodeEntryRecord, parse::StateBlock},
    filebit::{decoded_signed_vbrs_u8, decoded_signed_vbrs_u16, decoded_signed_vbrs_u32, decoded_signed_vbrs_u64},
};

use crate::filebit::BitReader;

use crate::bitcode::blocks::typeblock::{TypeBlock, TypeBlockData};

const CAPTURE: u64 = 102;
const NO_CAPTURE: u64 = 11;

macro_rules! enum_attr {
    (
        $mod1:pat, $($key1:ident = $value1:pat = $display1:literal,)*
        ;;
        $($mod2:pat, $ty2:ty, $method2:ident, $key2:ident = $value2:pat,;)*
        ;;
        $(
            $mod3:pat,
            $ty3:ty,
            $method3:ident,
            $($key3:ident = $value3:pat = $display3:literal,)*
            "####"
            $($key4:ident = $value4:pat = $display4:literal,)*;
        )*
    ) => {
        #[derive(Debug, PartialEq)]
        pub(in crate::bitcode) enum ParamAttr {
            $(
                $key1,
            )*
            $(
                $key2($ty2),
            )*
            $(
                $(
                    $key3($ty3),
                )*
                $(
                    $key4($ty3),
                )*
            )*
        }

        impl ParamAttr {
            fn parse(kind: u64, key: u64, data: &mut IntoIter<u64>, data_types: &TypeBlock) -> WResult<Self> {
                match (key, kind) {
                    $(
                        ($value1, $mod1) => Ok(Self::$key1),
                    )*
                    $(
                        ($value2, $mod2) => {
                            let v = $method2(kind, key, data, data_types)?;
                            Ok(Self::$key2(v))
                        },
                    )*
                    $(
                        $(
                            ($value3, $mod3) => {
                                let v = $method3(kind, key, data, data_types)?;
                                Ok(Self::$key3(v))
                            },
                        )*
                        $(
                            ($value4, $mod3) => {
                                let v = $method3(kind, key, data, data_types)?;
                                Ok(Self::$key4(v))
                            },
                        )*
                    )*
                    _ => Err(WError::DataNotAllowed(
                        "ParamAttr",
                        format!(
                            "key {}, kind {}",
                            key,
                            kind
                        )
                    )),
                }
            }
        }

        impl Display for ParamAttr {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$key1 => write!(f, $display1),
                    )*
                    $(
                        Self::$key2(v) => write!(f, "{}", v),
                    )*
                    $(
                        $(
                            Self::$key3(v) => write!(f, "{} {}", $display3, v),
                        )*
                        $(
                            Self::$key4(v) => write!(f, "{}({})", $display4, v),
                        )*
                    )*
                }
            }
        }
    };
}

enum_parse! {
    pub(in crate::bitcode) CaptureComponents, u64;
    Debug, PartialEq,;
    None = 0 = "none",
    AddressIsNull = 1 = "address_is_null",
    Address = 3 = "address",
    ReadProvenance = 4 = "read_provenance",
    AddressAndReadProvenance = 7 = "address, read_provenance",
    Provenance = 12 = "provenance",
    All = 15 = "address, provenance",
}

enum_parse! {
    pub(in crate::bitcode) AllocFnKindType, usize;
    Debug, PartialEq,;
    Unknown = 0 = "unknown",
    Alloc = 1 = "alloc",
    Realloc = 2 = "realloc",
    Free = 3 = "free",
    Uninitialized = 4 = "uninitialized",
    Zeroed = 5 = "zeroed",
    Aligned = 6 = "aligned",
}

#[derive(Debug, PartialEq)]
pub(in crate::bitcode) struct AllocFnKind(Vec<AllocFnKindType>);

impl TryFrom<u64> for AllocFnKind {
    type Error = WError;
    fn try_from(data: u64) -> WResult<Self> {
        if data == 0 {
            return Ok(Self(vec![AllocFnKindType::Unknown]));
        }
        let cs = format!("{data:b}");
        let mut res = Vec::with_capacity(4);
        for (i, c) in cs.chars().rev().enumerate() {
            if c == '0' {
                continue;
            }
            let t = if i > 5 {
                AllocFnKindType::Aligned
            } else {
                AllocFnKindType::try_from(i + 1)?
            };
            res.push(t);
        }
        res.shrink_to_fit();
        Ok(Self(res))
    }
}

impl Display for AllocFnKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self.0.iter()
            .map(| v | v.to_string())
            .collect::<Vec<_>>()
            .join(",");
        write!(f, "allockind(\"{}\")", data)
    }
}

#[derive(Debug, PartialEq, Default)]
#[repr(u8)]
pub(in crate::bitcode) enum UwTableKind {
    None = 0,
    Sync = 1,
    #[default]
    Async = 2,
}

impl TryFrom<u64> for UwTableKind {
    type Error = WError;
    fn try_from(value: u64) -> WResult<Self> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Sync),
            2 => Ok(Self::Async),
            _ => Err(WError::CodeNotAllowed(
                "Not match for UwTableKind",
                value as u32,
            )),
        }
    }
}

impl Display for UwTableKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, ""),
            Self::Sync => write!(f, "uwtable(sync)"),
            Self::Async => write!(f, "uwtable"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(in crate::bitcode) struct CaptureInfo {
    pub(in crate::bitcode) other: CaptureComponents,
    pub(in crate::bitcode) ret: CaptureComponents,
}

impl Default for CaptureInfo {
    fn default() -> Self {
        Self {
            other: CaptureComponents::None,
            ret: CaptureComponents::None,
        }
    }
}

impl TryFrom<u64> for CaptureInfo {
    type Error = WError;
    fn try_from(value: u64) -> WResult<Self> {
        let other = CaptureComponents::try_from(value >> 4)?;
        let ret = CaptureComponents::try_from(value & 0xF)?;
        Ok(Self { other, ret })
    }
}

impl Display for CaptureInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.other, &self.ret) {
            (CaptureComponents::All, CaptureComponents::All) => write!(f, ""),
            (CaptureComponents::None, CaptureComponents::None) => write!(f, "captures(none)"),
            (other, ret) if other == ret => write!(f, "captures({})", other),
            (other, CaptureComponents::None) => write!(f, "captures({})", other),
            (CaptureComponents::None, ret) => write!(f, "captures(ret: {})", ret),
            (other, ret) => write!(f, "captures(ret: {}, {})", ret, other),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(in crate::bitcode) struct TypeInfo {
    data: Option<Rc<TypeBlockData>>,
}

impl Display for TypeBlockData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Array(v) => write!(f, "[{} x {}]", v.num, v.ty),
            Self::Vector(v) => write!(f, "<{} x {}>", v.num, v.ty),
            Self::Integer(v) => write!(f, "i{}", v),
            Self::OpaquePointer(_) => write!(f, "ptr"),
            Self::Label => write!(f, "label"),
            Self::Void => write!(f, "void"),
            Self::Float => write!(f, "float"),
            Self::Double => write!(f, "double"),
            Self::Half => write!(f, "half"),
            i => write!(f, "{:?}", i),
        }
    }
}

impl Display for TypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = &self.data {
            write!(f, "{}", v)
        } else {
            write!(f, "")
        }
    }
}

#[derive(Debug, PartialEq)]
pub(in crate::bitcode) struct RangeInfo {
    bit: u16,
    start: i64,
    end: i64,
}

impl Display for RangeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "range(i{} {}, {})", self.bit, self.start, self.end)
    }
}

#[derive(Debug, PartialEq)]
pub(in crate::bitcode) struct RangeList {
    bit: u16,
    // start, end
    data: Vec<(i64, i64)>,
}

impl Display for RangeList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "initializes({})",
            self.data
                .iter()
                .map(|(start, end)| format!("({}, {})", start, end))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[derive(Debug, PartialEq)]
pub(in crate::bitcode) struct StringWithValue {
    data: String,
    value: String,
}

impl Display for StringWithValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}={:?}", self.data, self.value)
    }
}

enum_parse! {
    pub(in crate::bitcode) MemoryEffectsType, u64;
    Debug, PartialEq, ;
    No = 0 = "",
    Read = 1 = "read",
    Write = 2 = "write",
    ReadWrite = 3 = "readwrite",
}

#[derive(Debug, PartialEq)]
pub(in crate::bitcode) struct MemoryEffects {
    // 普通内存, bit 0–1, 访问 globals / captured ptr
    default: MemoryEffectsType,
    // 参数内存, bit 2–3, 访问参数指针指向的内存
    argmem: MemoryEffectsType,
    // InaccessibleMem, bit 4–5, 访问模块不可见内存
    inaccessiblemem: MemoryEffectsType,
}

impl TryFrom<u64> for MemoryEffects {
    type Error = WError;
    fn try_from(data: u64) -> WResult<Self> {
        if (data >> 56) == 0 {
            let argmem = MemoryEffectsType::try_from(data & 3)?;
            let inaccessiblemem = MemoryEffectsType::try_from((data >> 2) & 3)?;
            let default = MemoryEffectsType::try_from((data >> 4) & 3)?;
            return Ok(Self {
                default,
                argmem,
                inaccessiblemem,
            });
        }
        MemoryEffects::try_from(data & 0x00FFFFFFFFFFFFFF)
    }
}

impl Display for MemoryEffects {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let default = ["".to_owned(), self.default.to_string()];
        let argmem = ["argmem: ".to_owned(), self.argmem.to_string()];
        let inaccessiblemem = [
            "inaccessiblemem: ".to_owned(),
            self.inaccessiblemem.to_string(),
        ];
        let res = [default, argmem, inaccessiblemem]
            .into_iter()
            .filter(|v| !v[1].is_empty())
            .map(|v| format!("{}{}", v[0], v[1]))
            .collect::<Vec<_>>()
            .join(", ");
        write!(
            f,
            "memory({}{})",
            if res.is_empty() { "none" } else { "" },
            res,
        )
    }
}

fn parse_attr_uwtable(
    kind: u64,
    _key: u64,
    data: &mut IntoIter<u64>,
    _data_types: &TypeBlock,
) -> WResult<UwTableKind> {
    if kind == 0 {
        Ok(UwTableKind::default())
    } else {
        let data = data
            .next()
            .ok_or(WError::DataNotFound("parase param attr group type uwtable"))?;
        UwTableKind::try_from(data)
    }
}

fn parse_alloc_fn_kind_attr(
    _kind: u64,
    _key: u64,
    data: &mut IntoIter<u64>,
    _data_types: &TypeBlock,
) -> WResult<AllocFnKind> {
    let data = data.next().ok_or(WError::DataNotFound(
        "parase param attr group constant_range",
    ))?;
    AllocFnKind::try_from(data)
}

#[derive(Debug, PartialEq)]
pub(in crate::bitcode) struct AllocSize {
    elt: u32,
    num: u32
}

impl Display for AllocSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.num == u32::MAX {
            write!(f, "allocsize({})", self.elt)
        } else {
            write!(f, "allocsize({}, {})", self.elt, self.num)
        }
    }
}

fn parse_allocsize_fn_kind_attr(
    _kind: u64,
    _key: u64,
    data: &mut IntoIter<u64>,
    _data_types: &TypeBlock
) -> WResult<AllocSize> {
    let data = data.next()
        .ok_or(WError::DataNotFound("parase param attr group int"))?;

    let elt = (data >> 32) as u32;
    let num = (data & 0xFFFFFFFF) as u32;
    Ok(AllocSize { elt, num })
}

fn parse_int_attr(
    _kind: u64,
    _key: u64,
    data: &mut IntoIter<u64>,
    _data_types: &TypeBlock,
) -> WResult<u64> {
    data.next()
        .ok_or(WError::DataNotFound("parase param attr group int"))
}

fn parse_type_attr(
    kind: u64,
    _key: u64,
    data: &mut IntoIter<u64>,
    data_types: &TypeBlock,
) -> WResult<TypeInfo> {
    let data = if kind == 6 {
        let kind = data
            .next()
            .ok_or(WError::DataNotFound("parase param attr group type"))?;

        let r = data_types
            .get(kind as usize)
            .cloned()
            .ok_or(WError::CodeNotAllowed(
                "parase param attr group type",
                kind as u32,
            ))?;
        Some(r)
    } else {
        None
    };
    Ok(TypeInfo { data })
}

fn parse_memory(
    _kind: u64,
    _key: u64,
    data: &mut IntoIter<u64>,
    _data_types: &TypeBlock,
) -> WResult<MemoryEffects> {
    let data = data
        .next()
        .ok_or(WError::DataNotFound("parase param attr group int"))?;
    MemoryEffects::try_from(data)
}

fn parse_string_with_value_attr(
    kind: u64,
    key: u64,
    data: &mut IntoIter<u64>,
    data_types: &TypeBlock,
) -> WResult<StringWithValue> {
    let t = key as u8 as char;
    let mut a1 = parse_string_attr(kind, key, data, data_types)?;
    let a2 = parse_string_attr(kind, key, data, data_types)?;
    a1.insert(0, t);
    Ok(StringWithValue {
        data: a1,
        value: a2,
    })
}

fn parse_string_attr(
    _kind: u64,
    _key: u64,
    data: &mut IntoIter<u64>,
    _data_types: &TypeBlock,
) -> WResult<String> {
    let mut res = String::with_capacity(32);
    while let Some(v) = data.next()
        && v != 0
    {
        res.push(v as u8 as char);
    }
    Ok(res)
}

fn parse_cauture_info_attr(
    kind: u64,
    key: u64,
    data: &mut IntoIter<u64>,
    _data_types: &TypeBlock,
) -> WResult<CaptureInfo> {
    if kind == 0 && key == NO_CAPTURE {
        return Ok(CaptureInfo::default());
    }
    if kind == 1 && key == CAPTURE {
        let data = data
            .next()
            .ok_or(WError::DataNotFound("parase param attr group capture_info"))?;
        return CaptureInfo::try_from(data);
    }
    Err(WError::DataNotAllowed(
        "capture_info error",
        format!("kind {}, key {}", kind, key),
    ))
}

fn parse_range_start_end(data: &mut IntoIter<u64>, bit: u16) -> WResult<(i64, i64)> {
    let start = data.next().ok_or(WError::DataNotFound(
        "data1 in parase param attr group constant_range_list",
    ))?;
    let end = data.next().ok_or(WError::DataNotFound(
        "data2 in parase param attr group constant_range_list",
    ))?;
    // eprintln!("Range: {}..{} {}", start, end, bit);
    let (start, end) = match bit {
        8 => (
            decoded_signed_vbrs_u8(start as u8) as i64,
            decoded_signed_vbrs_u8(end as u8) as i64,
        ),
        16 => (
            decoded_signed_vbrs_u16(start as u16) as i64,
            decoded_signed_vbrs_u16(end as u16) as i64,
        ),
        32 | 24 => (
            decoded_signed_vbrs_u32(start as u32) as i64,
            decoded_signed_vbrs_u32(end as u32) as i64,
        ),
        64 => (
            decoded_signed_vbrs_u64(start),
            decoded_signed_vbrs_u64(end)
        ),
        _ => return Err(WError::UnImplemented("range other bit 8/32/64")),
    };
    Ok((start, end))
}

fn parse_constant_range_list_attr(
    _kind: u64,
    _key: u64,
    data: &mut IntoIter<u64>,
    _data_types: &TypeBlock,
) -> WResult<RangeList> {
    let num = data.next().map(|v| v as usize).ok_or(WError::DataNotFound(
        "data1 in parase param attr group constant_range",
    ))?;
    let bit = data.next().map(|v| v as u16).ok_or(WError::DataNotFound(
        "data1 in parase param attr group constant_range",
    ))?;
    let res = (0..num)
        .map(|_| parse_range_start_end(data, bit))
        .collect::<WResult<Vec<_>>>()?;
    Ok(RangeList { bit, data: res })
}

fn parse_constant_range_attr(
    _kind: u64,
    _key: u64,
    data: &mut IntoIter<u64>,
    _data_types: &TypeBlock,
) -> WResult<RangeInfo> {
    let bit = data.next().map(|v| v as u16).ok_or(WError::DataNotFound(
        "data1 in parase param attr group constant_range",
    ))?;
    if bit > 64 {
        return Err(WError::UnImplemented("param_attr_group range bit > 64"));
    }
    let (start, end) = parse_range_start_end(data, bit)?;
    Ok(RangeInfo { bit, start, end })
}

enum_attr! {
    // kind enum_attr
    0,
    AllocAlign = 80 = "allocalign",
    AllocatedPointer = 81 = "allocptr",
    AlwaysInline = 2 = "alwaysinline",
    Builtin = 35 = "builtin",
    Cold = 36 = "cold",
    Convergent = 43 = "convergent",
    CoroOnlyDestroyWhenComplete = 90 = "coroonlydestroywhencomplete",
    CoroElideSafe = 98 = "coroelidesafe",
    DeadOnReturn = 103 = "dead_on_return",
    DeadOnUnwind = 91 = "dead_on_unwind",
    DisableSanitizerInstrumentation = 78 = "disablesanitizerinstrumentation",
    FnRetThunkExtern = 84 = "fnretthunkextern",
    Hot = 72 = "hot",
    HybridPatchable = 95 = "hybridpatchable",
    ImmArg = 60 = "immarg",
    InReg = 5 = "inreg",
    InlineHint = 4 = "inlinehint",
    JumpTable = 40 = "jumptable",
    MinSize = 6 = "minsize",
    MustProgress = 70 = "mustprogress",
    Naked = 7 = "naked",
    Nest = 8 = "nest",
    NoAlias = 9 = "noalias",
    NoBuiltin = 10 = "nobuiltin",
    NoCallback = 71 = "nocallback",
    NocfCheck = 56 = "nocfcheck",
    NoCreateUndefOrPoison = 105 = "nocreateundeforpoison",
    NoDivergenceSource = 100 = "nodivergencesource",
    NoDuplicate = 12 = "noduplicate",
    NoExt = 99 = "noext",
    NoFree = 62 = "nofree",
    NoImplicitFloat = 13 = "noimplicitfloat",
    NoInline = 14 = "noinline",
    NoMerge = 66 = "nomerge",
    NoProfile = 73 = "noprofile",
    NoRecurse = 48 = "norecurse",
    NoRedZone = 16 = "noredzone",
    NoReturn = 17 = "noreturn",
    NoSanitizeBounds = 79 = "nosanitizebounds",
    NoSanitizeCoverage = 76 = "nosanitizecoverage",
    NoSync = 63 = "nosync",
    NoUndef = 68 = "noundef",
    NoUnwind = 18 = "nounwind",
    NonLazyBind = 15 = "nonlazybind",
    NonNull = 39 = "nonnull",
    NullPointerIsValid = 67 = "nullpointerisvalid",
    OptForFuzzing = 57 = "optforfuzzing",
    OptimizeForDebugging = 88 = "optimizefordebugging",
    OptimizeForSize = 19 = "optsize",
    OptimizeNone = 37 = "optimizenone",
    PresplitCoroutine = 83 = "presplitcoroutine",
    ReadNone = 20 = "readnone",
    ReadOnly = 21 = "readonly",
    Returned = 22 = "returned",
    ReturnsTwice = 23 = "returnstwice",
    SExt = 24 = "sext",
    Safestack = 44 = "safestack",
    SanitizeAddress = 30 = "sanitizeaddress",
    SanitizeAllocToken = 104 = "sanitizealloctoken",
    SanitizeHwaddress = 55 = "sanitizehwaddress",
    SanitizeMemTag = 64 = "sanitizememtag",
    SanitizeMemory = 32 = "sanitizememory",
    SanitizeNumericalStability = 93 = "sanitizenumericalstability",
    SanitizeRealtime = 96 = "sanitizerealtime",
    SanitizeRealtimeBlocking = 97 = "sanitizerealtimeblocking",
    SanitizeThread = 31 = "sanitizethread",
    SanitizeType = 101 = "sanitizetype",
    ShadowCallStack = 58 = "shadowcallstack",
    SkipProfile = 85 = "skipprofile",
    Speculatable = 53 = "speculatable",
    SpeculativeLoadHardening = 59 = "speculativeloadhardening",
    StackProtect = 26 = "stackprotect",
    StackProtectReq = 27 = "stackprotectreq",
    StackProtectStrong = 28 = "stackprotectstrong",
    StrictFp = 54 = "strictfp",
    SwiftAsync = 75 = "swiftasync",
    SwiftError = 47 = "swifterror",
    SwiftSelf = 46 = "swiftself",
    WillReturn = 61 = "willreturn",
    Writable = 89 = "writable",
    WriteOnly = 52 = "writeonly",
    ZExt = 34 = "zeroext",
    ;;
    // kind specific
    0 | 1, UwTableKind, parse_attr_uwtable, UwTable = 33,;
    // kind int_attr specific: alloc fn kind
    1, AllocFnKind, parse_alloc_fn_kind_attr, AllocKind = 82,;
    1, AllocSize, parse_allocsize_fn_kind_attr, AllocSize = 51,;
    // kind constant_range_attr, 100
    7, RangeInfo, parse_constant_range_attr, Range = 92,;
    // kind constant_range_list_attr, 101
    8, RangeList, parse_constant_range_list_attr, Initializes = 94,;
    0 | 1, CaptureInfo, parse_cauture_info_attr, CaptureInfo = NO_CAPTURE | CAPTURE,;
    3, String, parse_string_attr, String = _,;
    4, StringWithValue, parse_string_with_value_attr, StringWithValue = _,;
    1, MemoryEffects, parse_memory, Memory = 86,;
    ;;
    // kind type_attr
    5 | 6, TypeInfo, parse_type_attr,
    "####"
    ByRef = 69 = "byref",
    ByVal = 3 = "byval",
    StructRet = 29 = "sret",
    InAlloca = 38 = "inalloca",
    Elementtype = 77 = "elementtype",
    Preallocated = 65 = "preallocated",;
    // kind int_attr, 89-99
    1, u64, parse_int_attr,
    Alignment = 1 = "align",
    "####"
    Dereferenceable = 41 = "dereferenceable",
    DereferenceableOrNull = 42 = "dereferenceable_or_null",
    NoFPClass = 87 = "nofpclass",
    StackAlignment = 25 = "stackalignment",
    VScaleRange = 74 = "vscalerange",;
}

pub(in crate::bitcode) type ParamAttrList = Vec<Rc<ParamAttr>>;
pub(in crate::bitcode) type ParamAttrUnit = HashMap<u32, ParamAttrList>;
/// (grpid, paramid) -> [attrs]
pub(in crate::bitcode) type ParamAttrGroupBlock = HashMap<usize, ParamAttrUnit>;

fn parse_param_attr_group(
    data: Vec<u64>,
    data_types: &TypeBlock,
) -> WResult<(usize, u32, ParamAttrList)> {
    let mut data = data.into_iter();
    let group_id = data
        .next()
        .ok_or(WError::DataNotFound("group id in parse param_attr_group"))?;
    let param_id = data
        .next()
        .ok_or(WError::DataNotFound("param id in parse param_attr_group"))?;
    let mut res = Vec::with_capacity(16);
    while let Some(kind) = data.next() {
        let key = data
            .next()
            .ok_or(WError::DataNotFound("key in parase param attr group"))?;
        // eprintln!("param_attr_group: kind {}, key {} ...", kind, key);
        let t = ParamAttr::parse(kind, key, &mut data, data_types)?;
        res.push(Rc::new(t));
    }
    res.shrink_to_fit();
    // eprintln!("Group id: {}", group_id);
    Ok((group_id as usize, param_id as u32, res))
}

impl<F: Read + Seek> BitReader<F> {
    pub(in crate::bitcode) fn parse_param_attr_group_block(
        &mut self,
        mut state: StateBlock,
        block_infos: &BlockInfoAll,
        data_types: &TypeBlock,
    ) -> WResult<ParamAttrGroupBlock> {
        // todo!("module block: {:?} {}", state.get_current(), reader.position()?);
        let mut res: ParamAttrGroupBlock = HashMap::with_capacity(16);
        let mut parse_param_attr_group_now = |data: BitCodeEntryRecord<u64>| {
            let (gid, pid, vs) = parse_param_attr_group(data.data, data_types)?;
            match res.entry(gid) {
                Entry::Occupied(mut v) => {
                    v.get_mut().insert(pid, vs);
                }
                Entry::Vacant(v) => {
                    let mut temp = HashMap::with_capacity(32);
                    temp.insert(pid, vs);
                    v.insert(temp);
                }
            }
            Ok(())
        };
        self.parse_entry_only_record(
            &mut state,
            block_infos,
            "ParamAttr Block",
            &mut parse_param_attr_group_now,
        )?;
        for v in res.values_mut() {
            v.shrink_to_fit();
        }
        res.shrink_to_fit();
        Ok(res)
    }
}

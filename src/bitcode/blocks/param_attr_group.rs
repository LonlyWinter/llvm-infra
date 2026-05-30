//! ParamAttrGroup
//! https://llvm.org/docs/BitCodeFormat.html#paramattr-group-block-contents

use std::{
    collections::HashMap,
    io::{Read, Seek},
    rc::Rc,
    vec::IntoIter,
};

use crate::{
    WError, WResult,
    bitcode::{blocks::blockinfo::BlockInfoAll, entry::BitCodeEntryRecord, parse::StateBlock},
};

use crate::bitreader::BitReader;

use crate::bitcode::blocks::typeblock::{TypeBlock, TypeBlockData};

const CAPTURE: u64 = 102;
const NO_CAPTURE: u64 = 11;
const FUNC_IDX: u64 = u32::MAX as u64;

macro_rules! enum_attr {
    (
        $mod1:pat, $($key1:ident = $value1:pat,)*
        ;;
        $($mod2:pat, $ty:ty, $method:ident, $func:item, $($key2:ident = $value2:pat,)*;)*
    ) => {
        #[derive(Debug, PartialEq)]
        pub(in crate::bitcode) enum ParamAttr {
            $(
                $key1,
            )*
            $($(
                $key2($ty),
            )*)*
            MemoryEffects(MemoryEffects)
        }

        impl ParamAttr {
            $(
                $func
            )*

            fn parse(kind: u64, key: u64, data: &mut IntoIter<u64>, data_types: &TypeBlock) -> WResult<Self> {
                match (key, kind) {
                    $(
                        ($value1, $mod1) => Ok(Self::$key1),
                    )*
                    $($(
                        ($value2, $mod2) => {
                            let v = Self::$method(kind, key, data, data_types)?;
                            Ok(Self::$key2(v))
                        },
                    )*)*
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
    };
}

enum_parse! {
    pub(in crate::bitcode) MemoryEffects, u64;
    Debug, PartialEq,;
    ReadNone = 20,
    ReadOnly = 21,
    WriteOnly = 52,
    ArgMemOnly = 45,
    InaccessiblememOnly = 49,
    InaccessiblememOrArgmemonly = 50,
}

enum_parse! {
    pub(in crate::bitcode) CaptureComponents, u64;
    Debug, PartialEq,;
    None = 0,
    AddressIsNull = 1,
    Address = 3,
    ReadProvenance = 4,
    Provenance = 12,
    All = 15,
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

enum_parse! {
    pub(in crate::bitcode) AllocFnKind, u64;
    Debug, PartialEq,;
    Unknown = 0,
    Alloc = 1,
    Realloc = 2,
    Free = 4,
    Uninitialized = 8,
    Zeroed = 16,
    Aligned = 32,
}

enum_attr! {
    // kind enum_attr
    0,
    AllocAlign = 80,
    AllocatedPointer = 81,
    AlwaysInline = 2,
    Builtin = 35,
    Cold = 36,
    Convergent = 43,
    CoroOnlyDestroyWhenComplete = 90,
    CoroElideSafe = 98,
    DeadOnReturn = 103,
    DeadOnUnwind = 91,
    DisableSanitizerInstrumentation = 78,
    FnRetThunkExtern = 84,
    Hot = 72,
    HybridPatchable = 95,
    ImmArg = 60,
    InReg = 5,
    InlineHint = 4,
    JumpTable = 40,
    MinSize = 6,
    MustProgress = 70,
    Naked = 7,
    Nest = 8,
    NoAlias = 9,
    NoBuiltin = 10,
    NoCallback = 71,
    NocfCheck = 56,
    NoCreateUndefOrPoison = 105,
    NoDivergenceSource = 100,
    NoDuplicate = 12,
    NoExt = 99,
    NoFree = 62,
    NoImplicitFloat = 13,
    NoInline = 14,
    NoMerge = 66,
    NoProfile = 73,
    NoRecurse = 48,
    NoRedZone = 16,
    NoReturn = 17,
    NoSanitizeBounds = 79,
    NoSanitizeCoverage = 76,
    NoSync = 63,
    NoUndef = 68,
    NoUnwind = 18,
    NonLazyBind = 15,
    NonNull = 39,
    NullPointerIsValid = 67,
    OptForFuzzing = 57,
    OptimizeForDebugging = 88,
    OptimizeForSize = 19,
    OptimizeNone = 37,
    PresplitCoroutine = 83,
    ReadNone = 20,
    ReadOnly = 21,
    Returned = 22,
    ReturnsTwice = 23,
    SExt = 24,
    Safestack = 44,
    SanitizeAddress = 30,
    SanitizeAllocToken = 104,
    SanitizeHwaddress = 55,
    SanitizeMemTag = 64,
    SanitizeMemory = 32,
    SanitizeNumericalStability = 93,
    SanitizeRealtime = 96,
    SanitizeRealtimeBlocking = 97,
    SanitizeThread = 31,
    SanitizeType = 101,
    ShadowCallStack = 58,
    SkipProfile = 85,
    Speculatable = 53,
    SpeculativeLoadHardening = 59,
    StackProtect = 26,
    StackProtectReq = 27,
    StackProtectStrong = 28,
    StrictFp = 54,
    SwiftAsync = 75,
    SwiftError = 47,
    SwiftSelf = 46,
    WillReturn = 61,
    Writable = 89,
    WriteOnly = 52,
    ZExt = 34,
    ;;
    // kind specific
    0 | 1, UwTableKind, parse_attr_uwtable,
    fn parse_attr_uwtable(kind: u64, _key: u64, data: &mut IntoIter<u64>, _data_types: &TypeBlock) -> WResult<UwTableKind> {
        if kind == 0 {
            Ok(UwTableKind::default())
        } else {
            let data = data
                .next()
                .ok_or(WError::DataNotFound("parase param attr group type uwtable"))?;
            UwTableKind::try_from(data)
        }
    },
    UwTable = 33,;
    // kind type_attr
    5 | 6, Option<Rc<TypeBlockData>>, parse_type_attr,
    fn parse_type_attr(kind: u64, _key: u64, data: &mut IntoIter<u64>, data_types: &TypeBlock) -> WResult<Option<Rc<TypeBlockData>>> {
        if kind == 6 {
            let kind = data
                .next()
                .ok_or(WError::DataNotFound("parase param attr group type"))?;

            let r = data_types
                .get(kind as usize)
                .cloned()
                .ok_or(WError::CodeNotAllowed("parase param attr group type", kind as u32))?;
            Ok(Some(r))
        } else {
            Ok(None)
        }
    },
    ByRef = 69,
    ByVal = 3,
    StructRet = 29,
    InAlloca = 38,
    Elementtype = 77,
    Preallocated = 65,;
    // kind int_attr, 89-99
    1, u64, parse_int_attr,
    fn parse_int_attr(_kind: u64, _key: u64, data: &mut IntoIter<u64>, _data_types: &TypeBlock) -> WResult<u64> {
        data
            .next()
            .ok_or(WError::DataNotFound("parase param attr group int"))
    },
    Alignment = 1,
    AllocSize = 51,
    Dereferenceable = 41,
    DereferenceableOrNull = 42,
    Memory = 86,
    NoFPClass = 87,
    StackAlignment = 25,
    VScaleRange = 74,;
    // kind int_attr specific: alloc fn kind
    1, AllocFnKind, parse_alloc_fn_kind_attr,
    fn parse_alloc_fn_kind_attr(_kind: u64, _key: u64, data: &mut IntoIter<u64>, _data_types: &TypeBlock) -> WResult<AllocFnKind> {
        let data = data
            .next()
            .ok_or(WError::DataNotFound("parase param attr group constant_range"))?;
        AllocFnKind::try_from(data)
    },
    AllocKind = 82,;
    // kind constant_range_attr, 100
    7, usize, parse_constant_range_attr,
    fn parse_constant_range_attr(_kind: u64, _key: u64, data: &mut IntoIter<u64>, _data_types: &TypeBlock) -> WResult<usize> {
        data
            .next()
            .map(| v | v as usize)
            .ok_or(WError::DataNotFound("parase param attr group constant_range"))
    },
    Range = 92,;
    // kind constant_range_list_attr, 101
    8, [usize; 2], parse_constant_range_list_attr,
    fn parse_constant_range_list_attr(_kind: u64, _key: u64, data: &mut IntoIter<u64>, _data_types: &TypeBlock) -> WResult<[usize; 2]> {
        let a1 = data
            .next()
            .map(| v | v as usize)
            .ok_or(WError::DataNotFound("data1 in parase param attr group constant_range_list"))?;
        let a2 = data
            .next()
            .map(| v | v as usize)
            .ok_or(WError::DataNotFound("data2 in parase param attr group constant_range_list"))?;
        Ok([a1, a2])
    },
    Initializes = 94,;
    0 | 1, CaptureInfo, parse_cauture_info_attr,
    fn parse_cauture_info_attr(kind: u64, key: u64, data: &mut IntoIter<u64>, _data_types: &TypeBlock) -> WResult<CaptureInfo> {
        if kind == 0 && key == NO_CAPTURE {
            return Ok(CaptureInfo::default())
        }
        if kind == 1 && key == CAPTURE {
            let data = data
                .next()
                .ok_or(WError::DataNotFound("parase param attr group capture_info"))?;
            return CaptureInfo::try_from(data)
        }
        Err(WError::DataNotAllowed("parse capture_info error", format!("kind {}, key {}", kind, key)))
    },
    CaptureInfo = NO_CAPTURE | CAPTURE,;
    3, String, parse_string_attr,
    fn parse_string_attr(_kind: u64, _key: u64, data: &mut IntoIter<u64>, _data_types: &TypeBlock) -> WResult<String> {
        let mut res = String::with_capacity(32);
        while let Some(v) = data.next() && v != 0 {
            res.push(v as u8 as char);
        }
        Ok(res)
    },
    String = _,;
    4, [String; 2], parse_string_with_value_attr,
    fn parse_string_with_value_attr(kind: u64, key: u64, data: &mut IntoIter<u64>, data_types: &TypeBlock) -> WResult<[String; 2]> {
        let t = key as u8 as char;
        let mut a1 = Self::parse_string_attr(kind, key, data, data_types)?;
        let a2 = Self::parse_string_attr(kind, key, data, data_types)?;
        a1.insert(0, t);
        Ok([a1, a2])
    },
    StringWithValue = _,;
}

pub(in crate::bitcode) type ParamAttrList = Vec<Rc<ParamAttr>>;
/// (grpid, paramid) -> [attrs]
pub(in crate::bitcode) type ParamAttrGroupBlock = HashMap<usize, ParamAttrList>;

fn parse_param_attr_group(
    data: Vec<u64>,
    data_types: &TypeBlock,
) -> WResult<(usize, ParamAttrList)> {
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
        if param_id == FUNC_IDX
            && let Ok(t) = MemoryEffects::try_from(key)
        {
            res.push(Rc::new(ParamAttr::MemoryEffects(t)));
            continue;
        }
        let t = ParamAttr::parse(kind, key, &mut data, data_types)?;
        res.push(Rc::new(t));
    }
    // eprintln!("Group id: {}", group_id);
    Ok((group_id as usize, res))
}

impl<F: Read + Seek> BitReader<F> {
    pub(in crate::bitcode) fn parse_param_attr_group_block(
        &mut self,
        mut state: StateBlock,
        block_infos: &BlockInfoAll,
        data_types: &TypeBlock,
    ) -> WResult<ParamAttrGroupBlock> {
        // todo!("parse module block: {:?} {}", state.get_current(), reader.position()?);
        let mut res = HashMap::with_capacity(16);
        let mut parse_param_attr_group_now = |data: BitCodeEntryRecord<u64>| {
            let (k, v) = parse_param_attr_group(data.data, data_types)?;
            res.insert(k, v);
            Ok(())
        };
        self.parse_entry_only_record(
            &mut state,
            block_infos,
            "ParamAttr Block",
            &mut parse_param_attr_group_now,
        )?;
        res.shrink_to_fit();
        // let mut group_ids = res.keys().collect::<Vec<_>>();
        // group_ids.sort();
        // eprintln!(
        //     "parse_param_attr_group_block: {:?} {:?}",
        //     res,
        //     group_ids,
        // );
        Ok(res)
    }
}

//! MetaDataKind Block

use std::collections::HashMap;
use std::io::{Read, Seek};

use crate::bitcode::blocks::blockinfo::BlockInfoAll;
use crate::bitcode::entry::BitCodeEntryRecord;
use crate::bitcode::parse::StateBlock;
use crate::bitcode::parse_data_to_string;
use crate::{WError, WResult};

use crate::filebit::BitReader;

const META_DATA_KIND: u32 = 6;

macro_rules! enum_metadata {
    (
        $($key:ident = $value1:expr, $value2:pat)*
    ) => {
        #[derive(Debug, PartialEq, Eq, Hash, Clone)]
        #[repr(u8)]
        pub(in crate::bitcode) enum MetaDataKindMap {
            $(
                $key,
            )*
            Unknown(String)
        }

        impl MetaDataKindMap {
            pub(in crate::bitcode) fn to_string(&self) -> String {
                match self {
                    $(
                        Self::$key => $value1.to_string(),
                    )*
                    Self::Unknown(v) => v.clone()
                }
            }
        }

        impl TryFrom<String> for MetaDataKindMap {
            type Error = WError;
            fn try_from(value: String) -> WResult<Self> {
                match value.as_str() {
                    $(
                        $value1 => Ok(Self::$key),
                    )*
                    _ => Ok(Self::Unknown(value))
                }
            }
        }

        impl TryFrom<usize> for MetaDataKindMap {
            type Error = WError;
            fn try_from(value: usize) -> WResult<Self> {
                match value {
                    $(
                        $value2 => Ok(Self::$key),
                    )*
                    _ => Err(WError::DataNotAllowed(
                        stringify!(MetaDataKindMap),
                        format!("{:?}", value)
                    ))
                }
            }
        }
    };
}

enum_metadata! {
    Dbg = "dbg", 0
    Tbaa = "tbaa", 1
    Prof = "prof", 2
    Fpmath = "fpmath", 3
    Range = "range", 4
    TbaaStruct = "tbaa.struct", 5
    InvariantLoad = "invariant.load", 6
    AliasScope = "alias.scope", 7
    Noalias = "noalias", 8
    Nontemporal = "nontemporal", 9
    MemParallelLoopAccess = "llvm.mem.parallel_loop_access", 10
    Nonnull = "nonnull", 11
    Dereferenceable = "dereferenceable", 12
    DereferenceableOrNull = "dereferenceable_or_null", 13
    MakeImplicit = "make.implicit", 14
    Unpredictable = "unpredictable", 15
    InvariantGroup = "invariant.group", 16
    Align = "align", 17
    Loop = "llvm.loop", 18
    Type = "type", 19
    SectionPrefix = "section_prefix", 20
    AbsoluteSymbol = "absolute_symbol", 21
    Associated = "associated", 22
    Callees = "callees", 23
    IrrLoop = "irr_loop", 24
    AccessGroup = "llvm.access.group", 25
    Callback = "callback", 26
    PreserveAccessIndex = "llvm.preserve.access.index", 27
    VcallVisibility = "vcall_visibility", 28
    Noundef = "noundef", 29
    Annotation = "annotation", 30
    Nosanitize = "nosanitize", 31
    FuncSanitize = "func_sanitize", 32
    Exclude = "exclude", 33
    Memprof = "memprof", 34
    Callsite = "callsite", 35
    KcfiType = "kcfi_type", 36
    Pcsections = "pcsections", 37
    DIAssignID = "DIAssignID", 38
    CoroOutsideFrame = "coro.outside.frame", 39
    Mmra = "mmra", 40
    NoaliasAddrspace = "noalias.addrspace", 41
    CalleeType = "callee_type", 42
    Nofree = "nofree", 43
    Captures = "captures", 44
    AllocToken = "alloc_token", 45
    ImplicitRef = "implicit.ref", 46
}

fn parse_meta_data_kind_block(code: u32, data: Vec<u16>) -> WResult<(usize, MetaDataKindMap)> {
    if code != META_DATA_KIND {
        return Err(WError::CodeNotAllowed("MetaDataKind Record", code));
    }
    if data.len() < 2 {
        return Err(WError::DataNotFound("MetaDataKind key not found"));
    }
    let id = data[0] as usize;
    let chars = parse_data_to_string(data.into_iter().skip(1).map(|v| v as u8));
    let kind = MetaDataKindMap::try_from(chars)?;
    // eprintln!("MetaDataKind: {} {}", kind, chars);
    Ok((id, kind))
}

pub(in crate::bitcode) type MetaDataKindBlock = HashMap<usize, MetaDataKindMap>;

impl<F: Read + Seek> BitReader<F> {
    pub(in crate::bitcode) fn parse_meta_data_kind_block(
        &mut self,
        mut state: StateBlock,
        block_infos: &BlockInfoAll,
    ) -> WResult<MetaDataKindBlock> {
        let mut res = HashMap::with_capacity(64);
        let mut parse_meta_data_kind_block_now = |data: BitCodeEntryRecord<u16>| {
            let (k, v) = parse_meta_data_kind_block(data.code, data.data)?;
            res.insert(k, v);
            Ok(())
        };
        self.parse_entry_only_record(
            &mut state,
            block_infos,
            "MetaDataKind Block",
            &mut parse_meta_data_kind_block_now,
        )?;
        res.shrink_to_fit();
        Ok(res)
    }
}

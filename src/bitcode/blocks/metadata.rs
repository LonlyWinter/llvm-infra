//! MetaData Block
//! https://llvm.org/docs/BitCodeFormat.html#metadata-block-contents

use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek};

use crate::bitcode::blocks::blockinfo::BlockInfoAll;
use crate::bitcode::blocks::moduleblock::{GlobalValueList, GlobalValueUnit};
use crate::bitcode::blocks::typeblock::TypeBlock;
use crate::bitcode::entry::BitCodeEntryRecord;
use crate::bitcode::parse::StateBlock;
use crate::bitreader::BitReaderFileVec;
use crate::{WError, WResult};

use crate::bitreader::BitReader;

// STRING_OLD = 1,     // MDSTRING:      [values]
// KIND = 6,           // [n x [id, name]]
// LOCATION = 7,       // [distinct, line, col, scope, inlined-at?]
// OLD_NODE = 8,       // OLD_NODE:      [n x (type num, value num)]
// OLD_FN_NODE = 9,    // OLD_FN_NODE:   [n x (type num, value num)]
// ATTACHMENT = 11,    // [m x [value, [n x [id, mdnode]]]
// GENERIC_DEBUG = 12, // [distinct, tag, vers, header, n x md num]
// SUBRANGE = 13,      // [distinct, count, lo]
// ENUMERATOR = 14,    // [isUnsigned|distinct, value, name]
// BASIC_TYPE = 15,    // [distinct, tag, name, size, align, enc]
// FILE = 16, // [distinct, filename, directory, checksumkind, checksum]
// DERIVED_TYPE = 17,       // [distinct, ...]
// COMPOSITE_TYPE = 18,     // [distinct, ...]
// SUBROUTINE_TYPE = 19,    // [distinct, flags, types, cc]
// COMPILE_UNIT = 20,       // [distinct, ...]
// SUBPROGRAM = 21,         // [distinct, ...]
// LEXICAL_BLOCK = 22,      // [distinct, scope, file, line, column]
// LEXICAL_BLOCK_FILE = 23, //[distinct, scope, file, discriminator]
// NAMESPACE = 24, // [distinct, scope, file, name, line, exportSymbols]
// TEMPLATE_TYPE = 25,   // [distinct, scope, name, type, ...]
// TEMPLATE_VALUE = 26,  // [distinct, scope, name, type, value, ...]
// GLOBAL_VAR = 27,      // [distinct, ...]
// LOCAL_VAR = 28,       // [distinct, ...]
// EXPRESSION = 29,      // [distinct, n x element]
// OBJC_PROPERTY = 30,   // [distinct, name, file, line, ...]
// IMPORTED_ENTITY = 31, // [distinct, tag, scope, entity, line, name]
// MODULE = 32,          // [distinct, scope, name, ...]
// MACRO = 33,           // [distinct, macinfo, line, name, value]
// MACRO_FILE = 34,      // [distinct, macinfo, line, file, ...]
// GLOBAL_DECL_ATTACHMENT = 36, // [valueid, n x [id, mdnode]]
// GLOBAL_VAR_EXPR = 37,        // [distinct, var, expr]
// INDEX_OFFSET = 38,           // [offset]
// INDEX = 39,                  // [bitpos]
// LABEL = 40,                  // [distinct, scope, name, file, line]
// STRING_TYPE = 41,            // [distinct, name, size, align,...]
// // Codes 42 and 43 are reserved for support for Fortran array specific debug info.
// COMMON_BLOCK = 44,     // [distinct, scope, name, variable,...]
// GENERIC_SUBRANGE = 45, // [distinct, count, lo, up, stride]
// ARG_LIST = 46,         // [n x [type num, value num]]
// ASSIGN_ID = 47,        // [distinct, ...]
// SUBRANGE_TYPE = 48,    // [distinct, ...]
// FIXED_POINT_TYPE = 49, // [distinct, ...]

fn parse_strings(data: Vec<usize>, mut blob: Vec<u8>) -> WResult<Vec<String>> {
    if data.len() != 2 {
        return Err(WError::DataNotFound("count/offset in parse strings"));
    }
    let num = data[0];
    let offset = data[1];
    let mut strings = blob.split_off(offset);
    let mut reader = BitReaderFileVec::from_vec(blob);
    let mut res = Vec::with_capacity(num);
    for _ in 0..num {
        let size = reader.variable_width_integers::<usize>(6)?;
        // eprintln!("strings size: {} {} {:?}", strings.len(), size, res);
        if strings.len() < size {
            return Err(WError::DataNotFound("strings"));
        }
        let strings_left = strings.split_off(size);
        let s = strings.into_iter().map(|v| v as char).collect();
        res.push(s);
        strings = strings_left;
    }
    Ok(res)
}

fn parse_value(
    data: Vec<usize>,
    data_types: &TypeBlock,
    value_list: &GlobalValueList,
) -> WResult<GlobalValueUnit> {
    if data.len() != 2 {
        return Err(WError::DataNotFound("type/value num in parse strings"));
    }
    let ty_value = data_types
        .get(data[0])
        .cloned()
        .ok_or(WError::DataNotFound("type in parse value"))?;
    let idx_value = data[1];
    let res = value_list.get(idx_value).cloned();

    let res = if let Some(value) = res {
        let ty = value.get_ty("metadata value ty")?;
        if ty != ty_value {
            return Err(WError::DataNotAllowed(
                "metadata value",
                format!("{:?} {:?}", ty_value, ty),
            ));
        }
        value
    } else {
        GlobalValueUnit::Future(idx_value, ty_value)
    };
    Ok(res)
}

#[derive(Debug, Clone, PartialEq)]
pub(in crate::bitcode) enum MetaDataUnit {
    Value(GlobalValueUnit),
    String(String),
    Node(Vec<usize>),
    Future(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub(in crate::bitcode) enum MetaDataValue {
    Value(GlobalValueUnit),
    String(String),
    Node(Vec<Self>),
    Already(usize, HashSet<usize>),
}

enum_parse! {
    MetaDatakCode, u32;
    ;
    Value = 2, // [type num, value num]
    Name = 4, // [values]
    Strings = 35, // [count, offset] blob([lengths][chars])
    Node = 3, // [n x md num]
    DistinctNode = 5, // [n x md num]
    NamedNode = 10, // [n x mdnodes]
}

#[derive(Debug, Clone)]
pub(in crate::bitcode) struct MetaData {
    data: Vec<MetaDataUnit>,
}

pub(in crate::bitcode) type NamedMetaData = HashMap<String, Vec<usize>>;

impl Default for MetaData {
    fn default() -> Self {
        Self {
            data: Vec::with_capacity(64),
        }
    }
}

impl MetaData {
    pub(in crate::bitcode) fn get_unit(&self, idx: usize) -> MetaDataUnit {
        self.data
            .get(idx - 1)
            .cloned()
            .unwrap_or(MetaDataUnit::Future(idx))
    }

    fn get_value_inner(&self, idx: usize, mut seen: HashSet<usize>) -> WResult<MetaDataValue> {
        let v = self.data.get(idx - 1).cloned();
        if let Some(v) = v {
            if seen.contains(&idx) {
                seen.shrink_to_fit();
                return Ok(MetaDataValue::Already(idx, seen));
            }
            seen.insert(idx);
            let r = match v {
                MetaDataUnit::Future(idx) => self.get_value_inner(idx, seen),
                MetaDataUnit::Node(idxs) => {
                    let vs = idxs
                        .into_iter()
                        .map(|v| self.get_value_inner(v, seen.clone()))
                        .collect::<WResult<Vec<_>>>()?;
                    Ok(MetaDataValue::Node(vs))
                }
                MetaDataUnit::String(v) => Ok(MetaDataValue::String(v)),
                MetaDataUnit::Value(v) => Ok(MetaDataValue::Value(v)),
            };
            return r;
        }
        Err(WError::CodeNotAllowed("metadata get value", idx as u32))
    }

    pub(in crate::bitcode) fn get_value(&self, idx: usize) -> WResult<MetaDataValue> {
        self.get_value_inner(idx, HashSet::with_capacity(64))
    }
}

impl<F: Read + Seek> BitReader<F> {
    pub(in crate::bitcode) fn parse_metadata_block(
        &mut self,
        mut state: StateBlock,
        block_infos: &BlockInfoAll,
        data_types: &TypeBlock,
        value_list: &GlobalValueList,
        res: &mut MetaData,
        named_res: &mut NamedMetaData,
    ) -> WResult<()> {
        let mut name = None;
        let mut parse_metadata_block_now = |data: BitCodeEntryRecord<usize>| {
            let code = MetaDatakCode::try_from(data.code)?;
            let r = match code {
                MetaDatakCode::Strings => {
                    let vs = parse_strings(data.data, data.blob)?;
                    for v in vs {
                        res.data.push(MetaDataUnit::String(v));
                    }
                    return Ok(());
                }
                MetaDatakCode::Value => {
                    let v = parse_value(data.data, data_types, value_list)?;
                    MetaDataUnit::Value(v)
                }
                MetaDatakCode::Node => {
                    let r = MetaDataUnit::Node(data.data);
                    if res.data.iter().find(|&v| v == &r).is_some() {
                        return Ok(());
                    }
                    // eprintln!("Node {:?}", r);
                    r
                }
                MetaDatakCode::DistinctNode => MetaDataUnit::Node(data.data),
                MetaDatakCode::Name => {
                    let s = data
                        .data
                        .into_iter()
                        .map(|v| v as u8 as char)
                        .collect::<String>();
                    if name.replace(s).is_some() {
                        return Err(WError::DataSetAlready("MetaData Name"));
                    }
                    return Ok(());
                }
                MetaDatakCode::NamedNode => {
                    if let Some(name_now) = name.take() {
                        named_res.insert(name_now, data.data);
                        return Ok(());
                    }
                    return Err(WError::DataNotFound("MetaData Name"));
                }
            };
            // eprintln!("Metadata: {} {:?}", res.data.len(), r);
            res.data.push(r);
            Ok(())
        };

        self.parse_entry_only_record(
            &mut state,
            block_infos,
            "MetaData Block",
            &mut parse_metadata_block_now,
        )?;
        res.data.shrink_to_fit();
        // eprintln!("Metadata All: {}", res.data.len());
        Ok(())
    }
}

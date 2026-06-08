//! type block
//! https://llvm.org/docs/BitCodeFormat.html#type-block-contents

use std::{
    io::{Read, Seek},
    rc::Rc,
};

use crate::{
    WError, WResult,
    bitcode::{blocks::blockinfo::BlockInfoAll, entry::BitCodeEntryRecord, parse::StateBlock},
};

use crate::filebit::BitReader;

macro_rules! enum_type_block {
    (
        $($key1:ident = $value1:expr,)*;
        $($key2:ident = ($value2:expr, $func2:ident, $ty2:ty, $num:expr),)*;
        $($key3:ident = ($value3:expr, $func3:ident, $ty3:ty),)*;
        $($key4:ident = ($value4:expr, $func4:ident, $ty4:ty),)*;
        $($key5:ident = $value5:expr,)*
    ) => {
        #[derive(Debug, PartialEq)]
        pub(in crate::bitcode) enum TypeBlockData {
            $(
                $key1,
            )*
            $(
                $key2($ty2),
            )*
            $(
                $key3($ty3),
            )*
            Unknown(TypeBlockUnknown)
        }

        #[derive(Debug, Clone)]
        enum TypeBlockSingle {
            $(
                $key1,
            )*
            $(
                $key2($ty2),
            )*
            $(
                $key3($ty3),
            )*
            $(
                $key4($ty4),
            )*
            $(
                $key5(String),
            )*
            Unknown(TypeBlockUnknown)
        }

        impl TryFrom<TypeBlockSingle> for TypeBlockData {
            type Error = WError;
            fn try_from(value: TypeBlockSingle) -> WResult<Self> {
                match value {
                    $(
                        TypeBlockSingle::$key1 => Ok(TypeBlockData::$key1),
                    )*
                    $(
                        TypeBlockSingle::$key2(v) => Ok(TypeBlockData::$key2(v)),
                    )*
                    $(
                        TypeBlockSingle::$key3(v) => Ok(TypeBlockData::$key3(v)),
                    )*
                    TypeBlockSingle::Unknown(v) => Ok(TypeBlockData::Unknown(v)),
                    _ => Err(WError::DataNotAllowed(
                        "Convert data type in parse type block",
                        format!("{:?}", value)
                    ))
                }
            }
        }

        fn parse_type_block(code: u32, data: Vec<usize>, chars: String, blob: Vec<u8>, type_all: &TypeAll) -> WResult<TypeBlockSingle>{
            let res = match code {
                $(
                    $value1 => TypeBlockSingle::$key1,
                )*
                $(
                    $value2 => {
                        if data.len() < $num {
                            return Err(WError::Num(
                                "Data num in parse type block",
                                false,
                                data.len(),
                                $num
                            ))
                        }
                        let inner = $func2(data, type_all)?;
                        TypeBlockSingle::$key2(inner)
                    },
                )*
                $(
                    $value3 => {
                        let inner = $func3(type_all)?;
                        TypeBlockSingle::$key3(inner)
                    },
                )*
                $(
                    $value4 => {
                        let inner = $func4(data, type_all)?;
                        TypeBlockSingle::$key4(inner)
                    },
                )*
                $(
                    $value5 => TypeBlockSingle::$key5(chars),
                )*
                _ => {
                    TypeBlockSingle::Unknown(TypeBlockUnknown {
                        code,
                        data,
                        chars,
                        blob
                    })
                }
            };
            Ok(res)
        }

    };
}

pub(in crate::bitcode) type TypeBlockUnit = Rc<TypeBlockData>;

/// https://llvm.org/docs/BitCodeFormat.html#type-code-function-record
#[derive(Debug, Clone, PartialEq)]
pub(in crate::bitcode) struct TypeBlockFunction {
    pub(in crate::bitcode) vararg: bool,
    pub(in crate::bitcode) retty: TypeBlockUnit,
    pub(in crate::bitcode) paramty: Vec<TypeBlockUnit>,
}

/// https://llvm.org/docs/BitCodeFormat.html#type-code-function-old-record
#[derive(Debug, Clone, PartialEq)]
pub(in crate::bitcode) struct TypeBlockFunctionOld {
    pub(in crate::bitcode) ignored: bool,
    pub(in crate::bitcode) func: TypeBlockFunction,
}

/// https://llvm.org/docs/BitCodeFormat.html#type-code-array-record
#[derive(Debug, Clone, PartialEq)]
pub(in crate::bitcode) struct TypeBlockArray {
    pub(in crate::bitcode) num: usize,
    pub(in crate::bitcode) ty: TypeBlockUnit,
}

/// https://llvm.org/docs/BitCodeFormat.html#type-code-struct-anon-record
#[derive(Debug, Clone, PartialEq)]
pub(in crate::bitcode) struct TypeBlockStructAnon {
    pub(in crate::bitcode) is_packed: bool,
    pub(in crate::bitcode) tys: Vec<TypeBlockUnit>,
}

/// https://llvm.org/docs/BitCodeFormat.html#type-code-struct-named-record
#[derive(Debug, Clone, PartialEq)]
pub(in crate::bitcode) struct TypeBlockStructNamed {
    pub(in crate::bitcode) name: String,
    pub(in crate::bitcode) is_packed: bool,
    pub(in crate::bitcode) tys: Vec<TypeBlockUnit>,
}

/// https://llvm.org/docs/BitCodeFormat.html#type-code-target-type-record
#[derive(Debug, Clone, PartialEq)]
pub(in crate::bitcode) struct TypeBlockTargetType {
    pub(in crate::bitcode) ty_params: Vec<TypeBlockUnit>,
    pub(in crate::bitcode) int_params: Vec<TypeBlockUnit>,
}

/// https://llvm.org/docs/BitCodeFormat.html#type-code-pointer-record
#[derive(Debug, Clone, PartialEq)]
pub(in crate::bitcode) struct TypeBlockPointer {
    pub(in crate::bitcode) ty: TypeBlockUnit,
    pub(in crate::bitcode) space: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub(in crate::bitcode) struct TypeBlockUnknown {
    pub(in crate::bitcode) code: u32,
    pub(in crate::bitcode) data: Vec<usize>,
    pub(in crate::bitcode) chars: String,
    pub(in crate::bitcode) blob: Vec<u8>,
}

pub(in crate::bitcode) type TypeBlock = Vec<TypeBlockUnit>;
type TypeAll = Vec<TypeBlockSingle>;

fn get_ty(type_all: &TypeAll, idx: usize) -> WResult<TypeBlockUnit> {
    let v = type_all
        .get(idx)
        .cloned()
        .ok_or(WError::CodeNotAllowed("Type Idx", idx as u32))?;
    let r = TypeBlockData::try_from(v)?;
    Ok(Rc::new(r))
}

fn convert_tys(data: Vec<usize>, type_all: &TypeAll, skip: usize) -> WResult<Vec<TypeBlockUnit>> {
    data.into_iter()
        .skip(skip)
        .map(|v| get_ty(type_all, v))
        .collect::<WResult<Vec<_>>>()
}

fn convert_usize(data: Vec<usize>, _type_all: &TypeAll) -> WResult<usize> {
    if data.is_empty() {
        return Err(WError::DataNotFound("convert data in parse type block"));
    }
    Ok(data[0])
}

fn convert_pointer(data: Vec<usize>, type_all: &TypeAll) -> WResult<TypeBlockPointer> {
    let ty = data[0];
    let ty = get_ty(type_all, ty)?;
    let space = data.get(1).cloned().unwrap_or(0);
    Ok(TypeBlockPointer { ty, space })
}

fn convert_func_old(data: Vec<usize>, type_all: &TypeAll) -> WResult<TypeBlockFunctionOld> {
    let vararg = data[0] != 0;
    let ignored = data[1] != 0;
    let retty = data[2];
    let retty = get_ty(type_all, retty)?;
    let paramty = convert_tys(data, type_all, 3)?;
    let func = TypeBlockFunction {
        vararg,
        retty,
        paramty,
    };
    Ok(TypeBlockFunctionOld { ignored, func })
}

fn convert_array(data: Vec<usize>, type_all: &TypeAll) -> WResult<TypeBlockArray> {
    let num = data[0];
    let ty = data[1];
    let ty = get_ty(type_all, ty)?;
    Ok(TypeBlockArray { num, ty })
}

fn convert_struct_anno(data: Vec<usize>, type_all: &TypeAll) -> WResult<TypeBlockStructAnon> {
    let is_packed = data[0] != 0;
    let tys = convert_tys(data, type_all, 1)?;
    Ok(TypeBlockStructAnon { is_packed, tys })
}

fn convert_opaque(type_all: &TypeAll) -> WResult<String> {
    type_all
        .last()
        .and_then(|v| {
            if let TypeBlockSingle::StructName(v) = v {
                Some(v.clone())
            } else {
                None
            }
        })
        .ok_or(WError::DataNotAllowed(
            "Opaque name",
            format!("{:?}", type_all.last()),
        ))
}

fn convert_struct_named(data: Vec<usize>, type_all: &TypeAll) -> WResult<TypeBlockStructNamed> {
    let anno = convert_struct_anno(data, type_all)?;
    let name = type_all
        .last()
        .and_then(|v| {
            if let TypeBlockSingle::StructName(v) = v {
                Some(v.clone())
            } else {
                None
            }
        })
        .ok_or(WError::DataNotAllowed(
            "StructNamedAnno name",
            format!("{:?}", type_all.last()),
        ))?;
    Ok(TypeBlockStructNamed {
        name,
        is_packed: anno.is_packed,
        tys: anno.tys,
    })
}

fn convert_func(data: Vec<usize>, type_all: &TypeAll) -> WResult<TypeBlockFunction> {
    let vararg = data[0] != 0;
    let retty = data[1];
    let retty = get_ty(type_all, retty)?;
    let paramty = convert_tys(data, type_all, 2)?;
    Ok(TypeBlockFunction {
        vararg,
        retty,
        paramty,
    })
}

fn convert_target_type(data: Vec<usize>, type_all: &TypeAll) -> WResult<TypeBlockTargetType> {
    let num_tys = data[0];
    let mut ty_params = convert_tys(data, type_all, 1)?;
    let int_params = ty_params.split_off(num_tys);
    Ok(TypeBlockTargetType {
        ty_params,
        int_params,
    })
}

// 1. 简单分类，没有子数据
// 2. 有子数据，并且子数据依赖data，至少有一定长度
// 3. 有子数据，并且子数据只依赖type_all
// 4. 有子数据，但是是中间过程，不在最终结果内
// 5. 有子数据，直接读取char，不在最终结果内
enum_type_block! {
    Void = 2,
    Float = 3,
    Double = 4,
    Label = 5,
    Half = 10,
    X86Fp80 = 13,
    Fp128 = 14,
    PpcFp128 = 15,
    MetaData = 16,
    X86Mmx = 17,
    BFloat = 23,
    X86Amx = 24,
    ;
    Integer = (7, convert_usize, usize, 1),
    Pointer = (8, convert_pointer, TypeBlockPointer, 1),
    FunctionOld = (9, convert_func_old, TypeBlockFunctionOld, 3),
    Array = (11, convert_array, TypeBlockArray, 2),
    Vector = (12, convert_array, TypeBlockArray, 2),
    StructAnon = (18, convert_struct_anno, TypeBlockStructAnon, 1),
    StructNamed = (20, convert_struct_named, TypeBlockStructNamed, 1),
    Function = (21, convert_func, TypeBlockFunction, 2),
    OpaquePointer = (25, convert_usize, usize, 1),
    TragetType = (26, convert_target_type, TypeBlockTargetType, 1),
    ;
    Opaque = (6, convert_opaque, String),
    ;
    NumEntry = (1, convert_usize, usize),
    ;
    StructName = 19,
}

impl TypeBlockData {
    pub(super) fn is_float(&self) -> bool {
        matches!(
            self,
            Self::Float | Self::Double | Self::Half | Self::X86Fp80 | Self::Fp128 | Self::BFloat
        )
    }
    pub(super) fn is_float_vec(&self) -> bool {
        match self {
            Self::Array(v) if v.ty.is_float() => true,
            Self::Vector(v) if v.ty.is_float() => true,
            _ => false,
        }
    }
}

// impl Display for TypeBlockData {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let r = match self {
//             Self::
//         }
//         write!(
//             f,
//             "{}",

//         )
//     }
// }

impl<F: Read + Seek> BitReader<F> {
    pub(in crate::bitcode) fn parse_type_block(
        &mut self,
        mut state: StateBlock,
        block_infos: &BlockInfoAll,
    ) -> WResult<TypeBlock> {
        // todo!("parse module block: {:?} {}", state.get_current(), reader.position()?);
        let mut all = Vec::with_capacity(64);
        let mut res = Vec::with_capacity(64);
        let mut n = 0;
        let mut parse_type_block_now = |data: BitCodeEntryRecord<usize>| {
            let data = parse_type_block(data.code, data.data, data.chars, data.blob, &all)?;
            // eprintln!("Type single: {:?}", data);
            if let Ok(v) = TypeBlockData::try_from(data.clone()) {
                res.push(Rc::new(v));
            }
            if let TypeBlockSingle::NumEntry(num) = data {
                n = num;
            } else {
                all.push(data);
            }
            Ok(())
        };

        self.parse_entry_only_record(
            &mut state,
            block_infos,
            "Type Block",
            &mut parse_type_block_now,
        )?;
        if res.len() != n {
            return Err(WError::Num("Type block data num", true, n, res.len()));
        }
        Ok(res)
    }
}

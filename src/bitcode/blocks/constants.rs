//! Constants Block
//! https://llvm.org/docs/BitCodeFormat.html#constants-block

use std::{
    fmt::Debug,
    io::{Read, Seek},
    rc::Rc,
};

use crate::{
    WError, WResult,
    bitcode::{
        blocks::{blockinfo::BlockInfoAll, typeblock::TypeBlock},
        entry::BitCodeEntryRecord,
        parse::StateBlock,
    },
};

use crate::{
    bitcode::blocks::typeblock::{TypeBlockData, TypeBlockUnit},
    bitreader::{BitReader, decoded_signed_vbrs},
};

macro_rules! enum_constants {
    (
        "inner_head";
        $($key1:ident = $value1:expr,)*;
        $($key2:ident = $value2:expr,)*;
        $($key3:ident = (
            $value3:expr,
            $($key_data3:ident, $ty3:ty)*
        ),)*
        ;
        $($key4:ident = (
            $value4:expr,
            $($key_data4:ident, $ty4:ty)*
        ),)*
        ;
        $($key5:ident = ($value5:expr, $key_data5:ident),)*
    ) => {
        enum_parse!{
            ConstantsCode, u32;
            Debug,;
            $($key1 = $value1,)*
            $($key2 = $value2,)*
            $($key3 = $value3,)*
            $($key4 = $value4,)*
            $($key5 = $value5,)*
        }

        #[derive(Debug, PartialEq)]
        pub(in crate::bitcode) enum DataSingle {
            $($key1(TypeBlockUnit),)*
            $($key2(TypeBlockUnit),)*
            $($($key_data3(TypeBlockUnit, $ty3),)*)*
            $($($key_data4(TypeBlockUnit, $ty4),)*)*
            $($key_data5(TypeBlockUnit, Vec<u8>),)*
        }

        #[derive(Debug)]
        pub(in crate::bitcode) enum ValueSingle {
            $($key1,)*
            $($key2,)*
            $($($key_data3($ty3),)*)*
            $($($key_data4($ty4),)*)*
            $($key_data5(Vec<u8>),)*
        }

        impl DataSingle {
            pub(in crate::bitcode) fn ty(&self) -> TypeBlockUnit {
                match self {
                    $(
                        Self::$key1(v) => v.clone(),
                    )*
                    $(
                        Self::$key2(v) => v.clone(),
                    )*
                    $($(
                        Self::$key_data3(v, _) => v.clone(),
                    )*)*
                    $($(
                        Self::$key_data4(v, _) => v.clone(),
                    )*)*
                    $(
                        Self::$key_data5(v, _) => v.clone(),
                    )*
                }
            }
            pub(in crate::bitcode) fn value(&self, ty: &TypeBlockUnit) -> WResult<ValueSingle> {
                match self {
                    $(
                        Self::$key1(_) => Ok(ValueSingle::$key1),
                    )*
                    $(
                        Self::$key2(_) => Ok(ValueSingle::$key2),
                    )*
                    $($(
                        Self::$key_data3(ty_temp, v) if ty_temp == ty => Ok(ValueSingle::$key_data3(v.clone())),
                    )*)*
                    $($(
                        Self::$key_data4(ty_temp, v) if ty_temp == ty => Ok(ValueSingle::$key_data4(v.clone())),
                    )*)*
                    $(
                        Self::$key_data5(ty_temp, v) if ty_temp == ty => Ok(ValueSingle::$key_data5(v.clone())),
                    )*
                    _ => Err(WError::DataNotAllowed("data value", format!("Data: {self:?}, ty: {ty:?}")))
                }
            }
            pub(in crate::bitcode) fn value_inner(&self) -> ValueSingle {
                match self {
                    $(
                        Self::$key1(_) => ValueSingle::$key1,
                    )*
                    $(
                        Self::$key2(_) => ValueSingle::$key2,
                    )*
                    $($(
                        Self::$key_data3(_, v) => ValueSingle::$key_data3(v.clone()),
                    )*)*
                    $($(
                        Self::$key_data4(_, v) => ValueSingle::$key_data4(v.clone()),
                    )*)*
                    $(
                        Self::$key_data5(_, v) => ValueSingle::$key_data5(v.clone()),
                    )*
                }
            }
        }
    };

    (
        "inner_tail", $code:expr, $data:expr, $chars:expr, $type_now:expr;
        $($key1:ident = $value1:expr,)*;
        $($key2:ident = ($value2:expr, $func2:ident),)*;
        $($key3:ident = (
            $value3:expr, $default3:expr,
            $($key_data3:ident, $ty3:ty, $func3:ident, $pat3:pat)*
        ),)*
        ;
        $($key4:ident = (
            $value4:expr, $default4:expr,
            $($key_data4:ident, $ty4:ty, $func4:ident, $pat4:pat, $pat4_sub:expr)*
        ),)*
        ;
        $($key5:ident = ($value5:expr, $key_data5:ident),)*
    ) => {
        match $code {
            $(
                ConstantsCode::$key1 => DataSingle::$key1($type_now.clone()),
            )*
            $(
                ConstantsCode::$key2 => {
                    $func2($type_now.as_ref())?;
                    DataSingle::$key2($type_now.clone())
                },
            )*
            $(
                ConstantsCode::$key3 => {
                    match $type_now.as_ref() {
                        $(
                            $pat3 => {
                                let data = $func3($data, )?;
                                DataSingle::$key_data3($type_now.clone(), data)
                            },
                        )*
                        _ => $default3
                    }
                },
            )*
            $(
                ConstantsCode::$key4 => {
                    match $type_now.as_ref() {
                        $(
                            $pat4 if $pat4_sub => {
                                let data = $func4($data)?;
                                DataSingle::$key_data4($type_now.clone(), data)
                            },
                        )*
                        _ => $default4
                    }
                },
            )*
            $(
                ConstantsCode::$key5 => {
                    DataSingle::$key_data5(
                        $type_now.clone(),
                        $data.into_iter().map(| v | v as u8).collect()
                    )
                },
            )*
        }
    };

    (
        $($key1:ident = $value1:expr,)*;
        $($key2:ident = ($value2:expr, $func2:ident),)*;
        $($key3:ident = (
            $value3:expr,
            $($key_data3:ident, $ty3:ty, $func3:ident, $pat3:pat)*
        ),)*
        ;
        $($key5:ident = (
            $value5:expr, $default5:ident,
            $($key_data5:ident, $ty5:ty, $func5:ident, $pat5:pat)*
        ),)*
        ;
        $($key4:ident = (
            $value4:expr,
            $($key_data4:ident, $ty4:ty, $func4:ident, $pat4:pat, $pat4_sub:expr)*
        ),)*
        ;
        $($key6:ident = (
            $value6:expr, $default6:ident,
            $($key_data6:ident, $ty6:ty, $func6:ident, $pat6:pat, $pat6_sub:expr)*
        ),)*
        ;
        $($key7:ident = ($value7:expr, $key_data7:ident),)*
    ) => {
        enum_constants!{
            "inner_head";
            $($key1 = $value1,)*;
            $($key2 = $value2,)*;
            $($key3 = (
                $value3,
                $($key_data3, $ty3)*
            ),)*
            $($key5 = (
                $value5,
                $($key_data5, $ty5)*
            ),)*
            ;
            $($key4 = (
                $value4,
                $($key_data4, $ty4)*
            ),)*
            $($key6 = (
                $value6,
                $($key_data6, $ty6)*
            ),)*
            ;
            $($key7 = ($value7, $key_data7),)*
        }

        fn parse_constants_block(code: u32, data: Vec<u64>, type_now: &TypeBlockUnit) -> WResult<DataSingle> {
            let code = ConstantsCode::try_from(code)?;
            let r = enum_constants!{
                "inner_tail", code, data, chars, type_now;
                $($key1 = $value1,)*;
                $($key2 = ($value2, $func2),)*;
                $($key3 = (
                    $value3, return Err(WError::CodeNotAllowed(
                        stringify!($key3),
                        $value3
                    )),
                    $($key_data3, $ty3, $func3, $pat3)*
                ),)*
                $($key5 = (
                    $value5, DataSingle::$default5(type_now.clone()),
                    $($key_data5, $ty5, $func5, $pat5)*
                ),)*
                ;
                $($key4 = (
                    $value4, return Err(WError::DataNotAllowed(
                        stringify!($key4),
                        format!("{:?}", type_now.as_ref())
                    )),
                    $($key_data4, $ty4, $func4, $pat4, $pat4_sub)*
                ),)*
                $($key6 = (
                    $value6, DataSingle::$default6(type_now.clone()),
                    $($key_data6, $ty6, $func6, $pat6, $pat6_sub)*
                ),)*
                ;
                $($key7 = ($value7, $key_data7),)*
            };
            Ok(r)
        }
    };

    ("method_integer"; $($name_integer:ident, $name_wide:ident, $ty_sign:ty, $ty_unsign:ty;)*) => {
        $(
            fn $name_integer(data: Vec<u64>) -> WResult<$ty_unsign> {
                if data.len() != 1 {
                    return Err(WError::DataNotAllowed(
                        stringify!($name_integer),
                        format!("{:?}", data)
                    ));
                }
                let raw = data[0] as $ty_sign;
                let res = decoded_signed_vbrs(raw) as $ty_unsign;
                Ok(res)
            }

            fn $name_wide(data: Vec<u64>) -> WResult<Vec<$ty_unsign>> {
                let res = data.into_iter()
                    .map(| v | decoded_signed_vbrs(v as $ty_sign) as $ty_unsign)
                    .collect::<Vec<_>>();
                Ok(res)
            }
        )*
    };

    ("method_as"; $($name_single:ident, $name_vec:ident, $ty:ty;)*) => {
        $(
            fn $name_single(data: Vec<u64>) -> WResult<$ty> {
                if data.is_empty() {
                    return Err(WError::DataNotFound(
                        stringify!($name_single)
                    ));
                }
                Ok(data[0] as $ty)
            }

            fn $name_vec(data: Vec<u64>) -> WResult<Vec<$ty>> {
                let res = data.into_iter()
                    .map(| v | v as $ty)
                    .collect::<Vec<_>>();
                Ok(res)
            }
        )*
    };


    ("value_single_usize"; $($name:ident,)*) => {
        impl TryInto<usize> for ValueSingle {
            type Error = WError;
            fn try_into(self) -> WResult<usize> {
                match self {
                    $(
                        Self::$name(v) => Ok(v as usize),
                    )*
                    _ => Err(WError::DataNotAllowed("value_single usize", format!("{:?}", self)))
                }
            }
        }
    };


    (
        "value_single_usize_vec";
        $($name1:ident,)*
        ;
        $($name2:ident,)*
    ) => {
        impl TryInto<Vec<usize>> for ValueSingle {
            type Error = WError;
            fn try_into(self) -> WResult<Vec<usize>> {
                match self {
                    $(
                        Self::$name1(v) => Ok(v.clone()),
                    )*
                    $(
                        Self::$name2(v) => Ok(v
                            .clone()
                            .into_iter()
                            .map(| vv | vv as usize)
                            .collect()),
                    )*
                    _ => Err(WError::DataNotAllowed("value_single usize vec", format!("{:?}", self)))
                }
            }
        }
    };


    ("value_single_string"; $($name:ident,)*) => {
        impl TryInto<String> for ValueSingle {
            type Error = WError;
            fn try_into(self) -> WResult<String> {
                match self {
                    $(
                        Self::$name(v) => Ok(v.into_iter().map(| v | v as char).collect()),
                    )*
                    _ => Err(WError::DataNotAllowed("value_single string", format!("{:?}", self)))
                }
            }
        }
    }
}

fn parse_constants_null(type_now: &TypeBlockData) -> WResult<()> {
    if matches!(
        type_now,
        TypeBlockData::Void | TypeBlockData::Function(_) | TypeBlockData::Label
    ) {
        return Err(WError::DataNotAllowed(
            "Null value",
            format!("{:?}", type_now),
        ));
    }
    Ok(())
}

fn parse_constants_integer1(data: Vec<u64>) -> WResult<bool> {
    if data.is_empty() {
        return Err(WError::DataNotFound("parse constants integer1"));
    }
    Ok(data[0] != 0)
}

fn parse_constants_vec_integer1(data: Vec<u64>) -> WResult<Vec<bool>> {
    let res = data.into_iter().map(|v| v != 0).collect::<Vec<_>>();
    Ok(res)
}

enum_constants! {
    "method_integer";
    parse_constants_integer16, parse_constants_vec_integer16, i32, u16;
    parse_constants_integer32, parse_constants_vec_integer32, i64, u32;
    parse_constants_integer64, parse_constants_vec_integer64, i128, u64;
}

enum_constants! {
    "method_as";
    parse_constants_integer8, parse_constants_vec_integer8, u8;
    parse_constants_f32, parse_constants_vec_f32, f32;
    parse_constants_f64, parse_constants_vec_f64, f64;
}

fn parse_constants_f128(data: Vec<u64>) -> WResult<[u8; 16]> {
    if data.len() != 16 {
        return Err(WError::Num("F128 data len", false, data.len(), 1));
    }
    let mut res = [0; 16];
    let data = data.into_iter().map(| v | v as u8).collect::<Vec<_>>();
    res.clone_from_slice(&data);
    Ok(res)
}

fn parse_constants_aggregate(data: Vec<u64>) -> WResult<Vec<usize>> {
    Ok(data.into_iter().map(|v| v as usize).collect())
}

fn parse_constants_bit128(data: Vec<u64>) -> WResult<u128> {
    if data.len() != 2 {
        return Err(WError::DataNotAllowed(
            "parse_constants_bit128",
            format!("{:?}", data)
        ));
    }
    let v1 = data[0].to_le_bytes();
    let v2 = data[1].to_le_bytes();
    let mut vs = [0; 16];
    vs[..8].clone_from_slice(&v1);
    vs[8..].clone_from_slice(&v2);
    Ok(u128::from_le_bytes(vs))
}

fn parse_constants_integer128(data: Vec<u64>) -> WResult<u128> {
    let data = data
        .into_iter()
        .map(| v | decoded_signed_vbrs(v as i128) as u64)
        .collect::<Vec<_>>();
    let res = parse_constants_bit128(data)? + 1;
    Ok(res)
}
fn parse_constants_vec_integer128(mut data: Vec<u64>) -> WResult<Vec<u128>> {
    let mut res = Vec::with_capacity(data.len() >> 1);
    while !data.is_empty() {
        let v = data.split_off(2);
        let t = parse_constants_integer128(data)?;
        data = v;
        res.push(t);
    }
    Ok(res)
}

const SET_TYPE: u32 = 1;

// 1. 一种value对应任意type
// 2. 一种value对应多个type，没有具体值，需要检查type
// 3/5. 一种value对应多个type，不同type有不同具体值，每个type都有子数据，不匹配子类型
// 4/6. 一种value对应多个type，不同type有不同具体值，每个type都有子数据，匹配子类型
// 7. 一种value对应多个type，有具体值，需要生成子数据，子数据就是string
enum_constants! {
    Undef = 3, // UNDEF
    Poison = 26, // POISON
    ;
    Null = (2, parse_constants_null), // NULL
    ;
    Integer = (4,
        Integer1, bool, parse_constants_integer1, TypeBlockData::Integer(1)
        Integer8, u8, parse_constants_integer8, TypeBlockData::Integer(8)
        Integer16, u16, parse_constants_integer16, TypeBlockData::Integer(16)
        Integer32, u32, parse_constants_integer32, TypeBlockData::Integer(32)
        Integer64, u64, parse_constants_integer64, TypeBlockData::Integer(64)
        Integer128, u128, parse_constants_integer128, TypeBlockData::Integer(128)
    ), // [intval]
    ;
    Float = (6, Poison,
        F16, u16, parse_constants_integer16, TypeBlockData::Half
        BF16, u16, parse_constants_integer16, TypeBlockData::BFloat
        F32, f32, parse_constants_f32, TypeBlockData::Float
        F64, f64, parse_constants_f64, TypeBlockData::Double
        X86F80, [u8; 16], parse_constants_f128, TypeBlockData::X86Fp80
        F128, [u8; 16], parse_constants_f128, TypeBlockData::Fp128
        PpcF128, [u8; 16], parse_constants_f128, TypeBlockData::PpcFp128
    ), // [fpval]
    Aggregate = (7, Poison,
        AggStruct, Vec<usize>, parse_constants_aggregate, TypeBlockData::StructNamed(_) | TypeBlockData::StructAnon(_)
        AggArray, Vec<usize>, parse_constants_aggregate, TypeBlockData::Array(_)
        AggVector, Vec<usize>, parse_constants_aggregate, TypeBlockData::Vector(_)
    ), // [n x value number]
    ;
    WideInteger = (5,
        ArrayWide1, Vec<bool>, parse_constants_vec_integer1,
        TypeBlockData::Array(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(1))

        VectorWide1, Vec<bool>, parse_constants_vec_integer1,
        TypeBlockData::Vector(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(1))

        Wide1, bool, parse_constants_integer1,
        TypeBlockData::Integer(v), *v == 1

        ArrayWide8, Vec<u8>, parse_constants_vec_integer8,
        TypeBlockData::Array(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(8))

        VectorWide8, Vec<u8>, parse_constants_vec_integer8,
        TypeBlockData::Vector(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(8))

        Wide8, u8, parse_constants_integer8,
        TypeBlockData::Integer(v), *v == 8

        ArrayWide16, Vec<u16>, parse_constants_vec_integer16,
        TypeBlockData::Array(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(16))

        VectorWide16, Vec<u16>, parse_constants_vec_integer16,
        TypeBlockData::Vector(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(16))

        Wide16, u16, parse_constants_integer16,
        TypeBlockData::Integer(v), *v == 16

        ArrayWide32, Vec<u32>, parse_constants_vec_integer32,
        TypeBlockData::Array(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(32))

        VectorWide32, Vec<u32>, parse_constants_vec_integer32,
        TypeBlockData::Vector(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(32))

        Wide32, u32, parse_constants_integer32,
        TypeBlockData::Integer(v), *v == 32

        ArrayWide64, Vec<u64>, parse_constants_vec_integer64,
        TypeBlockData::Array(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(64))

        VectorWide64, Vec<u64>, parse_constants_vec_integer64,
        TypeBlockData::Vector(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(64))

        Wide64, u64, parse_constants_integer64,
        TypeBlockData::Integer(v), *v == 64

        ArrayWide128, Vec<u128>, parse_constants_vec_integer128,
        TypeBlockData::Vector(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(128))

        VectorWide128, Vec<u128>, parse_constants_vec_integer128,
        TypeBlockData::Vector(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(128))

        Wide128, u128, parse_constants_integer128,
        TypeBlockData::Integer(v), *v == 128
    ), // [n x intval]
    Data = (22,
        ArrayU8, Vec<u8>, parse_constants_vec_integer8,
        TypeBlockData::Array(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(8))

        VectorU8, Vec<u8>, parse_constants_vec_integer8,
        TypeBlockData::Vector(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(8))

        ArrayU16, Vec<u16>, parse_constants_vec_integer16,
        TypeBlockData::Array(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(16))

        VectorU16, Vec<u16>, parse_constants_vec_integer16,
        TypeBlockData::Vector(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(16))

        ArrayU32, Vec<u32>, parse_constants_vec_integer32,
        TypeBlockData::Array(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(32))

        VectorU32, Vec<u32>, parse_constants_vec_integer32,
        TypeBlockData::Vector(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(32))

        ArrayU64, Vec<u64>, parse_constants_vec_integer64,
        TypeBlockData::Array(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(64))

        VectorU64, Vec<u64>, parse_constants_vec_integer64,
        TypeBlockData::Vector(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(64))

        ArrayU128, Vec<u128>, parse_constants_vec_integer128,
        TypeBlockData::Array(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(128))

        VectorU128, Vec<u128>, parse_constants_vec_integer128,
        TypeBlockData::Vector(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(128))

        ArrayF16, Vec<u16>, parse_constants_vec_integer16,
        TypeBlockData::Array(v), matches!(v.ty.as_ref(), TypeBlockData::Half)

        VectorF16, Vec<u16>, parse_constants_vec_integer16,
        TypeBlockData::Vector(v), matches!(v.ty.as_ref(), TypeBlockData::Half)

        ArrayBF16, Vec<u16>, parse_constants_vec_integer16,
        TypeBlockData::Array(v), matches!(v.ty.as_ref(), TypeBlockData::BFloat)

        VectorBF16, Vec<u16>, parse_constants_vec_integer16,
        TypeBlockData::Vector(v), matches!(v.ty.as_ref(), TypeBlockData::BFloat)

        ArrayF32, Vec<f32>, parse_constants_vec_f32,
        TypeBlockData::Array(v), matches!(v.ty.as_ref(), TypeBlockData::Float)

        VectorF32, Vec<f32>, parse_constants_vec_f32,
        TypeBlockData::Vector(v), matches!(v.ty.as_ref(), TypeBlockData::Float)

        ArrayF64, Vec<f64>, parse_constants_vec_f64,
        TypeBlockData::Array(v), matches!(v.ty.as_ref(), TypeBlockData::Double)

        VectorF64, Vec<f64>, parse_constants_vec_f64,
        TypeBlockData::Vector(v), matches!(v.ty.as_ref(), TypeBlockData::Double)
    ), // [n x elements]
    ;
    ;
    String = (8, String), // [values]
    CString = (9, CString), // [values]
}
// CeBinop = 10, // [opcode, opval, opval]
// CeCast = 11, // [opcode, opty, opval]
// CeGepOld = 12, // [n x operands]
// CeSelect = 13, // [opval, opval, opval]
// CeExtractelt = 14, // [opty, opval, opval]
// CeInsertelt = 15, // [opval, opval, opval]
// CeShufflevec = 16, // [opval, opval, opval]
// CeCmp = 17, // [opty, opval, opval, pred]
// InlineasmOld = 18, // [sideeffect|alignstack,asmstr,conststr]
// CeShufvecEx = 19, // [opty, opval, opval, opval]
// CeInboundsGep = 20, // [n x operands]
// Blockaddress = 21, // [fnty, fnval, bb#]
// InlineasmOld2 = 23, // [sideeffect|alignstack|asmdialect,asmstr,conststr]
// CeGepWithInrangeIndexOld = 24, // [opty, flags, n x operands]
// CeUnop = 25, // [opcode, opval]
// DsoLocalEquivalent = 27, // [gvty, gv]
// InlineasmOld3 = 28, // [sideeffect|alignstack|asmdialect|unwind,asmstr,conststr]
// NoCfiValue = 29, // [fty, f]
// Inlineasm = 30, // [fnty,sideeffect|alignstack|asmdialect|unwind,asmstr,conststr]
// CeGepWithInrange = 31, // [flags, range, n x operands]
// CeGep = 32, // [flags, n x operands]
// Ptrauth = 33, // [key, disc, addrdisc]
// Ptrauth2 = 34, // [key, disc, addrdisc, deactivation_symbol]

enum_constants! {
    "value_single_usize";
    Integer8,
    Integer16,
    Integer32,
    Integer64,
    Integer128,
    Wide8,
    Wide16,
    Wide32,
    Wide64,
    Wide128,
}

enum_constants! {
    "value_single_usize_vec";
    AggArray,
    AggVector,
    ;
    ArrayU8,
    ArrayU16,
    ArrayU32,
    ArrayU64,
    VectorU8,
    VectorU16,
    VectorU32,
    VectorU64,
    ArrayWide8,
    ArrayWide16,
    ArrayWide32,
    ArrayWide64,
    ArrayWide128,
    VectorWide8,
    VectorWide16,
    VectorWide32,
    VectorWide64,
    VectorWide128,
}

enum_constants! {
    "value_single_string";
    String,
    CString,
}

// impl Display for DataSingle {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let r = match self {
//             Self::Integer1(v1, v2)
//         }
//         write!(
//             f,
//             "{}",

//         )
//     }
// }

// impl Debug for DataSingle {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self)
//     }
// }

pub(in crate::bitcode) type ConstantsBlock = Vec<Rc<DataSingle>>;

impl<F: Read + Seek> BitReader<F> {
    pub(in crate::bitcode) fn parse_constants_block(
        &mut self,
        mut state: StateBlock,
        block_infos: &BlockInfoAll,
        data_types: &TypeBlock,
    ) -> WResult<ConstantsBlock> {
        let mut res = Vec::with_capacity(64);
        let mut type_now = Rc::new(TypeBlockData::Void);
        let mut parse_constants_block_now = |data: BitCodeEntryRecord<u64>| {
            // eprintln!("parse constants block: {} {:?} {:?}", code, data, chars);
            if data.code == SET_TYPE {
                type_now = data
                    .data
                    .first()
                    .cloned()
                    .and_then(|idx| data_types.get(idx as usize).cloned())
                    .ok_or(WError::DataNotFound(
                        "No type found in parse constants block",
                    ))?;
                return Ok(());
            }
            let constant = parse_constants_block(data.code, data.data, &type_now)?;
            res.push(Rc::new(constant));
            Ok(())
        };
        self.parse_entry_only_record(
            &mut state,
            block_infos,
            "Constants Block",
            &mut parse_constants_block_now,
        )?;
        res.shrink_to_fit();
        Ok(res)
    }
}

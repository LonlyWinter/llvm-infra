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
        blocks::{
            blockinfo::BlockInfoAll,
            function_block::{
                inst_binop::InstrBinOPType, inst_cast::InstrCastType, inst_gep::InstrGEPNoWrapFlags,
            },
            typeblock::{TypeBlock, TypeBlockData, TypeBlockFunction, TypeBlockUnit},
        },
        entry::BitCodeEntryRecord,
        parse::StateBlock, parse_data_to_string,
    },
    filebit::{
        BitReader, decoded_signed_vbrs_u8, decoded_signed_vbrs_u16, decoded_signed_vbrs_u32, decoded_signed_vbrs_u64
    },
};

macro_rules! enum_constants {
    (
        "inner_head";
        $($key1:ident = $value1:expr,)*;
        $($key2:ident = $value2:expr,)*;
        $($key3:ident = ($value3:expr,$ty3:ty))*
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
            $($key3(TypeBlockUnit, $ty3),)*
            $($($key_data4(TypeBlockUnit, $ty4),)*)*
            $($key_data5(TypeBlockUnit, Rc<Vec<u8>>),)*
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
                    $(
                        Self::$key3(v, _) => v.clone(),
                    )*
                    $($(
                        Self::$key_data4(v, _) => v.clone(),
                    )*)*
                    $(
                        Self::$key_data5(v, _) => v.clone(),
                    )*
                }
            }
        }
    };

    (
        "inner_tail", $code:expr, $data:expr, $chars:expr, $type_now:expr, $data_types:expr;
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
        ;
        $($key6:ident = ($value6:expr, $func6:ident),)*
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
                    let data = if $chars.is_empty() {
                        $data.into_iter().map(| v | v as u8).collect()
                    } else {
                        $chars.chars().into_iter().map(| v | v as u8).collect()
                    };
                    DataSingle::$key_data5(
                        $type_now.clone(),
                        Rc::new(data)
                    )
                },
            )*
            $(
                ConstantsCode::$key6 => {
                    let data = $func6($data, $type_now, $data_types.as_ref())?;
                    DataSingle::$key6($type_now.clone(), data)
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
        ;
        $($key8:ident = ($value8:expr, $ty8:ty, $func8:ident))*
    ) => {
        enum_constants!{
            "inner_head";
            $($key1 = $value1,)*;
            $($key2 = $value2,)*;
            $($key8 = ($value8, $ty8))*
            ;
            $($key3 = (
                $value3,
                $($key_data3, $ty3)*
            ),)*
            $($key4 = (
                $value4,
                $($key_data4, $ty4)*
            ),)*
            $($key5 = (
                $value5,
                $($key_data5, $ty5)*
            ),)*
            $($key6 = (
                $value6,
                $($key_data6, $ty6)*
            ),)*
            ;
            $($key7 = ($value7, $key_data7),)*
        }

        fn parse_constants_block(code: u32, data: Vec<u64>, chars: String, type_now: &TypeBlockUnit, data_types: &TypeBlock) -> WResult<DataSingle> {
            let code = ConstantsCode::try_from(code)?;
            let r = enum_constants!{
                "inner_tail", code, data, chars, type_now, data_types;
                $($key1 = $value1,)*;
                $($key2 = ($value2, $func2),)*;
                $($key3 = (
                    $value3, return Err(WError::DataNotAllowed(
                        stringify!($key3),
                        format!("{:?}", type_now.as_ref())
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
                ;
                $($key8 = ($value8, $func8),)*
            };
            Ok(r)
        }
    };

    ("method_integer"; $($name_integer:ident, $name_wide:ident, $func_vbr:ident, $ty_vbr:ty, $ty_sign:ty;)*) => {
        $(
            fn $name_integer(data: Vec<u64>) -> WResult<$ty_sign> {
                if data.len() != 1 {
                    return Err(WError::DataNotAllowed(
                        stringify!($name_integer),
                        format!("{:?}", data)
                    ));
                }
                let data = data[0] as $ty_vbr;
                let r = $func_vbr(data);
                Ok(r)
            }

            fn $name_wide(data: Vec<u64>) -> WResult<Vec<$ty_sign>> {
                let res = data.into_iter()
                    .map(| v | v as $ty_sign)
                    .collect::<Vec<_>>();
                Ok(res)
            }
        )*
    };

    ("method_float"; $($name_single:ident, $name_vec:ident, $ty0:ty, $ty:ty;)*) => {
        $(
            fn $name_single(data: Vec<u64>) -> WResult<$ty> {
                if data.len() != 1 {
                    return Err(WError::DataNotFound(
                        stringify!($name_single)
                    ));
                }
                Ok(<$ty>::from_le_bytes((data[0] as $ty0).to_le_bytes()))
            }

            fn $name_vec(data: Vec<u64>) -> WResult<Vec<$ty>> {
                let res = data.into_iter()
                    .map(| v | <$ty>::from_le_bytes((v as $ty0).to_le_bytes()))
                    .collect::<Vec<_>>();
                Ok(res)
            }
        )*
    };


    ("data_single_i128"; $($name:ident,)*) => {
        impl DataSingle {
            pub(in crate::bitcode) fn convert_i128(&self) -> WResult<i128> {
                match self {
                    $(
                        Self::$name(_, v) => {
                            Ok(*v as i128)
                        },
                    )*
                    Self::Null(_) => Ok(0),
                    _ => Err(WError::DataNotAllowed("value_single u128", format!("{:?}", self)))
                }
            }
        }
    };



    ("data_single_string"; $($name:ident,)*) => {
        impl DataSingle {
            pub(in crate::bitcode) fn convert_string(&self) -> WResult<String> {
                match self {
                    $(
                        Self::$name(_, v) => Ok(parse_data_to_string(v.iter().cloned())),
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
    if data.len() != 1 {
        return Err(WError::DataNotFound("constants integer1"));
    }
    Ok(data[0] != 0)
}

fn parse_constants_vec_integer1(data: Vec<u64>) -> WResult<Vec<bool>> {
    let res = data.into_iter().map(|v| v != 0).collect::<Vec<_>>();
    Ok(res)
}

enum_constants! {
    "method_integer";
    parse_constants_integer8, parse_constants_vec_integer8, decoded_signed_vbrs_u8, u8, i8;
    parse_constants_integer16, parse_constants_vec_integer16, decoded_signed_vbrs_u16, u16, i16;
    parse_constants_integer32, parse_constants_vec_integer32, decoded_signed_vbrs_u32, u32, i32;
    parse_constants_integer64, parse_constants_vec_integer64, decoded_signed_vbrs_u64, u64, i64;
}

enum_constants! {
    "method_float";
    parse_constants_f16, parse_constants_vec_f16, u16, u16;
    parse_constants_f32, parse_constants_vec_f32, u32, f32;
    parse_constants_f64, parse_constants_vec_f64, u64, f64;
}

fn parse_constants_f128(data: Vec<u64>) -> WResult<[u8; 16]> {
    if data.len() != 16 {
        return Err(WError::Num("F128 data len", false, data.len(), 1));
    }
    let mut res = [0; 16];
    let data = data.into_iter().map(|v| v as u8).collect::<Vec<_>>();
    res.clone_from_slice(&data);
    Ok(res)
}

fn parse_constants_aggregate(data: Vec<u64>) -> WResult<Vec<usize>> {
    Ok(data.into_iter().map(|v| v as usize).collect())
}

fn parse_constants_integer128_inner(data: u64) -> [u8; 8] {
    let sign = (data & 1) as i64;
    let val = (data >> 1) as i64;
    let t = if sign == 0 {
        val
    } else {
        -val
    };
    t.to_le_bytes()
}

pub(super) fn parse_constants_integer128(data: Vec<u64>) -> WResult<i128> {
    if data.len() == 1 {
        let mut vs = [0; 16];
        let v1 = parse_constants_integer128_inner(data[0]);
        vs[..8].clone_from_slice(&v1);
        return Ok(i128::from_le_bytes(vs))
    }
    if data.len() != 2 {
        return Err(WError::DataNotAllowed(
            "parse_constants_bit128",
            format!("{:?}", data),
        ));
    }
    let v1 = parse_constants_integer128_inner(data[0]);
    let v2 = parse_constants_integer128_inner(data[1]);
    let mut vs = [0; 16];
    vs[..8].clone_from_slice(&v1);
    vs[8..].clone_from_slice(&v2);
    Ok(i128::from_le_bytes(vs))
}
fn parse_constants_vec_integer128(mut data: Vec<u64>) -> WResult<Vec<i128>> {
    let mut res = Vec::with_capacity(data.len() >> 1);
    while !data.is_empty() {
        let v = data.split_off(2);
        let t = parse_constants_integer128(data)?;
        data = v;
        res.push(t);
    }
    Ok(res)
}

#[derive(Debug, PartialEq, Clone)]
pub(in crate::bitcode) struct ConstantCast {
    pub(in crate::bitcode) op: InstrCastType,
    pub(in crate::bitcode) ty_res: TypeBlockUnit,
    pub(in crate::bitcode) ty_val: TypeBlockUnit,
    pub(in crate::bitcode) val: usize,
}

fn parse_constant_cast(
    data: Vec<u64>,
    type_now: &TypeBlockUnit,
    data_types: &TypeBlock,
) -> WResult<ConstantCast> {
    if data.len() != 3 {
        return Err(WError::DataNotFound("constants cast"));
    }
    let op = InstrCastType::try_from(data[0] as usize)?;
    let ty_val = data_types
        .get(data[1] as usize)
        .cloned()
        .ok_or(WError::DataNotFound("constant cast ty"))?;
    Ok(ConstantCast {
        op,
        ty_res: type_now.clone(),
        ty_val,
        val: data[2] as usize,
    })
}

#[derive(Debug, PartialEq, Clone)]
pub(in crate::bitcode) struct ConstantBinOP {
    pub(in crate::bitcode) op: InstrBinOPType,
    pub(in crate::bitcode) ty: TypeBlockUnit,
    pub(in crate::bitcode) left: usize,
    pub(in crate::bitcode) right: usize,
}

fn parse_constant_binop(
    data: Vec<u64>,
    type_now: &TypeBlockUnit,
    _data_types: &TypeBlock,
) -> WResult<ConstantBinOP> {
    if data.len() < 3 {
        return Err(WError::DataNotFound("constants cast"));
    }
    let fp = type_now.is_float() || type_now.is_float_vec();
    let mut op = InstrBinOPType::try_from((data[0] as usize, fp))?;
    if data.len() == 4 {
        op.update(data[3] as usize)?;
    }
    Ok(ConstantBinOP {
        op,
        ty: type_now.clone(),
        left: data[1] as usize,
        right: data[2] as usize,
    })
}

#[derive(Debug, PartialEq, Clone)]
pub(in crate::bitcode) enum ConstantInlineASMDialect {
    Att,
    Intel,
}

#[derive(Debug, PartialEq, Clone)]
pub(in crate::bitcode) struct ConstantInlineASM {
    pub(in crate::bitcode) ty: Rc<TypeBlockFunction>,
    pub(in crate::bitcode) asm: String,
    pub(in crate::bitcode) constr: String,
    pub(in crate::bitcode) has_side_effects: bool,
    pub(in crate::bitcode) is_align_stack: bool,
    pub(in crate::bitcode) can_throw: bool,
    pub(in crate::bitcode) asm_dialect: ConstantInlineASMDialect,
}

fn parse_constant_inlineasm(
    mut data: Vec<u64>,
    _type_now: &TypeBlockUnit,
    data_types: &TypeBlock,
) -> WResult<Rc<ConstantInlineASM>> {
    if data.len() < 3 {
        return Err(WError::DataNotFound("constants inlineasm"));
    }
    let ty = data_types
        .get(data[0] as usize)
        .and_then(|v| {
            if let TypeBlockData::Function(v) = v.as_ref() {
                Some(v.clone())
            } else if let TypeBlockData::FunctionOld(v) = v.as_ref() {
                Some(v.func.clone())
            } else {
                None
            }
        })
        .ok_or(WError::DataNotFound("constant inlineasm ty"))?;
    let has_side_effects = (data[1] & 1) != 0;
    let is_align_stack = (data[1] & 2) != 0;
    let asm_dialect = if (data[1] & 4) == 0 {
        ConstantInlineASMDialect::Att
    } else {
        ConstantInlineASMDialect::Intel
    };
    let can_throw = (data[1] & 8) != 0;

    let asm_size = data[2] as usize;
    let asm = data
        .drain(3..(3 + asm_size))
        .map(|v| v as u8 as char)
        .collect::<String>();
    if asm.len() != asm_size {
        return Err(WError::DataNotFound("constants inlineasm asm"));
    }
    let constr_size = data[3] as usize;
    let constr = data.drain(4..).map(|v| v as u8 as char).collect::<String>();
    if constr.len() != constr_size {
        return Err(WError::DataNotFound("constants inlineasm constr"));
    }
    Ok(Rc::new(ConstantInlineASM {
        ty,
        asm,
        constr,
        has_side_effects,
        is_align_stack,
        can_throw,
        asm_dialect,
    }))
}

#[derive(Debug, PartialEq, Clone)]
pub(in crate::bitcode) struct ConstantGep {
    pub(in crate::bitcode) ty: TypeBlockUnit,
    pub(in crate::bitcode) flag: InstrGEPNoWrapFlags,
    pub(in crate::bitcode) elts: Vec<(TypeBlockUnit, usize)>,
}

fn parse_constant_gep(
    data: Vec<u64>,
    _type_now: &TypeBlockData,
    data_types: &TypeBlock,
) -> WResult<ConstantGep> {
    if data.len() < 2 {
        return Err(WError::DataNotFound("constants gep"));
    }
    let ty = data_types
        .get(data[0] as usize)
        .cloned()
        .ok_or(WError::DataNotFound("constant gep ty"))?;
    let flag = InstrGEPNoWrapFlags::from(data[1] as usize);
    let mut elts = Vec::with_capacity(16);
    let mut data = data.into_iter().skip(2);
    while let Some(idx_ty) = data.next()
        && let Some(val) = data.next()
    {
        let ty_temp = data_types
            .get(idx_ty as usize)
            .cloned()
            .ok_or(WError::DataNotFound("constant gep elt ty"))?;
        elts.push((ty_temp, val as usize));
    }

    elts.shrink_to_fit();

    Ok(ConstantGep { ty, flag, elts })
}

const SET_TYPE: u32 = 1;

// 1. 一种value对应任意type
// 2. 一种value对应多个type，没有具体值，需要检查type
// 3/5. 一种value对应多个type，不同type有不同具体值，每个type都有子数据，不匹配子类型
// 4/6. 一种value对应多个type，不同type有不同具体值，每个type都有子数据，匹配子类型
// 7. 一种value对应多个type，有具体值，需要生成子数据，子数据就是string
// 8. 一种value对应一个type，有具体值，需要依赖type_list
enum_constants! {
    Undef = 3, // UNDEF
    Poison = 26, // POISON
    ;
    Null = (2, parse_constants_null), // NULL
    ;
    Integer = (4,
        Integer1, bool, parse_constants_integer1, TypeBlockData::Integer(1)
        Integer8, i8, parse_constants_integer8, TypeBlockData::Integer(2..=8)
        Integer16, i16, parse_constants_integer16, TypeBlockData::Integer(9..=16)
        Integer32, i32, parse_constants_integer32, TypeBlockData::Integer(17..=32)
        Integer64, i64, parse_constants_integer64, TypeBlockData::Integer(33..=64)
    ), // [intval]
    WideInteger = (5,
        Wide128, i128, parse_constants_integer128, TypeBlockData::Integer(65..=128)
    ), // [n x intval]
    ;
    Float = (6, Poison,
        F16, u16, parse_constants_f16, TypeBlockData::Half
        BF16, u16, parse_constants_f16, TypeBlockData::BFloat
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
    Data = (22,
        ArrayBool, Vec<bool>, parse_constants_vec_integer1,
        TypeBlockData::Array(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(1))

        VectorBool, Vec<bool>, parse_constants_vec_integer1,
        TypeBlockData::Vector(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(1))

        ArrayU8, Vec<i8>, parse_constants_vec_integer8,
        TypeBlockData::Array(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(2..=8))

        VectorU8, Vec<i8>, parse_constants_vec_integer8,
        TypeBlockData::Vector(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(2..=8))

        ArrayU16, Vec<i16>, parse_constants_vec_integer16,
        TypeBlockData::Array(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(9..=16))

        VectorU16, Vec<i16>, parse_constants_vec_integer16,
        TypeBlockData::Vector(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(9..=16))

        ArrayU32, Vec<i32>, parse_constants_vec_integer32,
        TypeBlockData::Array(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(17..=32))

        VectorU32, Vec<i32>, parse_constants_vec_integer32,
        TypeBlockData::Vector(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(17..=32))

        ArrayU64, Vec<i64>, parse_constants_vec_integer64,
        TypeBlockData::Array(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(33..=64))

        VectorU64, Vec<i64>, parse_constants_vec_integer64,
        TypeBlockData::Vector(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(33..=64))

        ArrayU128, Vec<i128>, parse_constants_vec_integer128,
        TypeBlockData::Array(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(65..=128))

        VectorU128, Vec<i128>, parse_constants_vec_integer128,
        TypeBlockData::Vector(v), matches!(v.ty.as_ref(), TypeBlockData::Integer(65..=128))

        ArrayF16, Vec<u16>, parse_constants_vec_f16,
        TypeBlockData::Array(v), matches!(v.ty.as_ref(), TypeBlockData::Half)

        VectorF16, Vec<u16>, parse_constants_vec_f16,
        TypeBlockData::Vector(v), matches!(v.ty.as_ref(), TypeBlockData::Half)

        ArrayBF16, Vec<u16>, parse_constants_vec_f16,
        TypeBlockData::Array(v), matches!(v.ty.as_ref(), TypeBlockData::BFloat)

        VectorBF16, Vec<u16>, parse_constants_vec_f16,
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
    ;
    Cast = (11, ConstantCast, parse_constant_cast) // [opcode, opty, opval]
    BinOp = (10, ConstantBinOP, parse_constant_binop) // [opcode, opty, opval]
    Inlineasm = (30, Rc<ConstantInlineASM>, parse_constant_inlineasm) // [fnty,sideeffect|alignstack|asmdialect|unwind,asmstr,conststr]
    Gep = (32, ConstantGep, parse_constant_gep) // [flags, n x operands]
}
// CeBinop = 10, // [opcode, opval, opval]
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
// CeGepWithInrange = 31, // [flags, range, n x operands]
// Ptrauth = 33, // [key, disc, addrdisc]
// Ptrauth2 = 34, // [key, disc, addrdisc, deactivation_symbol]

enum_constants! {
    "data_single_i128";
    Integer8,
    Integer16,
    Integer32,
    Integer64,
    Wide128,
}

enum_constants! {
    "data_single_string";
    String,
    CString,
}

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
            // eprintln!("constants block: {} {:?} {:?}", code, data, chars);
            if data.code == SET_TYPE {
                type_now = data.data.first()
                    .cloned()
                    .and_then(|idx| data_types.get(idx as usize).cloned())
                    .ok_or(WError::DataNotFound(
                        "No type found in parse constants block",
                    ))?;
                return Ok(());
            }
            if !data.blob.is_empty() {
                return Err(WError::DataNotAllowed(
                    "constant blob",
                    format!(
                        "code: {}, blob: {:?}, char: {}",
                        data.code, data.blob, data.chars
                    ),
                ));
            }
            let constant = parse_constants_block(
                data.code,
                data.data,
                data.chars,
                &type_now,
                data_types
            )?;
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

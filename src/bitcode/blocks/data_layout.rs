//! DataLayout

use std::{mem::replace, str::FromStr};

use crate::{WError, WResult, bitcode::parse_data_to_string};

struct CharsNext {
    split: char,
    data: Vec<char>,
}

impl CharsNext {
    fn new(data: Vec<char>, split: char) -> Self {
        Self { split, data }
    }

    fn left(self) -> Vec<char> {
        self.data
    }
}

impl Iterator for CharsNext {
    type Item = Vec<char>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.data.is_empty() {
            return None;
        }
        let idx = self
            .data
            .iter()
            .enumerate()
            .find(|&(_, v)| v == &self.split)
            .map(|v| v.0 + 1)
            .unwrap_or(self.data.len());
        let data_left = self.data.split_off(idx);
        let mut res = replace(&mut self.data, data_left);
        if !self.data.is_empty() {
            res.pop();
        }
        Some(res)
    }
}

macro_rules! enum_layout {
    ("mangling", $($name:ident = $value:expr)*) => {
        #[derive(Debug, Default)]
        pub(in crate::bitcode) enum DataLayoutMangling {
            $(
                $name,
            )*
            #[default]
            None
        }


        impl DataLayoutMangling {
            fn parse(chars: Vec<char>) -> WResult<Self> {
                if chars.len() != 2 {
                    return Err(WError::DataNotAllowed("mangling", String::from_iter(chars)));
                }
                let r = match (chars[0], chars[1]) {
                    $(
                        (':', $value) => Self::$name,
                    )*
                    _ => return Err(WError::DataNotAllowed(
                        "mangling",
                        String::from_iter(chars)
                    ))
                };
                Ok(r)
            }
        }

    };
}

enum_layout! {
    "mangling",
    Elf = 'e'
    Goff = 'l'
    MachO = 'o'
    Mips = 'm'
    WinCoff = 'w'
    WinCoffX86 = 'x'
    XCoff = 'a'
}

#[derive(Debug, Default)]
pub(in crate::bitcode) enum DataLayoutFunctionPtrAlignType {
    #[default]
    Independent,
    MultipleOfFunctionAlign,
}

fn parse_integer<T: FromStr>(chars: Vec<char>) -> WResult<T> {
    let t = String::from_iter(chars);
    // eprintln!("t: {}", t);
    t.parse::<T>()
        .ok()
        .ok_or(WError::DataNotAllowed("integer", t))
}

fn parse_integer_width(chars: Vec<char>) -> WResult<Vec<u16>> {
    CharsNext::new(chars, ':')
        .map(parse_integer::<u16>)
        .collect::<WResult<Vec<_>>>()
}

fn parse_alignment(chars: Vec<char>, allow_zero: bool) -> WResult<u16> {
    let align = parse_integer::<u16>(chars)?;
    if align == 0 {
        if !allow_zero {
            return Err(WError::DataNotAllowed("alignment", "align zero".to_owned()));
        }
        return Ok(1);
    }
    if (align & 7) > 0 {
        return Err(WError::DataNotAllowed(
            "alignment",
            format!("align {}", align),
        ));
    }
    Ok(align >> 3)
}

#[derive(Debug, Default)]
pub(super) struct DataLayoutPrimitive {
    pub(super) bit_width: u16,
    pub(super) align_abi: u16,
    pub(super) align_pref: Option<u16>,
}

fn parse_primitive(
    chars: Vec<char>,
    allow_zero: bool,
) -> WResult<(DataLayoutPrimitive, Option<Vec<char>>)> {
    let mut chars = CharsNext::new(chars, ':');
    let bit_width = {
        let bit_width = chars
            .next()
            .ok_or(WError::DataNotFound("primitives bit_width"))?;
        if bit_width.is_empty() {
            0
        } else {
            parse_integer::<u16>(bit_width)?
        }
    };
    let align_abi = {
        let align_abi = chars
            .next()
            .ok_or(WError::DataNotFound("primitives align"))?;
        parse_alignment(align_abi, false)?
    };
    let align_pref = if let Some(align_pref) = chars.next() {
        let align_pref = parse_alignment(align_pref, allow_zero)?;
        if align_pref < align_abi {
            return Err(WError::DataNotAllowed(
                "primitives",
                format!(
                    "preferred alignment ({align_pref}) cannot be less than the ABI alignment ({align_abi})"
                ),
            ));
        }
        Some(align_pref)
    } else {
        None
    };
    let next = chars.next();
    Ok((
        DataLayoutPrimitive {
            bit_width,
            align_abi,
            align_pref,
        },
        next,
    ))
}

#[derive(Debug)]
pub(super) struct DataLayoutPointer {
    pub(super) address_space: u16,
    pub(super) external_state: bool,
    pub(super) unstable_repr: bool,
    pub(super) bit_width: u16,
    pub(super) align_abi: u16,
    pub(super) align_pref: Option<u16>,
    pub(super) index_bit_width: u16,
}

fn parse_pointer(chars: Vec<char>) -> WResult<DataLayoutPointer> {
    let mut chars = CharsNext::new(chars, ':');
    let mut address_space = 0;
    let mut external_state = false;
    let mut unstable_repr = false;
    let mut chars_address_space = chars
        .next()
        .ok_or(WError::DataNotFound("pointer address_space"))?;
    chars_address_space.reverse();
    while let Some(front) = chars_address_space.pop() {
        if front == 'e' {
            external_state = true;
        } else if front == 'u' {
            unstable_repr = true;
        } else if front.is_ascii_digit() {
            break;
        } else {
            chars_address_space.reverse();
            return Err(WError::DataNotAllowed(
                "pointer address_space error flag",
                String::from_iter(chars_address_space),
            ));
        };
    }
    if !chars_address_space.is_empty() {
        chars_address_space.reverse();
        address_space = parse_integer(chars_address_space)?;
    }
    if address_space == 0 && (external_state || unstable_repr) {
        return Err(WError::DataNotAllowed(
            "pointer address_space error state",
            format!(
                "address_space: {address_space}, external_state: {external_state}, unstable_repr: {unstable_repr}"
            ),
        ));
    }
    let chars = chars.left();
    let (primitives, chars_left) = parse_primitive(chars, false)?;
    let index_bit_width = if let Some(chars) = chars_left {
        parse_integer(chars)?
    } else {
        primitives.bit_width
    };
    Ok(DataLayoutPointer {
        address_space,
        external_state,
        unstable_repr,
        bit_width: primitives.bit_width,
        align_abi: primitives.align_abi,
        align_pref: primitives.align_pref,
        index_bit_width,
    })
}

fn parse_ni(mut chars: Vec<char>) -> WResult<Vec<u16>> {
    if chars.len() < 2 {
        return Err(WError::DataNotFound("ni"));
    }
    let f = chars.remove(0);
    if f != ':' {
        return Err(WError::DataNotAllowed("ni", String::from_iter(chars)));
    }
    parse_integer_width(chars)
}

const CHARS_NI: [char; 2] = ['n', 'i'];

#[derive(Debug, Default)]
pub(in crate::bitcode) struct DataLayout {
    pub(in crate::bitcode) src: String,
    pub(super) mangling: DataLayoutMangling,
    pub(super) big_endian: bool,
    pub(super) align_stack: u16,
    pub(super) align_type_function: DataLayoutFunctionPtrAlignType,
    pub(super) align_function: u16,
    pub(super) address_space_function: u16,
    pub(super) address_space_stack_alloca: u16,
    pub(super) address_space_global_var: u16,
    pub(super) legal_int_width: Vec<u16>,
    pub(super) pointers: Vec<DataLayoutPointer>,
    pub(super) primitive_integer: Vec<DataLayoutPrimitive>,
    pub(super) primitive_float: Vec<DataLayoutPrimitive>,
    pub(super) primitive_vector: Vec<DataLayoutPrimitive>,
    pub(super) primitive_aggregate: Vec<DataLayoutPrimitive>,
}

pub(super) fn parse_data_layout(data: Vec<u64>) -> WResult<DataLayout> {
    let data = data.into_iter().map(|v| v as u8).collect::<Vec<_>>();
    let src = parse_data_to_string(data.clone().into_iter());
    let datas = CharsNext::new(src.chars().collect(), '-');
    let mut mangling = DataLayoutMangling::None;
    let mut big_endian = false;
    let mut align_stack = 0;
    let mut align_type_function = DataLayoutFunctionPtrAlignType::Independent;
    let mut align_function = 0;
    let mut address_space_function = 0;
    let mut address_space_stack_alloca = 0;
    let mut address_space_global_var = 0;
    let mut legal_int_width = Vec::with_capacity(8);
    let mut pointers = Vec::with_capacity(8);
    let mut primitive_integer = Vec::with_capacity(8);
    let mut primitive_float = Vec::with_capacity(8);
    let mut primitive_vector = Vec::with_capacity(8);
    let mut primitive_aggregate = Vec::with_capacity(8);

    let mut address_space_non_int = Vec::with_capacity(8);

    for mut data_now in datas {
        // eprintln!("data_now: {:?}", data_now);
        if data_now.starts_with(&CHARS_NI) {
            let data_now = data_now.split_off(2);
            let t = parse_ni(data_now)?;
            address_space_non_int.extend(t);
        } else {
            let data_now_rest = data_now.split_off(1);
            let data_front = data_now[0];
            match data_front {
                'i' | 'f' | 'v' => {
                    let (t, _) = parse_primitive(data_now_rest, false)?;
                    if data_front == 'i' && t.bit_width == 8 && t.align_abi != 1 {
                        return Err(WError::DataNotAllowed(
                            "primitives",
                            format!("i8 must be 8-bit aligned: {}", t.align_abi),
                        ));
                    }
                    if data_front == 'i' {
                        primitive_integer.push(t);
                    } else if data_front == 'f' {
                        primitive_float.push(t);
                    } else {
                        primitive_vector.push(t);
                    }
                }
                'a' => {
                    let (t, _) = parse_primitive(data_now_rest, true)?;
                    if t.bit_width != 0 {
                        return Err(WError::DataNotAllowed(
                            "aggregate",
                            format!("size must be zero: {}", t.bit_width),
                        ));
                    }
                    primitive_aggregate.push(t);
                }
                'p' => {
                    let t = parse_pointer(data_now_rest)?;
                    pointers.push(t);
                }
                's' => {
                    return Err(WError::Deprecated("data layout for asm file"));
                }
                'e' | 'E' => {
                    if !data_now_rest.is_empty() {
                        return Err(WError::Num("data layout e/E", true, data_now_rest.len(), 1));
                    }
                    big_endian = data_front == 'E';
                }
                'n' => {
                    let vs = parse_integer_width(data_now_rest)?;
                    legal_int_width.extend(vs);
                }
                'S' => {
                    align_stack = parse_alignment(data_now_rest, false)?;
                }
                'F' => {
                    if data_now_rest.len() < 2 {
                        return Err(WError::DataNotFound("function ptr align"));
                    }
                    let mut data_now_rest = data_now_rest;
                    let f = data_now_rest.remove(0);
                    align_function = parse_alignment(data_now_rest, false)?;
                    match f {
                        'i' => {
                            align_type_function = DataLayoutFunctionPtrAlignType::Independent;
                        }
                        'n' => {
                            align_type_function =
                                DataLayoutFunctionPtrAlignType::MultipleOfFunctionAlign;
                        }
                        _ => {
                            return Err(WError::CodeNotAllowed(
                                "function ptr align type",
                                f as u32,
                            ));
                        }
                    }
                }
                'P' => {
                    address_space_function = parse_integer::<u16>(data_now_rest)?;
                }
                'A' => {
                    address_space_stack_alloca = parse_integer::<u16>(data_now_rest)?;
                }
                'G' => {
                    address_space_global_var = parse_integer::<u16>(data_now_rest)?;
                }
                'm' => {
                    mangling = DataLayoutMangling::parse(data_now_rest)?;
                }
                i => return Err(WError::CodeNotAllowed("data_layout", i as u32)),
            }
        }
    }

    for i in address_space_non_int {
        let p = pointers.iter_mut().find(|v| v.address_space == i);
        if let Some(p) = p {
            p.unstable_repr = true;
            p.external_state = false;
        }
    }

    Ok(DataLayout {
        src,
        mangling,
        big_endian,
        align_stack,
        align_type_function,
        align_function,
        address_space_function,
        address_space_stack_alloca,
        address_space_global_var,
        legal_int_width,
        pointers,
        primitive_integer,
        primitive_float,
        primitive_vector,
        primitive_aggregate,
    })
}

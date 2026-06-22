//! FileBit utils

use std::fmt::{Binary, Debug, Display};
use std::ops::{AddAssign, BitAnd, BitXorAssign, Shl, ShlAssign, Sub};

use crate::{WError, WResult};

pub(super) fn bit_align_check(n: usize) -> WResult<()> {
    let align_remainder = n & 7;
    if align_remainder != 0 {
        return Err(WError::BitAlign(8, align_remainder));
    }
    Ok(())
}

pub(super) static CHARACTERS_6BIT: [char; 64] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9', '.', '_',
];

pub trait BitType
where
    Self: Sized
        + Debug
        + Copy
        + From<u8>
        + Display
        + Binary
        + PartialEq
        + AddAssign<Self>
        + Shl<usize, Output = Self>
        + Sub<Self, Output = Self>
        + BitAnd<Self, Output = Self>
        + ShlAssign<usize>
        + BitXorAssign<Self>,
{
    fn parse_data_inner(data: Vec<u8>) -> Self;
    fn into_u8(self) -> u8;

    fn parse_data(mut data: Vec<u8>) -> Self {
        let n_need = size_of::<Self>();
        let n_have = data.len();
        if n_need > n_have {
            data.extend(vec![0u8; n_need - n_have]);
        } else if n_need < n_have {
            data.truncate(n_need);
        }
        Self::parse_data_inner(data)
    }
}

macro_rules! bit_parse {
    ($($ident:ident,)*) => {
        $(
            impl BitType for $ident {
                fn parse_data_inner(data: Vec<u8>) -> Self {
                    let data: [u8; _] = data.try_into().unwrap();
                    Self::from_le_bytes(data)
                }

                fn into_u8(self) -> u8 {
                    self as u8
                }
            }
        )*
    };
}

bit_parse! {u8, u16, i16, u32, i32, u64, i64, u128, i128, usize, }

//! BitCode

macro_rules! enum_reader {
    ("inner", $doc:expr, $vis:vis, $name:ident; $($derives:ident,)*; $($key:ident = $value:expr,)*; $($func:item,)*) => {
        #[doc = $doc]
        #[derive($($derives,)*)]
        #[repr(u8)]
        $vis enum $name {
            $(
                $key,
            )*
        }

        impl $name {
            $(
                $func
            )*
        }
    };


    ("inner", $doc:expr, $vis:vis, $name:ident, $default:ident; $($derives:ident,)*; $($key:ident = $value:expr,)*; $($func:item,)*) => {
        #[doc = $doc]
        #[derive($($derives,)*)]
        #[repr(u8)]
        $vis enum $name {
            $(
                $key = $value,
            )*
            $default(u8)
        }

        impl $name {
            $(
                $func
            )*
        }
    };


    (#[doc = $doc:expr] $name:ident, $method:ident, $len:expr, $vis0:vis, $vis:vis, $default:ident; $($derives:ident,)*; $($key:ident = $value:expr,)*) => {
        enum_reader!{
            "inner", $doc, $vis0, $name, $default;
            $($derives,)*;
            $($key = $value,)*;
            $vis fn parse<F: Read + Seek>(reader: &mut BitReader<F>) -> WResult<Self> {
                let code = reader.$method::<u8>($len)?;
                let res = match code {
                    $(
                        $value => Self::$key,
                    )*
                    _ => Self::$default(code),
                };
                Ok(res)
            },
        }

        impl From<$name> for usize {
            fn from(value: $name) -> Self {
                let r = match value {
                    $(
                        $name::$key => $value,
                    )*
                    $name::$default(v) => v
                };
                r as usize
            }
        }
    };

    (#[doc = $doc:expr] $name:ident, $default:ident; $($derives:ident,)*; $($key:ident = $value:expr,)*) => {
        #[doc = $doc]
        #[derive($($derives,)*)]
        enum $name {
            $(
                $key,
            )*
            $default(usize)
        }


        impl From<usize> for $name {
            fn from(value: usize) -> Self {
                match value {
                    $(
                        $value => Self::$key,
                    )*
                    _ => Self::$default(value),
                }
            }
        }

        impl $name {
            fn parse_with_data<F: Read + Seek>(reader: &mut BitReader<F>, state: &usize) -> WResult<Self> {
                let code = reader.fixed_width_integers::<usize>(*state)?;
                let res = Self::from(code);
                Ok(res)
            }
        }
    };


    (#[doc = $doc:expr] $name:ident, $method:ident, $len:expr; $($derives:ident,)*; $($key:ident = $value:expr,)*) => {
        enum_reader!{
            "inner", $doc, pub(super), $name;
            $($derives,)*;
            $($key = $value,)*;
            fn parse<F: Read + Seek>(reader: &mut BitReader<F>) -> WResult<Self> {
                let code = reader.$method::<u8>($len)?;
                let res = match code {
                    $(
                        $value => Self::$key,
                    )*
                    _ => return Err(WError::CodeNotAllowed(
                        stringify!($name),
                        code as u32
                    ))
                };
                Ok(res)
            },
        }
    };

    (#[doc = $doc:expr] $name:ident, $method:ident, $len:expr; $($derives:ident,)*; $($key:ident = $value:expr,)*; $parse_with_state:item) => {
        enum_reader!{
            "inner", $doc, pub(super), $name, $state;
            $($derives,)*;
            $($key = $value,)*;
            fn parse<F: Read + Seek>(reader: &mut BitReader<F>) -> WResult<Self> {
                let code = reader.$method::<u8>($len)?;
                let res = match code {
                    $(
                        $value => Self::$key,
                    )*
                    _ => return Err(WError::CodeNotAllowed(
                        stringify!($name),
                        code as u32
                    ))
                };
                Ok(res)
            },
            $parse_with_state,
        }
    };
}

mod abbrev;
mod blocks;
mod convert;
mod entry;
mod parse;

pub use abbrev::{AbbrevOp, BlockIDs};
pub use parse::BitCode;

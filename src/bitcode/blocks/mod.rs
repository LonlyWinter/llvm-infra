//! blocks

macro_rules! enum_parse {
    (
        $vis:vis $name:ident, $ty:ty;
        $($derives:ident,)*;
        $($key1:ident = $value1:pat,)*;
        $($key2:ident = ($related2:ty; $default:expr; $value2:pat),)*
    ) => {
        #[derive($($derives,)*)]
        #[repr(u8)]
        $vis enum $name {
            $(
                $key1,
            )*
            $(
                $key2($related2),
            )*
        }


        impl TryFrom<$ty> for $name {
            type Error = WError;
            fn try_from(value: $ty) -> WResult<Self> {
                match value {
                    $(
                        $value1 => Ok(Self::$key1),
                    )*
                    $(
                        $value2 => Ok(Self::$key2($default)),
                    )*
                    _ => Err(WError::DataNotAllowed(
                        stringify!($name),
                        format!("{:?}", value)
                    ))
                }
            }
        }
    };

    (
        $vis:vis $name:ident, $ty:ty;
        $($derives:ident,)*;
        $($key:ident = $value:pat,)*
    ) => {
        enum_parse!{
            $vis $name, $ty;
            $($derives,)*;
            $($key = $value,)*
            ;
        }
    };

    (
        $vis:vis $name:ident, $ty:ty, $parent:literal;
        $($derives:ident,)*;
        $($key:ident = $value:pat = $display:literal,)*
    ) => {
        enum_parse!{
            $vis $name, $ty;
            $($derives,)*;
            $($key = $value,)*
            ;
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$key => write!(f, "{}({})", $parent, $display),
                    )*
                }
            }
        }
    };

    (
        $vis:vis $name:ident, $ty:ty;
        $($derives:ident,)*;
        $($key:ident = $value:pat = $display:literal,)*
    ) => {
        enum_parse!{
            $vis $name, $ty;
            $($derives,)*;
            $($key = $value,)*
            ;
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$key => write!(f, $display),
                    )*
                }
            }
        }
    };
}

pub(super) mod app_spec;
pub(super) mod blockinfo;
pub(super) mod constants;
pub(super) mod data_layout;
pub(super) mod function_block;
pub(super) mod function_record;
pub(super) mod globalvar;
pub(super) mod identification;
pub(super) mod metadata;
pub(super) mod metadata_attachment;
pub(super) mod metadata_kind;
pub(super) mod moduleblock;
pub(super) mod operand_bundle_tags;
pub(super) mod param_attr;
pub(super) mod param_attr_group;
pub(super) mod str_sym_table;
pub(super) mod sync_scope_names;
pub(super) mod typeblock;
pub(super) mod uselist;
pub(super) mod value_symtab;

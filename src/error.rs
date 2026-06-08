//! 错误处理

use std::{
    fmt::{Debug, Display},
    io::Error as IOError,
};

use crate::{AbbrevOp, BlockIDs};

pub enum WError {
    UnImplemented(&'static str),

    /// NumError: Greater(true, num_offer > num_need), LessThan(false, num_offer <= num_need)
    Num(&'static str, bool, usize, usize),

    /// BitAlignError: align_size, remainder(normal = 0)
    BitAlign(usize, usize),
    /// file empty
    FileEmpty,
    /// file extension error
    FileExtNeed(&'static str),
    /// file read error
    FileRead(String),
    /// file write posi error: need, max
    FileWritePosi(u64, u64),

    /// Abbreviation error
    AbbrevType(AbbrevOp),
    /// AbbreviationLiteralNum > 128
    AbbrevLiteralNum(u32),
    /// AbbreviationOpEmpty
    AbbrevOpEmpty,
    /// Array element type can'not be an array or blob or none
    AbbrevOpArrayElement,

    /// CodeNotAllowed
    CodeNotAllowed(&'static str, u32),
    /// DataNotAllowed
    DataNotAllowed(&'static str, String),
    /// No more data
    DataNotFound(&'static str),
    /// BlockNotAllowed
    BlockNotAllowed(&'static str, BlockIDs),
    /// BlockNotFound: no data for block_size
    BlockNotFound(usize),
    /// block_size = 0
    BlockEmpty,
    /// already set data
    DataSetAlready(&'static str),

    /// MagicNumber
    MagicNumber(Vec<u8>),
    /// Version: need 0
    Version(Option<u8>),
    /// Deprecated
    Deprecated(&'static str),
    /// State empty
    StateEmpty,
}

pub type WResult<T> = Result<T, WError>;

impl From<IOError> for WError {
    fn from(value: IOError) -> Self {
        Self::FileRead(value.to_string())
    }
}

impl Display for WError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Num(info, true, n_offer, n_need) => {
                write!(f, "NumError: {info}, max {n_need} but given {n_offer}",)
            }
            Self::Num(info, false, n_offer, n_need) => {
                write!(f, "NumError: {info}, min {n_need} but given {n_offer}",)
            }
            Self::BitAlign(align, remainder) => write!(
                f,
                "BitAlignError: need align {align} bit(s), but remainder {remainder}"
            ),
            Self::FileEmpty => write!(f, "FileNoMoreData"),
            Self::FileRead(err) => write!(f, "FileReadeError: {}", err),
            Self::FileWritePosi(posi_need, posi_max) => write!(
                f,
                "FileWritePositionError: need {}, but max {}",
                posi_need, posi_max
            ),
            Self::AbbrevType(t) => write!(f, "AbbrevTypeError: {t:?}",),
            Self::AbbrevLiteralNum(n) => write!(f, "AbbrevLiteralTooLarge: {n}, need < 128"),
            Self::AbbrevOpEmpty => write!(f, "AbbrevOpEmpty"),
            Self::AbbrevOpArrayElement => {
                write!(f, "Array element type can'not be an array or blob or none")
            }
            Self::CodeNotAllowed(info, code) => write!(f, "Code {code} not allowed in {info}"),
            Self::DataNotAllowed(info, data) => write!(f, "Data {data} not allowed in {info}"),
            Self::DataNotFound(info) => write!(f, "No Data for {info}"),
            Self::BlockNotAllowed(info, id) => write!(f, "Block {id:?} not allow in {info}"),
            Self::BlockNotFound(size) => write!(f, "Block not found with size {size}"),
            Self::BlockEmpty => write!(f, "Block size 0"),
            Self::DataSetAlready(info) => write!(f, "{info} has already set"),
            Self::MagicNumber(magic) => write!(
                f,
                "Error magic: [{}]",
                magic
                    .iter()
                    .map(|v| format!("{v:#X}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Version(ver) => write!(f, "Error version: {ver:?}"),
            Self::Deprecated(info) => write!(f, "Deprecated: {info}"),
            Self::StateEmpty => write!(f, "Parse state empty"),
            Self::FileExtNeed(ext) => write!(f, "Error file extension, need {ext:?}"),
            Self::UnImplemented(info) => write!(f, "UnImplemented {info:?}"),
        }
    }
}

impl Debug for WError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

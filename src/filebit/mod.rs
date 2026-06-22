//! FileBit Reader/Writer

mod filevec;
mod reader;
mod utils;
mod writer;

pub use filevec::{BitReaderFileVec, BitWriterFileVec, FileVec};
pub use reader::{
    BitReader, decoded_signed_vbrs_u8, decoded_signed_vbrs_u16, decoded_signed_vbrs_u32, decoded_signed_vbrs_u64,
};
pub use utils::BitType;
pub use writer::BitWriter;

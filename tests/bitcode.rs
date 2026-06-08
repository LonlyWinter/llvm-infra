//! test bitcode

use llvm_infra::bitcode::BitCode;
use llvm_infra::{LLVM, WResult};

#[test]
fn test_parse_bitcode_aes128() -> WResult<()> {
    let _ = BitCode::from_file("data/aes128.bc")?;
    Ok(())
}

#[test]
fn test_convert_aes128() -> WResult<()> {
    let _ = LLVM::from_file_bc("data/aes128.bc")?;
    Ok(())
}

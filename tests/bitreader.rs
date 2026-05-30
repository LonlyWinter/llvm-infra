//! test bitreader

use llvm_infra::WResult;
use llvm_infra::bitreader::BitReaderFileVec;

#[test]
fn test_variable_width_integers() -> WResult<()> {
    let mut r = BitReaderFileVec::from_vec(vec![0b0011_1110; 8]);
    let n = r.variable_width_integers::<u32>(4)?;
    assert_eq!(n, 30, "variable_width_integers1: {:b}", n);
    let n = r.variable_width_integers::<u32>(4)?;
    assert_eq!(n, 30, "variable_width_integers2: {:b}", n);
    Ok(())
}

#[test]
fn test_fixed_width_integers() -> WResult<()> {
    let mut r = BitReaderFileVec::from_vec(vec![0b0011_1110; 8]);
    let n = r.fixed_width_integers::<u32>(5)?;
    assert_eq!(n, 30, "fixed_width_integers1: {:b}", n);
    let n = r.fixed_width_integers::<u32>(4)?;
    assert_eq!(n, 1, "fixed_width_integers2: {:b}", n);
    let n = r.fixed_width_integers::<u32>(3)?;
    assert_eq!(n, 7, "fixed_width_integers3: {:b}", n);
    let n = r.fixed_width_integers::<u32>(10)?;
    assert_eq!(n, 995, "fixed_width_integers4: {:b}", n);
    Ok(())
}

#[test]
fn test_word_alignment() -> WResult<()> {
    let mut r = BitReaderFileVec::from_vec(vec![0b0011_1110; 8]);
    let n = r.fixed_width_integers::<u32>(5)?;
    assert_eq!(n, 30, "word_alighment1: {:b}", n);
    r.word_alignment(12)?;
    let n = r.fixed_width_integers::<u32>(10)?;
    assert_eq!(n, 995, "word_alighment2: {:b}", n);
    Ok(())
}

#[test]
fn test_signed_vbr() -> WResult<()> {
    let mut r = BitReaderFileVec::from_vec(vec![
        0b0111_0101,
        0b0011_0001,
        0b1100_0000,
        0b1000_1111,
        0b1110_0000,
        0b1110_0000,
        0b1110_0000,
        0b1110_0000,
    ]);
    let n = r.signed_vbrs::<i32>(4)?;
    assert_eq!(n, -3, "signed_vbrs1: {:b}", n);
    let n = r.signed_vbrs::<i32>(4)?;
    assert_eq!(n, -4, "signed_vbrs2: {:b}", n);
    let n = r.signed_vbrs::<i32>(4)?;
    assert_eq!(n, -1, "signed_vbrs3: {:b}", n);
    let n = r.signed_vbrs::<i32>(4)?;
    assert_eq!(n, -2, "signed_vbrs4: {:b}", n);
    Ok(())
}

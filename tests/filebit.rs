//! test filebit

use llvm_infra::WResult;
use llvm_infra::filebit::{BitReaderFileVec, BitWriterFileVec, decoded_signed_vbrs_u32};

#[test]
fn test_reader_variable_width_integers() -> WResult<()> {
    let mut r = BitReaderFileVec::from_vec(vec![0b0011_1110; 8]);
    let n = r.variable_width_integers::<u32>(4)?;
    assert_eq!(n, 30, "variable_width_integers1: {:b}", n);
    let n = r.variable_width_integers::<u32>(4)?;
    assert_eq!(n, 30, "variable_width_integers2: {:b}", n);
    Ok(())
}

#[test]
fn test_reader_fixed_width_integers() -> WResult<()> {
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
fn test_reader_word_alignment() -> WResult<()> {
    let mut r = BitReaderFileVec::from_vec(vec![0b0011_1110; 8]);
    let n = r.fixed_width_integers::<u32>(5)?;
    assert_eq!(n, 30, "word_alighment1: {:b}", n);
    r.word_alignment(12)?;
    let n = r.fixed_width_integers::<u32>(10)?;
    assert_eq!(n, 995, "word_alighment2: {:b}", n);
    Ok(())
}

#[test]
fn test_reader_signed_vbr_func() -> WResult<()> {
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
    let n = r.variable_width_integers::<u32>(4)?;
    let n = decoded_signed_vbrs_u32(n) as i32;
    assert_eq!(n, -2, "signed_vbrs1: {:b}", n);
    let n = r.variable_width_integers::<u32>(4)?;
    let n = decoded_signed_vbrs_u32(n) as i32;
    assert_eq!(n, -3, "signed_vbrs2: {:b}", n);
    let n = r.variable_width_integers::<u32>(4)?;
    let n = decoded_signed_vbrs_u32(n) as i32;
    assert_eq!(n, -2147483648, "signed_vbrs3: {:b}", n);
    let n = r.variable_width_integers::<u32>(4)?;
    let n = decoded_signed_vbrs_u32(n) as i32;
    assert_eq!(n, -1, "signed_vbrs4: {:b}", n);
    Ok(())
}

#[test]
fn test_writer_write() -> WResult<()> {
    let mut w = BitWriterFileVec::with_capacity(8);
    let data = vec![
        0b0111_0101u8,
        0b0000_0001,
        0b0000_1110,
        0b0011_1110,
        0b1100_0000,
        0b1000_1111,
        0b1110_0000,
        0b1110_0000,
        0b1110_0000,
        0b1100_0000,
        0b1000_1111,
        0b1110_0000,
    ];
    w.write_data(2, &[0b01])?;
    w.write_data(4, &[0b1101])?;
    w.write_data(6, &[0b00_0101])?;
    w.write_data(16, &[0b1110_0000, 0b1110_0000])?;
    w.write_data(8, &[0b0000_0011])?;
    w.write_data(12, &[0b1111_1100, 0b1000])?;
    w.write_data(24, &[0b1110_0000, 0b1110_0000, 0b1110_0000])?;
    w.write_data(24, &[0b1100_0000, 0b1000_1111, 0b1110_0000])?;
    let data_write = w.inner()?.data();
    assert_eq!(data_write, data, "write");
    Ok(())
}

#[test]
fn test_writer_alignment() -> WResult<()> {
    let mut w = BitWriterFileVec::with_capacity(8);
    let data = vec![
        0b0111_0101u8,
        0b0000_0001,
        0b0000_1110,
        0b0000_1110,
        0b0000_0011,
        0b1111_1100,
        0b0000_1000,
        0b1110_0000,
        0b1100_0000,
        0b1110_0000,
        0b1100_0000,
        0b1000_1111,
        0b1110_0000,
    ];
    w.write_data(2, &[0b01])?;
    w.write_data(4, &[0b1101])?;
    w.write_data(6, &[0b00_0101])?;
    w.write_data(16, &[0b1110_0000, 0b1110_0000])?;
    w.bit_alignment(8)?;
    w.write_data(8, &[0b0000_0011])?;
    w.write_data(12, &[0b1111_1100, 0b1000])?;
    w.bit_alignment(8)?;
    w.write_data(24, &[0b1110_0000, 0b1100_0000, 0b1110_0000])?;
    w.write_data(24, &[0b1100_0000, 0b1000_1111, 0b1110_0000])?;
    let data_write = w.inner()?.data();
    assert_eq!(data_write, data, "write");
    Ok(())
}


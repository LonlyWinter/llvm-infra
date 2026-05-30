//! Abbrev

use std::{
    io::{Read, Seek},
    rc::Rc,
};

use crate::{WError, WResult, bitcode::parse::StateBlock};

use crate::{
    bitcode::entry::MAX_CHUNK_SIZE,
    bitreader::{BitReader, BitType},
};

const BLOCK_ID_WIDTH: usize = 8;

enum_reader! {
    /// https://llvm.org/docs/BitCodeFormat.html#abbreviation-ids
    AbbrevOpEncoding, fixed_width_integers, 3;
    Debug, Clone,;
    Fixed = 1,
    Vbr = 2,
    Array = 3,
    Char6 = 4,
    Blob = 5,
}

/// https://llvm.org/docs/BitCodeFormat.html#define-abbrev-encoding
#[derive(Debug, Clone)]
pub enum AbbrevOp {
    Literal(u32),
    Fixed(usize),
    Vbr(usize),
    Array,
    Char6,
    Blob,
}

pub(super) type Abbrev = Rc<Vec<AbbrevOp>>;

enum_reader! {
    /// https://llvm.org/docs/BitCodeFormat.html#llvm-ir-blocks
    BlockIDs, variable_width_integers, BLOCK_ID_WIDTH, pub, pub(super), AppSpec;
    PartialEq, Debug, Clone, ;
    BlockInfo = 0,
    ModuleBlock = 8,
    ParamAttrBlock = 9,
    ParamAttrGroupBlock = 10,
    ConstantsBlock = 11,
    FunctionBlock = 12,
    IdentificationBlock = 13,
    ValueSymtabBlock = 14,
    MetaDataBlock = 15,
    MetaDataAttachment = 16,
    TypeBlock = 17,
    UselistBlock = 18,
    ModuleStrtabBlock = 19,
    GlobalvalSummaryBlock = 20,
    OperandBundleTagsBlock = 21,
    MetaDataKindBlock = 22,
    StrtabBlock = 23,
    FullLtoGlobalvalSummaryBlock = 24,
    SymtabBlock = 25,
    SyncScopeNamesBlock = 26,
}

pub(super) enum UnabbrevRecordData<T: BitType> {
    Data(T),
    Char(char),
}

impl<F: Read + Seek> BitReader<F> {
    pub(super) fn read_abbreviated_field_code(&mut self, op: &AbbrevOp) -> WResult<u32> {
        match op {
            AbbrevOp::Fixed(n) => self.fixed_width_integers(*n),
            AbbrevOp::Vbr(n) => self.variable_width_integers(*n),
            AbbrevOp::Literal(v) => Ok(*v),
            _ => Err(WError::AbbrevType(op.clone())),
        }
    }

    pub(super) fn read_abbreviated_field<T: BitType>(
        &mut self,
        op: &AbbrevOp,
    ) -> WResult<UnabbrevRecordData<T>> {
        match op {
            AbbrevOp::Fixed(n) => {
                let data = self.fixed_width_integers::<T>(*n)?;
                Ok(UnabbrevRecordData::Data(data))
            }
            AbbrevOp::Vbr(n) => {
                let data = self.variable_width_integers(*n)?;
                Ok(UnabbrevRecordData::Data(data))
            }
            AbbrevOp::Char6 => {
                let r = self.characters_6bit()?;
                Ok(UnabbrevRecordData::Char(r))
            }
            AbbrevOp::Literal(v) => {
                let v = *v;
                if v > 127 {
                    return Err(WError::AbbrevLiteralNum(v));
                }
                Ok(UnabbrevRecordData::Data(T::from(v as u8)))
            }
            _ => Err(WError::AbbrevType(op.clone())),
        }
    }

    // pub(super) fn skip_record_abbrev(&mut self, bitcode: &BitCode, abbrev_id: usize) -> WResult<usize> {
    //     let abbrev = bitcode.state.last()
    //         .and_then(| v | v.abbrevs
    //             .get(abbrev_id))
    //         .ok_or(WError::Parse(format!("No abbrev found in skip_abbrev_record: {}", abbrev_id)))?;
    //     let code_op = abbrev
    //         .first()
    //         .ok_or(WError::Parse(format!("No code op found in abbrev: {}", abbrev.len())))?;
    //     let code = self.read_abbreviated_field_code(code_op)?;
    //     for ops in abbrev.windows(2).skip(1) {
    //         let op = &ops[0];
    //         let r = self.read_abbreviated_field(op);
    //         if r.is_ok() {
    //             continue;
    //         }
    //         let num_elements = self.variable_width_integers::<usize>(6)?;
    //         if matches!(op, AbbrevOp::Array) {
    //             match ops[1] {
    //                 AbbrevOp::Fixed(n) => {
    //                     let n_skip = num_elements + n;
    //                     self.skip_bits(n_skip)?;
    //                 },
    //                 AbbrevOp::Vbr(n) => {
    //                     for _ in 0..num_elements {
    //                         let _res = self.variable_width_integers::<u64>(n)?;
    //                     }
    //                 },
    //                 AbbrevOp::Char6 => {
    //                     let n_skip = num_elements * 6;
    //                     self.skip_bits(n_skip)?;
    //                 },
    //                 _ => {
    //                     return Err(WError::Parse("Array element type can'not be an array or blob".to_owned()));
    //                 }
    //             }
    //             continue;
    //         }

    //         // blob
    //         self.word_alignment(32)?;
    //         let n_skip = (num_elements & (!4)) << 3;
    //         if self.skip_bits(n_skip).is_err() {
    //             self.skip_end()?;
    //         }
    //     }
    //     Ok(code)
    // }

    /// https://llvm.org/docs/BitCodeFormat.html#define-abbrev-encoding
    pub(super) fn read_abbrev_record(&mut self, state: &mut StateBlock) -> WResult<()> {
        let num_ops = self.variable_width_integers::<usize>(5)?;
        let mut abbv = Vec::with_capacity(num_ops);
        for _ in 0..num_ops {
            // eprintln!("read abbrev record: {}/{}", i, num_ops);
            let is_literal = self.read_flag()?;
            // eprintln!("read abbrev record: {}/{}, is_literal: {}", i, num_ops, is_literal);
            if is_literal {
                let op = self.variable_width_integers::<u32>(8)?;
                let r = AbbrevOp::Literal(op);
                // eprintln!("read abbrev record: {}/{}, res literal: {:?}", i, num_ops, r);
                abbv.push(r);
                continue;
            }
            let encoding = AbbrevOpEncoding::parse(self)?;
            // eprintln!("read abbrev record: {}/{}, encoding: {:?}", i, num_ops, encoding);
            let r = match encoding {
                AbbrevOpEncoding::Vbr | AbbrevOpEncoding::Fixed => {
                    let data = self.variable_width_integers::<usize>(5)?;
                    if data > MAX_CHUNK_SIZE {
                        return Err(WError::Num(
                            "Vbr/Fixed bit size",
                            true,
                            data,
                            MAX_CHUNK_SIZE,
                        ));
                    }
                    if data == 0 {
                        AbbrevOp::Literal(0)
                    } else if matches!(encoding, AbbrevOpEncoding::Vbr) {
                        AbbrevOp::Vbr(data)
                    } else {
                        AbbrevOp::Fixed(data)
                    }
                }
                AbbrevOpEncoding::Array => AbbrevOp::Array,
                AbbrevOpEncoding::Blob => AbbrevOp::Blob,
                AbbrevOpEncoding::Char6 => AbbrevOp::Char6,
            };
            // eprintln!("read abbrev record: {}/{}, res: {:?}", i, num_ops, r);
            abbv.push(r);
        }
        if abbv.is_empty() {
            return Err(WError::AbbrevOpEmpty);
        }
        abbv.shrink_to_fit();
        // eprintln!("read abbrev record: {:?}", abbv);
        state.abbrevs.push(Rc::new(abbv));
        Ok(())
    }
}

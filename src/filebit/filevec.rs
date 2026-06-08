//! FileVec

use std::io::{Read, Seek, SeekFrom, Write};

use super::{BitReader, BitWriter};

pub struct FileVec {
    posi: usize,
    data: Vec<u8>,
}

impl FileVec {
    pub fn data(self) -> Vec<u8> {
        self.data
    }
}

impl Read for FileVec {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let left = self.data.len() - self.posi;
        let num = left.min(buf.len());
        let idx_now = self.posi + num;
        for (i0, i1) in (0..num).zip(self.posi..idx_now) {
            buf[i0] = self.data[i1];
        }
        self.posi = idx_now;
        Ok(num)
    }
}

impl Seek for FileVec {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match pos {
            SeekFrom::Current(i) => {
                self.posi += i as usize;
            }
            SeekFrom::End(i) if i < 0 => {
                self.posi -= i.unsigned_abs() as usize;
            }
            SeekFrom::Start(i) => {
                self.posi = i as usize;
            }
            _ => {}
        }
        Ok(self.posi as u64)
    }
}

pub type BitReaderFileVec = BitReader<FileVec>;

impl BitReaderFileVec {
    pub fn from_vec(data: Vec<u8>) -> Self {
        let f = FileVec { posi: 0, data };
        Self::new(f)
    }
}

impl Write for FileVec {
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let num = buf.len();
        let num_old = self.data.len();
        let posi_new = self.posi + num;
        // 写入数据
        self.data.extend_from_slice(buf);
        self.data.copy_within(num_old..(num_old + num), self.posi);
        let num_now = if posi_new > num_old {
            // 需要延长数据
            posi_new
        } else {
            // 恢复原长
            num_old
        };
        self.data.truncate(num_now);
        self.posi = posi_new;

        Ok(num)
    }
}

pub type BitWriterFileVec = BitWriter<FileVec>;

impl BitWriterFileVec {
    pub fn with_capacity(capacity: usize) -> Self {
        let f = FileVec {
            posi: 0,
            data: Vec::with_capacity(capacity),
        };
        Self::new(f)
    }
}

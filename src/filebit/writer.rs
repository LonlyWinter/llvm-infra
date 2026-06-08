//! FileBit Writer

use std::io::{Read, Seek, SeekFrom, Write};

use crate::{WError, WResult};

const BUF_SIZE: usize = 128;
const BUF_IDX_MAX: usize = BUF_SIZE << 3;
const BUF_ZERO: [u8; BUF_SIZE] = [0u8; BUF_SIZE];

pub struct BitWriter<F: Write + Read + Seek> {
    // 文件
    file: F,
    // 文件buf
    buf: [u8; BUF_SIZE],
    // idx
    idx: usize,
}

impl<F: Write + Read + Seek> BitWriter<F> {
    pub fn new(file: F) -> Self {
        Self {
            file,
            buf: BUF_ZERO,
            idx: 0,
        }
    }

    /// 将buf内完整byte数据写入file
    pub fn flush(&mut self) -> WResult<()> {
        let num = self.idx >> 3;
        // buf数据不足一个byte
        if num == 0 {
            return Ok(());
        }
        // 写入前几个byte
        self.file.write_all(&self.buf[..num])?;
        // 把剩余的byte写入第一个byte
        let byte_bits = self.idx & 7;
        if byte_bits != 0 {
            self.buf[0] = self.buf[num];
        }
        // 指针结果
        self.idx = byte_bits;
        Ok(())
    }

    /// 检查数据
    /// res:
    /// 1. 不完整byte的bit num
    /// 2. 是否是不完整byte
    /// 3. 完整byte有多少个
    fn data_num(n: usize) -> (usize, bool, usize) {
        let bit_more = n & 7;
        let bit_more_bool = bit_more != 0;
        let byte_raw = n >> 3;
        (bit_more, bit_more_bool, byte_raw)
    }

    /// 检查数据
    /// res:
    /// 1. 不完整byte的bit num
    /// 2. 是否是不完整byte
    /// 3. 完整byte有多少个
    fn check_data(n: usize, data: &[u8]) -> WResult<(usize, bool, usize)> {
        if n == 0 {
            return Err(WError::Num("data len check", false, data.len(), 0));
        }
        let (bit_more, bit_more_bool, byte_raw) = Self::data_num(n);
        let byte_need = byte_raw + (bit_more_bool as usize);
        if data.len() < byte_need {
            return Err(WError::Num("data len need", false, data.len(), byte_need));
        }
        Ok((bit_more, bit_more_bool, byte_raw))
    }

    /// 在文件特定位置写入数据
    pub fn write_file_posi(&mut self, posi: usize, n: usize, data: &[u8]) -> WResult<()> {
        // 检查数据
        let (bit_more, bit_more_bool, byte_raw) = Self::check_data(n, data)?;
        // 将完整byte数据写入
        self.flush()?;
        // 检查位置
        let posi_file = self.file.stream_position()?;
        let posi_need = posi as u64;
        if posi_need > posi_file {
            return Err(WError::FileWritePosi(posi_need, posi_file));
        }
        // 跳到指定位置
        self.file.seek(SeekFrom::Start(posi_need))?;
        // 写入完整byte
        self.file.write_all(&data[..byte_raw])?;
        // 不完整byte
        if bit_more_bool {
            // 取旧数据
            let mut data_read = [0u8; 1];
            self.file.read_exact(&mut data_read)?;
            // 拼接成完整byte
            // data[byte_raw] 低 bit_more
            // data_read[0] 高 8 - bit_more
            let mask = (1 << bit_more) - 1;
            let data_write_low = data[byte_raw] & mask;
            let data_write_high = data_read[0] & (!mask);
            let data_write = [data_write_high | data_write_low; 1];
            // 写入新数据
            self.file.seek(SeekFrom::Current(-1))?;
            self.file.write_all(&data_write)?;
        }
        // 返回原来位置
        self.file.seek(SeekFrom::Start(posi_file))?;
        Ok(())
    }

    /// 写入n bit数据到buf
    pub fn write_data(&mut self, mut n: usize, data: &[u8]) -> WResult<()> {
        // 检查输入数据
        let _ = Self::check_data(n, data)?;
        // buf num数据
        let (bit_take, byte_half, mut idx_buf) = Self::data_num(self.idx);
        let mask_buf_low = (1 << bit_take) - 1;
        let bit_more = 8 - bit_take;
        let mask_buf_high = if bit_take == 0 {
            u8::MAX
        } else {
            (1 << bit_more) - 1
        };
        self.idx += n;
        let mut idx_data = 0;

        while n != 0 {
            eprintln!("idx_data: {idx_data}, n: {n}, bit_more: {bit_more}");
            // 先将buf byte写完整
            if byte_half {
                let data_low = self.buf[idx_buf] & mask_buf_low;
                // 不够完整byte
                if bit_more > n {
                    let mask = (1 << n) - 1;
                    let data_high = (data[idx_data] & mask) << bit_take;
                    self.buf[idx_buf] = data_high | data_low;
                    break;
                }
                // 剩余数量够完整byte, 取byt_more数量
                let data_high = (data[idx_data] & mask_buf_high) << bit_take;
                self.buf[idx_buf] = data_high | data_low;
                n -= bit_more;
                idx_buf += 1;
            }
            // buf满了
            if idx_buf == BUF_IDX_MAX {
                self.flush()?;
                idx_buf = 0;
            }
            // 再将data读完整
            // 长度是bit_take
            let (data_low, bit_need) = if bit_take == 0 {
                (data[idx_data], 8)
            } else {
                (data[idx_data] >> bit_more, bit_take)
            };
            eprintln!("data_low: {data_low}, n: {n}, bit_need: {bit_need}");
            // 不够完整byte
            if bit_need > n {
                let mask = (1 << n) - 1;
                self.buf[idx_buf] = data_low & mask;
                break;
            }
            // data该byte剩余的bit全部都是数据
            n -= bit_need;
            self.buf[idx_buf] = data_low;
            idx_data += 1;
            if bit_need == 8 {
                idx_buf += 1;
            }
        }
        Ok(())
    }

    /// 对齐bit，使用0补全
    pub fn bit_alignment(&mut self, n: usize) -> WResult<()> {
        let bit_used = ((self.file.stream_position()? as usize) << 3) + self.idx;
        let bit_extra = bit_used % n;
        if bit_extra == 0 {
            // 现在正好对齐
            return Ok(());
        }
        let mut bit_need = n - bit_extra;
        while bit_need != 0 {
            let n = if bit_need > BUF_IDX_MAX {
                BUF_IDX_MAX
            } else {
                bit_need
            };
            self.write_data(n, &BUF_ZERO)?;
            bit_need -= n;
        }
        Ok(())
    }

    pub fn inner(mut self) -> WResult<F> {
        self.flush()?;
        if self.idx != 0 {
            return Err(WError::Num("writer flush", true, self.idx, 0));
        }
        Ok(self.file)
    }
}

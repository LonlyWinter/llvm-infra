//! FileBit Reader

use std::io::SeekFrom;
use std::io::{Read, Seek};

use super::utils::{BitType, CHARACTERS_6BIT};
use crate::filebit::utils::bit_align_check;
use crate::{WError, WResult};

pub struct BitReader<F: Read + Seek> {
    // 文件
    file: F,
    // 文件buf
    buf: [u8; 16],
    // buf里面start..end比特位有效
    start: usize,
    end: usize,
}

impl<F: Read + Seek> BitReader<F> {
    pub fn new(file: F) -> Self {
        Self {
            file,
            buf: [0u8; 16],
            start: 0,
            end: 0,
        }
    }

    /// 读取buf数据，不跨byte
    /// 已经确定buf内数据足够，不再二次验证
    fn read_buf(&mut self, n: usize) -> u8 {
        // 先判断是否需要跨byte
        // 当前start所在的byte剩余多少位
        let idx = self.start >> 3;
        let bit_start = self.start & 7;
        self.start += n;
        let bit_head = (bit_start + n) & 7;
        let r = if bit_head == 0 {
            self.buf[idx]
        } else {
            self.buf[idx] & ((1 << bit_head) - 1)
        };
        let r = r >> bit_start;
        // wlog!("read: {} {} {:08b}", n, r, r);
        r
    }

    /// 空的话补数据
    fn read_file(&mut self) -> WResult<()> {
        if self.start == self.end {
            self.start = 0;
            self.end = self.file.read(&mut self.buf)? << 3;
            if self.end == 0 {
                return Err(WError::FileEmpty);
            }
        }
        Ok(())
    }

    pub fn position(&mut self) -> WResult<usize> {
        let p = self.file.stream_position()? as usize;
        let r = (p << 3) - self.end + self.start;
        Ok(r)
    }

    /// 恢复bit位置，返回现在的位置
    pub fn reload(&mut self, posi: usize) -> WResult<usize> {
        // 检查要恢复的位置是否对齐
        bit_align_check(posi)?;
        // 检查现在所在的位置是否对齐
        let n = self.position()?;
        bit_align_check(n)?;
        // 开始恢复
        let posi = (posi >> 3) as u64;
        self.start = 0;
        self.end = 0;
        let v = self.file.seek(SeekFrom::Start(posi))?;
        if v != posi {
            return Err(WError::FileRead(format!("Read {}/{} posi error", v, posi)));
        }
        self.read_file()?;
        // 返回现在的位置
        Ok(n)
    }

    pub fn has_data(&mut self) -> bool {
        let _ = self.read_file();
        self.start != self.end
    }

    /// 读取1bit数据
    pub fn read_flag(&mut self) -> WResult<bool> {
        self.read_file()?;
        let r = self.read_buf(1);
        Ok(r == 1)
    }

    pub fn skip_end(&mut self) -> WResult<()> {
        self.start = self.end;
        let mut n_old = 0;
        while let Ok(n) = self.file.seek(SeekFrom::Current(i64::MAX)) {
            if n == n_old {
                break;
            }
            n_old = n;
        }
        Ok(())
    }

    /// skip bits from now
    pub fn skip_bits(&mut self, n: usize) -> WResult<()> {
        let bit_buf = self.end - self.start;
        if n <= bit_buf {
            // buf内的数据就够了
            self.start += n;
        } else {
            // buf内的数据不够
            // buf清空
            self.start = self.end;
            // 移动文件位置
            let bit_more = n - bit_buf;
            let byte_more = (bit_more >> 3) as i64;
            self.file.seek(SeekFrom::Current(byte_more))?;
            // 剩余bit
            let bit_more = bit_more & 7;
            if bit_more != 0 {
                self.read_file()?;
                self.start = bit_more;
            }
        }
        Ok(())
    }

    /// has n bits data
    pub fn has_bits(&mut self, n: usize) -> WResult<bool> {
        let bit_buf = self.end - self.start;
        if n <= bit_buf {
            return Ok(true);
        }
        // buf内的数据不够
        // 需要多少个byte
        let mut bit_more = n - bit_buf;
        if (bit_more & 7) != 0 {
            bit_more += 8;
        }
        let byte_more = (bit_more >> 3) as i64;
        let posi_old = self.file.stream_position()?;
        let r = self.file.seek(SeekFrom::Current(byte_more))?;
        self.file.seek(SeekFrom::Start(posi_old))?;
        Ok((r - posi_old) == (byte_more as u64))
    }

    /// Word Alignment
    /// - n: bit num
    ///
    /// https://llvm.org/docs/BitCodeFormat.html#word-alignment
    pub fn word_alignment(&mut self, n: usize) -> WResult<()> {
        let bit_file = (self.file.stream_position()? as usize) << 3;
        let bit_left = self.end - self.start;
        let bit_used = bit_file - bit_left;
        let bit_need = n - (bit_used % n);
        if bit_need == n {
            return Ok(());
        }
        // 需要alignment
        if bit_need <= bit_left {
            // 剩下的足够
            self.start += bit_need;
        } else {
            // 剩下的不够
            let bit_more_all = bit_left - bit_need;
            self.start = bit_more_all & 7;
            let byte_more = bit_more_all >> 3;
            if byte_more > 0 {
                self.file.seek(SeekFrom::Current(byte_more as i64))?;
            }
            if self.start > 0 {
                // 剩下的不是整byte，读数据
                self.end = self.file.read(&mut self.buf)? << 3;
            } else {
                // 剩下的正好是整byte
                self.end = 0;
            }
        }
        Ok(())
    }

    /// 读取数据，比特长度小于等于8
    pub fn read_bit8(&mut self, n: usize) -> WResult<u8> {
        if n > 8 {
            return Err(WError::Num("BitReader read bit", true, n, 8));
        }
        if n == 0 {
            return Ok(0);
        }
        self.read_file()?;
        let byte_left = 8 - (self.start & 7);
        let res = if n > byte_left {
            // 当前byte剩余的不够，先读出来，读出来的是低位
            let res_low = self.read_buf(byte_left);
            // 再读，读出来的是高位
            let res_high = self.read_bit8(n - byte_left)?;
            (res_high << byte_left) ^ res_low
        } else {
            // 当前byte剩余的够了，读出来
            self.read_buf(n)
        };
        Ok(res)
    }

    /// 读取数据，比特长度大于8
    pub fn read_bitn(&mut self, n: usize) -> WResult<Vec<u8>> {
        if n <= 8 {
            return Err(WError::Num("BitReader read bitn", false, n, 8));
        }
        let mut res = Vec::with_capacity((n >> 3) + 1);
        // 每8bit读取一次
        let mut n_left = n;
        while n_left > 8 {
            let byte_now = self.read_bit8(8)?;
            n_left -= 8;
            res.push(byte_now);
        }
        let byte_now = self.read_bit8(n_left)?;
        res.push(byte_now);
        Ok(res)
    }

    /// 读取bytes数据，已经align 8bit
    pub fn read_bytes(&mut self, n: usize) -> WResult<Vec<u8>> {
        bit_align_check(self.start)?;
        let idx_start = self.start >> 3;
        let bytes_left = (self.end >> 3) - idx_start;
        let res = if bytes_left >= n {
            // buf里面的够用
            self.start += n << 3;
            self.buf[idx_start..(idx_start + n)].to_vec()
        } else {
            // buf里面的不够用
            let bytes_more = n - bytes_left;
            let mut res = Vec::with_capacity(n);
            // 先读取剩余的
            self.start = self.end;
            res.extend_from_slice(&self.buf[idx_start..]);
            // 再读文件补充
            self.read_file()?;
            let bytes_other = self.read_bytes(bytes_more)?;
            res.extend(bytes_other);
            res
        };
        Ok(res)
    }

    /// Fixed Width Integers
    /// https://llvm.org/docs/BitCodeFormat.html#fixed-width-value
    pub fn fixed_width_integers<T: BitType>(&mut self, n: usize) -> WResult<T> {
        let res = if n > 8 {
            let data = self.read_bitn(n)?;
            T::parse_data(data)
        } else {
            let data = self.read_bit8(n)?;
            T::from(data)
        };
        // eprintln!("Fixed: {} {}", n, res);
        Ok(res)
    }

    /// Variable Width Integers
    /// https://llvm.org/docs/BitCodeFormat.html#variable-width-value
    pub fn variable_width_integers<T: BitType>(&mut self, n: usize) -> WResult<T> {
        let mut res = T::from(0u8);
        let one = T::from(1u8);
        if n > 8 {
            let mask_high = one << (n - 1);
            let mask_data = mask_high - one;
            let mut idx = 0;
            while let Ok(num) = self.read_bitn(n) {
                // wlog!("res: {}, {:b}", res, num);
                let num = T::parse_data(num);
                res += (num & mask_data) << idx;
                idx += n - 1;
                // 检查高位，如果不是1则退出
                if (num & mask_high) != mask_high {
                    break;
                }
            }
        } else if n > 1 {
            let mask_high = 1 << (n - 1);
            let mask_data = mask_high - 1;
            let mut idx = 0;
            while let Ok(num) = self.read_bit8(n) {
                let r = T::from(num & mask_data);
                // eprintln!("Vbr single: {}, {:08b}", r, num);
                res += r << idx;
                idx += n - 1;
                // 检查高位，如果不是1则退出
                if (num & mask_high) != mask_high {
                    break;
                }
            }
        } else {
            return Err(WError::Num("BitReader read vbr", false, n, 1));
        }
        // eprintln!("Vbr: {} {} {:08b}", n, res, res);
        Ok(res)
    }

    /// 6-bit characters
    /// https://llvm.org/docs/BitCodeFormat.html#bit-characters
    pub fn characters_6bit(&mut self) -> WResult<char> {
        let idx = self.fixed_width_integers::<u8>(6)? as usize;
        Ok(CHARACTERS_6BIT[idx])
    }
}

/// Signed VBRs
/// https://llvm.org/docs/BitCodeFormat.html#signed-vbrs
macro_rules! decoded_signed_vbrs {
    ($($name:ident = $unsign:ty = $sign:ty,)*) => {
        $(
            pub fn $name(data: $unsign) -> $unsign {
                let sign = (data & 1) as $sign;
                let val = (data >> 1) as $sign;
                let r = if sign == 0 {
                    val
                } else {
                    -val
                };
                r as $unsign
            }
        )*
    };
}

decoded_signed_vbrs! {
    decoded_signed_vbrs_u8 = u8 = i8,
    decoded_signed_vbrs_u32 = u32 = i32,
    decoded_signed_vbrs_u64 = u64 = i64,
}

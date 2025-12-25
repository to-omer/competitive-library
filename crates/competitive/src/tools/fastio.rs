use std::{
    ffi::{c_int, c_void},
    fs::File,
    io::{BufWriter, Read, StdoutLock, Write, stdout},
    os::fd::FromRawFd,
    ptr,
    str::FromStr,
};

unsafe extern "C" {
    fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: c_int,
        flags: c_int,
        fd: c_int,
        offset: isize,
    ) -> *mut c_void;
}

pub struct FastInput {
    ptr: *const u8,
}

impl FastInput {
    pub unsafe fn stdin() -> Self {
        unsafe {
            let mut stdin = File::from_raw_fd(0);
            let ptr = if let Ok(metadata) = stdin.metadata() {
                let len = metadata.len() as usize;
                mmap(ptr::null_mut(), len, 1, 2, 0, 0) as *const u8
            } else {
                let mut buf = vec![];
                stdin.read_to_end(&mut buf).unwrap();
                Box::into_raw(buf.into_boxed_slice()) as *const u8
            };
            FastInput { ptr }
        }
    }

    pub unsafe fn from_slice(s: &[u8]) -> Self {
        FastInput { ptr: s.as_ptr() }
    }

    unsafe fn fetch_ud4(&mut self) -> u16 {
        unsafe {
            let mut x: u32 = ptr::read_unaligned(self.ptr as *const u32);
            x ^= 0x30303030;
            let tmp = (x & 0xf0f0f0f0).trailing_zeros() >> 3;
            x <<= 32 - (tmp << 3);
            x = x.wrapping_mul(10).wrapping_add(x >> 8) & 0x00ff00ff;
            x = x.wrapping_mul(100).wrapping_add(x >> 16) & 0x0000ffff;
            self.ptr = self.ptr.add((tmp + 1) as usize);
            x as u16
        }
    }

    unsafe fn fetch_ud8(&mut self) -> u32 {
        unsafe {
            let mut x: u64 = ptr::read_unaligned(self.ptr as *const u64);
            x ^= 0x3030303030303030;
            let tmp = (x & 0xf0f0f0f0f0f0f0f0).trailing_zeros() >> 3;
            x <<= 64 - (tmp << 3);
            x = x.wrapping_mul(10).wrapping_add(x >> 8) & 0x00ff00ff00ff00ff;
            x = x.wrapping_mul(100).wrapping_add(x >> 16) & 0x0000ffff0000ffff;
            x = x.wrapping_mul(10000).wrapping_add(x >> 32) & 0x00000000ffffffff;
            self.ptr = self.ptr.add((tmp + 1) as usize);
            x as u32
        }
    }

    pub unsafe fn u8(&mut self) -> u8 {
        unsafe { self.fetch_ud4() as u8 }
    }

    pub unsafe fn u16(&mut self) -> u16 {
        unsafe { self.fetch_ud8() as u16 }
    }

    /// 0..=99_999_999
    pub unsafe fn u32_small(&mut self) -> u32 {
        unsafe { self.fetch_ud8() }
    }

    pub unsafe fn u32(&mut self) -> u32 {
        unsafe {
            let mut res = 0u32;
            let mut buf: [u64; 2] = ptr::read_unaligned(self.ptr as *const [u64; 2]);
            buf[0] ^= 0x3030303030303030;
            buf[1] ^= 0x3030303030303030;
            let mut rem;
            {
                let mut x = buf[0];
                rem = x;
                if (x & 0xf0f0f0f0f0f0f0f0) == 0 {
                    rem = buf[1];
                    x = x.wrapping_mul(10).wrapping_add(x >> 8) & 0x00ff00ff00ff00ff;
                    x = x.wrapping_mul(100).wrapping_add(x >> 16) & 0x0000ffff0000ffff;
                    x = x.wrapping_mul(10000).wrapping_add(x >> 32) & 0x00000000ffffffff;
                    res = x as u32;
                    self.ptr = self.ptr.add(8);
                }
            }
            {
                let mut x = (rem & 0xffffffff) as u32;
                if (x & 0xf0f0f0f0) == 0 {
                    rem >>= 32;
                    x = x.wrapping_mul(10).wrapping_add(x >> 8) & 0x00ff00ff;
                    x = x.wrapping_mul(100).wrapping_add(x >> 16) & 0x0000ffff;
                    res = res.wrapping_mul(10000).wrapping_add(x);
                    self.ptr = self.ptr.add(4);
                }
            }
            {
                let mut x = (rem & 0xffff) as u16;
                if (x & 0xf0f0) == 0 {
                    rem >>= 16;
                    x = x.wrapping_mul(10).wrapping_add(x >> 8) & 0x00ff;
                    res = res.wrapping_mul(100).wrapping_add(x as u32);
                    self.ptr = self.ptr.add(2);
                }
            }
            {
                let x = (rem & 0xf0) == 0;
                if x {
                    res = res.wrapping_mul(10).wrapping_add((rem & 0xff) as u32);
                }
                self.ptr = self.ptr.add(x as usize + 1);
            }
            res
        }
    }

    pub unsafe fn u64(&mut self) -> u64 {
        unsafe {
            let mut res;
            let mut x = ptr::read_unaligned(self.ptr as *const u64);
            x ^= 0x3030303030303030;
            if (x & 0xf0f0f0f0f0f0f0f0) == 0 {
                self.ptr = self.ptr.add(8);
                let mut y = ptr::read_unaligned(self.ptr as *const u64);
                x = x.wrapping_mul(10).wrapping_add(x >> 8) & 0x00ff00ff00ff00ff;
                x = x.wrapping_mul(100).wrapping_add(x >> 16) & 0x0000ffff0000ffff;
                x = x.wrapping_mul(10000).wrapping_add(x >> 32) & 0x00000000ffffffff;
                res = x;
                y ^= 0x3030303030303030;
                if (y & 0xf0f0f0f0f0f0f0f0) == 0 {
                    self.ptr = self.ptr.add(8);
                    y = y.wrapping_mul(10).wrapping_add(y >> 8) & 0x00ff00ff00ff00ff;
                    y = y.wrapping_mul(100).wrapping_add(y >> 16) & 0x0000ffff0000ffff;
                    y = y.wrapping_mul(10000).wrapping_add(y >> 32) & 0x00000000ffffffff;
                    res = res.wrapping_mul(100000000).wrapping_add(y);
                    let mut rem = ptr::read_unaligned(self.ptr as *const u32);
                    rem ^= 0x30303030;
                    if (rem & 0xf0f0f0f0) == 0 {
                        rem = rem.wrapping_mul(10).wrapping_add(rem >> 8) & 0x00ff00ff;
                        rem = rem.wrapping_mul(100).wrapping_add(rem >> 16) & 0x0000ffff;
                        res = res.wrapping_mul(10000).wrapping_add(rem as u64);
                        self.ptr = self.ptr.add(5);
                    } else if (rem & 0xf0f0f0) == 0 {
                        res = res.wrapping_mul(1000).wrapping_add(
                            ((rem & 0xff) as u64)
                                .wrapping_mul(100)
                                .wrapping_add((((rem.wrapping_mul(2561)) & 0xff0000) >> 16) as u64),
                        );
                        self.ptr = self.ptr.add(4);
                    } else if (rem & 0xf0f0) == 0 {
                        res = res.wrapping_mul(100).wrapping_add(
                            (((rem >> 8).wrapping_add(rem.wrapping_mul(10))) & 0xff) as u64,
                        );
                        self.ptr = self.ptr.add(3);
                    } else if (rem & 0xf0) == 0 {
                        res = res.wrapping_mul(10).wrapping_add((rem & 0x0000000f) as u64);
                        self.ptr = self.ptr.add(2);
                    } else {
                        self.ptr = self.ptr.add(1);
                    }
                } else {
                    let mut x = (y & 0xffffffff) as u32;
                    if (x & 0xf0f0f0f0) == 0 {
                        y >>= 32;
                        x = x.wrapping_mul(10).wrapping_add(x >> 8) & 0x00ff00ff;
                        x = x.wrapping_mul(100).wrapping_add(x >> 16) & 0x0000ffff;
                        res = res.wrapping_mul(10000).wrapping_add(x as u64);
                        self.ptr = self.ptr.add(4);
                    }
                    let mut x = (y & 0xffff) as u16;
                    if (x & 0xf0f0) == 0 {
                        y >>= 16;
                        x = x.wrapping_mul(10).wrapping_add(x >> 8) & 0x00ff;
                        res = res.wrapping_mul(100).wrapping_add(x as u64);
                        self.ptr = self.ptr.add(2);
                    }
                    let x = (y & 0xf0) == 0;
                    if x {
                        res = res.wrapping_mul(10).wrapping_add((y & 0xff) as u64)
                    }
                    self.ptr = self.ptr.add(x as usize + 1);
                }
            } else {
                let tmp = (x & 0xf0f0f0f0f0f0f0f0).trailing_zeros() >> 3;
                x = x.wrapping_shl(64 - (tmp << 3));
                x = x.wrapping_mul(10).wrapping_add(x >> 8) & 0x00ff00ff00ff00ff;
                x = x.wrapping_mul(100).wrapping_add(x >> 16) & 0x0000ffff0000ffff;
                x = x.wrapping_mul(10000).wrapping_add(x >> 32) & 0x00000000ffffffff;
                res = x;
                self.ptr = self.ptr.add((tmp + 1) as usize);
            }
            res
        }
    }

    pub unsafe fn u128(&mut self) -> u128 {
        unsafe {
            let mut res = 0u128;
            for i in 0..4 {
                let mut x = ptr::read_unaligned(self.ptr as *const u64);
                x ^= 0x3030303030303030;
                if (x & 0xf0f0f0f0f0f0f0f0) != 0 {
                    break;
                }
                x = x.wrapping_mul(10).wrapping_add(x >> 8) & 0x00ff00ff00ff00ff;
                x = x.wrapping_mul(100).wrapping_add(x >> 16) & 0x0000ffff0000ffff;
                x = x.wrapping_mul(10000).wrapping_add(x >> 32) & 0x00000000ffffffff;
                if i == 0 {
                    res = x as u128;
                } else {
                    res = res.wrapping_mul(100000000).wrapping_add(x as u128);
                }
                self.ptr = self.ptr.add(8);
            }
            let mut res2 = 0u64;
            let mut pow = 1u64;
            let mut x = ptr::read_unaligned(self.ptr as *const u64);
            x ^= 0x3030303030303030;
            let mut rem = x;
            {
                if (x & 0xf0f0f0f0) == 0 {
                    rem >>= 32;
                    x = x.wrapping_mul(10).wrapping_add(x >> 8) & 0x00ff00ff;
                    x = x.wrapping_mul(100).wrapping_add(x >> 16) & 0x0000ffff;
                    res2 = x as u64;
                    pow = 10000;
                    self.ptr = self.ptr.add(4);
                }
            }
            {
                let mut x = (rem & 0xffff) as u16;
                if (x & 0xf0f0) == 0 {
                    rem >>= 16;
                    x = x.wrapping_mul(10).wrapping_add(x >> 8) & 0x00ff;
                    res2 = res2.wrapping_mul(100).wrapping_add(x as u64);
                    pow = pow.wrapping_mul(100);
                    self.ptr = self.ptr.add(2);
                }
            }
            {
                let x = (rem & 0xf0) == 0;
                if x {
                    res2 = res2.wrapping_mul(10).wrapping_add((rem & 0xff) as u64);
                    pow = pow.wrapping_mul(10);
                }
                self.ptr = self.ptr.add(x as usize + 1);
            }
            res = res.wrapping_mul(pow as u128).wrapping_add(res2 as u128);
            res
        }
    }

    pub unsafe fn usize(&mut self) -> usize {
        unsafe { self.u64() as usize }
    }

    pub unsafe fn i8(&mut self) -> i8 {
        unsafe {
            let b = *self.ptr == b'-';
            self.ptr = self.ptr.add(b as usize);
            let mut x = self.u8() as i8;
            if b {
                x = x.wrapping_neg();
            }
            x
        }
    }

    pub unsafe fn i16(&mut self) -> i16 {
        unsafe {
            let b = *self.ptr == b'-';
            self.ptr = self.ptr.add(b as usize);
            let mut x = self.u16() as i16;
            if b {
                x = x.wrapping_neg();
            }
            x
        }
    }

    pub unsafe fn i32(&mut self) -> i32 {
        unsafe {
            let b = *self.ptr == b'-';
            self.ptr = self.ptr.add(b as usize);
            let mut x = self.u32() as i32;
            if b {
                x = x.wrapping_neg();
            }
            x
        }
    }

    pub unsafe fn i64(&mut self) -> i64 {
        unsafe {
            let b = *self.ptr == b'-';
            self.ptr = self.ptr.add(b as usize);
            let mut x = self.u64() as i64;
            if b {
                x = x.wrapping_neg();
            }
            x
        }
    }

    pub unsafe fn i128(&mut self) -> i128 {
        unsafe {
            let b = *self.ptr == b'-';
            self.ptr = self.ptr.add(b as usize);
            let mut x = self.u128() as i128;
            if b {
                x = x.wrapping_neg();
            }
            x
        }
    }

    pub unsafe fn isize(&mut self) -> isize {
        unsafe { self.i64() as isize }
    }

    pub unsafe fn byte(&mut self) -> u8 {
        unsafe {
            let c = *self.ptr;
            self.ptr = self.ptr.add(2);
            c
        }
    }

    pub unsafe fn bytes<'a>(&mut self) -> &'a [u8] {
        unsafe {
            let start = self.ptr;
            while !(*self.ptr).is_ascii_whitespace() {
                self.ptr = self.ptr.add(1);
            }
            let len = self.ptr.offset_from(start) as usize;
            self.ptr = self.ptr.add(1);
            std::slice::from_raw_parts(start, len)
        }
    }

    pub unsafe fn parse<T>(&mut self) -> T
    where
        T: FromStr,
    {
        unsafe {
            let s = std::str::from_utf8_unchecked(self.bytes());
            s.parse().ok().unwrap()
        }
    }
}

static DIGIT4: [[u8; 4]; 10000] = const {
    let mut arr = [[b' '; 4]; 10000];
    let mut i = 0;
    while i < 10000 {
        let mut x = i;
        let mut j = 4;
        while j > 0 {
            j -= 1;
            arr[i][j] = b'0' + (x % 10) as u8;
            x /= 10;
        }
        i += 1;
    }
    arr
};

pub struct FastOutput<W>
where
    W: Write,
{
    buf: BufWriter<W>,
}

impl FastOutput<StdoutLock<'static>> {
    pub fn stdout() -> Self {
        Self::with_capacity(1 << 12, stdout().lock())
    }
}

impl<W> FastOutput<W>
where
    W: Write,
{
    pub fn new(writer: W) -> Self {
        FastOutput {
            buf: BufWriter::new(writer),
        }
    }

    pub fn with_capacity(capacity: usize, writer: W) -> Self {
        FastOutput {
            buf: BufWriter::with_capacity(capacity, writer),
        }
    }

    pub fn flush(&mut self) {
        self.buf.flush().unwrap();
    }

    fn write_digit4(&mut self, x: usize) {
        debug_assert!(x < 10000);
        self.buf.write_all(&DIGIT4[x]).unwrap();
    }

    fn write_digit4_trimmed(&mut self, x: usize) {
        debug_assert!(x < 10000);
        let off = (x < 10) as u8 + (x < 100) as u8 + (x < 1000) as u8;
        self.buf.write_all(&DIGIT4[x][off as usize..]).unwrap();
    }

    pub fn u8(&mut self, x: u8) {
        let off = (x < 10) as u8 + (x < 100) as u8 + 1;
        self.buf
            .write_all(&DIGIT4[x as usize][off as usize..])
            .unwrap();
    }

    pub fn u16(&mut self, x: u16) {
        if x >= 10000 {
            self.write_digit4_trimmed((x / 10000) as usize);
            self.write_digit4((x % 10000) as usize);
        } else {
            self.write_digit4_trimmed(x as usize);
        }
    }

    pub fn u32(&mut self, x: u32) {
        if x >= 1_0000_0000 {
            let b = x / 10000;
            let a = b / 10000;
            self.write_digit4_trimmed(a as usize);
            self.write_digit4((b % 10000) as usize);
            self.write_digit4((x % 10000) as usize);
        } else if x >= 10000 {
            self.write_digit4_trimmed((x / 10000) as usize);
            self.write_digit4((x % 10000) as usize);
        } else {
            self.write_digit4_trimmed(x as usize);
        }
    }

    pub fn u64(&mut self, x: u64) {
        if x >= 1_0000_0000_0000_0000 {
            let d = x / 10000;
            let c = d / 10000;
            let b = c / 10000;
            let a = b / 10000;
            self.write_digit4_trimmed(a as usize);
            self.write_digit4((b % 10000) as usize);
            self.write_digit4((c % 10000) as usize);
            self.write_digit4((d % 10000) as usize);
            self.write_digit4((x % 10000) as usize);
        } else if x >= 1_0000_0000_0000 {
            let c = x / 10000;
            let b = c / 10000;
            let a = b / 10000;
            self.write_digit4_trimmed(a as usize);
            self.write_digit4((b % 10000) as usize);
            self.write_digit4((c % 10000) as usize);
            self.write_digit4((x % 10000) as usize);
        } else if x >= 1_0000_0000 {
            let b = x / 10000;
            let a = b / 10000;
            self.write_digit4_trimmed(a as usize);
            self.write_digit4((b % 10000) as usize);
            self.write_digit4((x % 10000) as usize);
        } else if x >= 10000 {
            self.write_digit4_trimmed((x / 10000) as usize);
            self.write_digit4((x % 10000) as usize);
        } else {
            self.write_digit4_trimmed(x as usize);
        }
    }

    pub fn i8(&mut self, x: i8) {
        if x < 0 {
            self.buf.write_all(b"-").unwrap();
            self.u8(x.wrapping_neg() as u8);
        } else {
            self.u8(x as u8);
        }
    }

    pub fn i16(&mut self, x: i16) {
        if x < 0 {
            self.buf.write_all(b"-").unwrap();
            self.u16(x.wrapping_neg() as u16);
        } else {
            self.u16(x as u16);
        }
    }

    pub fn i32(&mut self, x: i32) {
        if x < 0 {
            self.buf.write_all(b"-").unwrap();
            self.u32(x.wrapping_neg() as u32);
        } else {
            self.u32(x as u32);
        }
    }

    pub fn i64(&mut self, x: i64) {
        if x < 0 {
            self.buf.write_all(b"-").unwrap();
            self.u64(x.wrapping_neg() as u64);
        } else {
            self.u64(x as u64);
        }
    }

    pub fn byte(&mut self, b: u8) {
        self.buf.write_all(&[b]).unwrap();
    }

    pub fn bytes(&mut self, s: &[u8]) {
        self.buf.write_all(s).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn test_past_input_u8() {
        let mut a = vec![];
        let mut s = String::new();
        for i in 0..=u8::MAX {
            a.push(i);
            s.push_str(&format!("{}\n", i));
        }
        let mut fi = unsafe { FastInput::from_slice(s.as_bytes()) };
        for a in a {
            let x = unsafe { fi.u8() };
            assert_eq!(x, a);
        }
    }

    #[test]
    fn test_past_input_u16() {
        let mut a = vec![];
        let mut s = String::new();
        for i in 0..=u16::MAX {
            a.push(i);
            s.push_str(&format!("{}\n", i));
        }
        let mut fi = unsafe { FastInput::from_slice(s.as_bytes()) };
        for a in a {
            let x = unsafe { fi.u16() };
            assert_eq!(x, a);
        }
    }

    #[test]
    fn test_past_input_u32() {
        let mut rng = Xorshift::default();
        let mut a = vec![];
        let mut s = String::new();
        for _ in 0..100_000 {
            let k = rng.random(0..=u32::BITS);
            let i = rng.random(0..=u32::MAX.wrapping_shr(u32::BITS - k));
            a.push(i);
            s.push_str(&format!("{}\n", i));
        }
        let mut fi = unsafe { FastInput::from_slice(s.as_bytes()) };
        for a in a {
            let x = unsafe { fi.u32() };
            assert_eq!(x, a);
        }
    }

    #[test]
    fn test_past_input_u64() {
        let mut rng = Xorshift::default();
        let mut a = vec![];
        let mut s = String::new();
        for _ in 0..100_000 {
            let k = rng.random(0..=u64::BITS);
            let i = rng.random(0..=u64::MAX.wrapping_shr(u64::BITS - k));
            a.push(i);
            s.push_str(&format!("{}\n", i));
        }
        let mut fi = unsafe { FastInput::from_slice(s.as_bytes()) };
        for a in a {
            let x = unsafe { fi.u64() };
            assert_eq!(x, a);
        }
    }

    #[test]
    fn test_past_input_u128() {
        let mut rng = Xorshift::default();
        let mut a = vec![];
        let mut s = String::new();
        for _ in 0..100_000 {
            let k = rng.random(0..=u128::BITS);
            let i = rng.random(0..=u128::MAX.wrapping_shr(u128::BITS - k));
            a.push(i);
            s.push_str(&format!("{}\n", i));
        }
        let mut fi = unsafe { FastInput::from_slice(s.as_bytes()) };
        for a in a {
            let x = unsafe { fi.u128() };
            assert_eq!(x, a);
        }
    }

    #[test]
    fn test_past_input_i8() {
        let mut a = vec![];
        let mut s = String::new();
        for i in i8::MIN..=i8::MAX {
            a.push(i);
            s.push_str(&format!("{}\n", i));
        }
        let mut fi = unsafe { FastInput::from_slice(s.as_bytes()) };
        for a in a {
            let x = unsafe { fi.i8() };
            assert_eq!(x, a);
        }
    }

    #[test]
    fn test_past_input_i16() {
        let mut a = vec![];
        let mut s = String::new();
        for i in i16::MIN..=i16::MAX {
            a.push(i);
            s.push_str(&format!("{}\n", i));
        }
        let mut fi = unsafe { FastInput::from_slice(s.as_bytes()) };
        for a in a {
            let x = unsafe { fi.i16() };
            assert_eq!(x, a);
        }
    }

    #[test]
    fn test_past_input_i32() {
        let mut rng = Xorshift::default();
        let mut a = vec![];
        let mut s = String::new();
        for _ in 0..100_000 {
            let k = rng.random(0..=u32::BITS);
            let i = rng
                .random(0..=u32::MAX.wrapping_shr(u32::BITS - k))
                .cast_signed();
            a.push(i);
            s.push_str(&format!("{}\n", i));
        }
        let mut fi = unsafe { FastInput::from_slice(s.as_bytes()) };
        for a in a {
            let x = unsafe { fi.i32() };
            assert_eq!(x, a);
        }
    }

    #[test]
    fn test_past_input_i64() {
        let mut rng = Xorshift::default();
        let mut a = vec![];
        let mut s = String::new();
        for _ in 0..100_000 {
            let k = rng.random(0..=u64::BITS);
            let i = rng
                .random(0..=u64::MAX.wrapping_shr(u64::BITS - k))
                .cast_signed();
            a.push(i);
            s.push_str(&format!("{}\n", i));
        }
        let mut fi = unsafe { FastInput::from_slice(s.as_bytes()) };
        for a in a {
            let x = unsafe { fi.i64() };
            assert_eq!(x, a);
        }
    }

    #[test]
    fn test_past_input_i128() {
        let mut rng = Xorshift::default();
        let mut a = vec![];
        let mut s = String::new();
        for _ in 0..100_000 {
            let k = rng.random(0..=u128::BITS);
            let i = rng
                .random(0..=u128::MAX.wrapping_shr(u128::BITS - k))
                .cast_signed();
            a.push(i);
            s.push_str(&format!("{}\n", i));
        }
        let mut fi = unsafe { FastInput::from_slice(s.as_bytes()) };
        for a in a {
            let x = unsafe { fi.i128() };
            assert_eq!(x, a);
        }
    }

    #[test]
    fn test_fast_output_u8() {
        let mut fo = FastOutput::new(Vec::new());
        for i in 0..=u8::MAX {
            fo.u8(i);
            fo.byte(b'\n');
        }
        let buf = fo.buf.into_inner().unwrap();
        let s = String::from_utf8(buf).unwrap();
        let mut lines = s.lines();
        for i in 0..=u8::MAX {
            let line = lines.next().unwrap();
            assert_eq!(line, i.to_string());
        }
    }

    #[test]
    fn test_fast_output_u16() {
        let mut fo = FastOutput::new(Vec::new());
        for i in 0..=u16::MAX {
            fo.u16(i);
            fo.byte(b'\n');
        }
        let buf = fo.buf.into_inner().unwrap();
        let s = String::from_utf8(buf).unwrap();
        let mut lines = s.lines();
        for i in 0..=u16::MAX {
            let line = lines.next().unwrap();
            assert_eq!(line, i.to_string());
        }
    }

    #[test]
    fn test_fast_output_u32() {
        let mut rng = Xorshift::default();
        let mut fo = FastOutput::new(Vec::new());
        let mut a = vec![];
        for _ in 0..100_000 {
            let k = rng.random(0..=u32::BITS);
            let i = rng.random(0..=u32::MAX.wrapping_shr(u32::BITS - k));
            a.push(i);
            fo.u32(i);
            fo.byte(b'\n');
        }
        let buf = fo.buf.into_inner().unwrap();
        let s = String::from_utf8(buf).unwrap();
        let mut lines = s.lines();
        for &a in &a {
            let line = lines.next().unwrap();
            assert_eq!(line, a.to_string());
        }
    }

    #[test]
    fn test_fast_output_u64() {
        let mut rng = Xorshift::default();
        let mut fo = FastOutput::new(Vec::new());
        let mut a = vec![];
        for _ in 0..100_000 {
            let k = rng.random(0..=u64::BITS);
            let i = rng.random(0..=u64::MAX.wrapping_shr(u64::BITS - k));
            a.push(i);
            fo.u64(i);
            fo.byte(b'\n');
        }
        let buf = fo.buf.into_inner().unwrap();
        let s = String::from_utf8(buf).unwrap();
        let mut lines = s.lines();
        for &a in &a {
            let line = lines.next().unwrap();
            assert_eq!(line, a.to_string());
        }
    }

    #[test]
    fn test_fast_output_i8() {
        let mut fo = FastOutput::new(Vec::new());
        for i in i8::MIN..=i8::MAX {
            fo.i8(i);
            fo.byte(b'\n');
        }
        let buf = fo.buf.into_inner().unwrap();
        let s = String::from_utf8(buf).unwrap();
        let mut lines = s.lines();
        for i in i8::MIN..=i8::MAX {
            let line = lines.next().unwrap();
            assert_eq!(line, i.to_string());
        }
    }

    #[test]
    fn test_fast_output_i16() {
        let mut fo = FastOutput::new(Vec::new());
        for i in i16::MIN..=i16::MAX {
            fo.i16(i);
            fo.byte(b'\n');
        }
        let buf = fo.buf.into_inner().unwrap();
        let s = String::from_utf8(buf).unwrap();
        let mut lines = s.lines();
        for i in i16::MIN..=i16::MAX {
            let line = lines.next().unwrap();
            assert_eq!(line, i.to_string());
        }
    }

    #[test]
    fn test_fast_output_i32() {
        let mut rng = Xorshift::default();
        let mut fo = FastOutput::new(Vec::new());
        let mut a = vec![];
        for _ in 0..100_000 {
            let k = rng.random(0..=u32::BITS);
            let i = rng
                .random(0..=u32::MAX.wrapping_shr(u32::BITS - k))
                .cast_signed();
            a.push(i);
            fo.i32(i);
            fo.byte(b'\n');
        }
        let buf = fo.buf.into_inner().unwrap();
        let s = String::from_utf8(buf).unwrap();
        let mut lines = s.lines();
        for &a in &a {
            let line = lines.next().unwrap();
            assert_eq!(line, a.to_string());
        }
    }

    #[test]
    fn test_fast_output_i64() {
        let mut rng = Xorshift::default();
        let mut fo = FastOutput::new(Vec::new());
        let mut a = vec![];
        for _ in 0..100_000 {
            let k = rng.random(0..=u64::BITS);
            let i = rng
                .random(0..=u64::MAX.wrapping_shr(u64::BITS - k))
                .cast_signed();
            a.push(i);
            fo.i64(i);
            fo.byte(b'\n');
        }
        let buf = fo.buf.into_inner().unwrap();
        let s = String::from_utf8(buf).unwrap();
        let mut lines = s.lines();
        for &a in &a {
            let line = lines.next().unwrap();
            assert_eq!(line, a.to_string());
        }
    }
}

use std::{
    fmt,
    fs::File,
    io::{Read, StdoutLock, Write, stdout},
    os::fd::AsFd,
    ptr,
    str::FromStr,
};

#[cfg(target_os = "linux")]
use std::ffi::{c_int, c_void};

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: c_int,
        flags: c_int,
        fd: c_int,
        offset: isize,
    ) -> *mut c_void;
    fn munmap(addr: *mut c_void, len: usize) -> c_int;
    fn getpagesize() -> c_int;
}

/// Token reader for little-endian targets. Integer reads require a decimal
/// representation that fits the requested type, with at most that type's maximum
/// number of digits and an optional `-` for signed types.
///
/// Every read, including iterator reads, must start at an available token.
/// Reads consume exactly one trailing ASCII whitespace byte. `parse` and
/// `ScanSource` require UTF-8. Slices returned by `bytes` must not outlive the input
/// allocation. `bytes` and `parse` may also read an empty field at a delimiter.
///
/// Empty collection scans skip pending whitespace when their length is specified.
///
/// On x86-64, enabling `ssse3` at compile time selects SIMD for `u64` tokens.
pub struct FastInput {
    ptr: *const u8,
    end: *const u8,
}

impl FastInput {
    /// Reads all of stdin and retains its storage until process exit.
    ///
    /// # Safety
    /// Call before any other stdin reads. A mapped input file must not be modified
    /// while its contents or slices returned by this reader are in use.
    /// Subsequent `ScanSource` reads must satisfy this type's token requirements.
    pub unsafe fn stdin() -> Self {
        let mut stdin = File::from(std::io::stdin().as_fd().try_clone_to_owned().unwrap());
        #[cfg(target_os = "linux")]
        unsafe {
            if let Ok(metadata) = stdin.metadata()
                && metadata.is_file()
                && metadata.len() != 0
            {
                let len = metadata.len() as usize;
                let page = getpagesize() as usize;
                let mapped = len.div_ceil(page) * page;
                let reserved = mapped + page;
                // MAP_PRIVATE | MAP_ANONYMOUS reserves an initialized page after EOF.
                let region = mmap(ptr::null_mut(), reserved, 1, 2 | 0x20, -1, 0);
                if region as isize != -1 {
                    // MAP_FIXED replaces only the file portion of our own reservation.
                    if mmap(region, len, 1, 2 | 0x10, 0, 0) as isize != -1 {
                        return FastInput {
                            ptr: region.cast(),
                            end: region.cast::<u8>().add(len),
                        };
                    }
                    assert_eq!(munmap(region, reserved), 0);
                }
            }
        }
        let mut buf = Vec::new();
        stdin.read_to_end(&mut buf).unwrap();
        let len = buf.len();
        buf.resize(len + 16, b' ');
        let ptr = Box::into_raw(buf.into_boxed_slice()).cast::<u8>();
        FastInput {
            ptr,
            end: unsafe { ptr.add(len) },
        }
    }

    /// # Safety
    /// `s` must contain at least 16 initialized padding bytes after its final
    /// token delimiter. Its allocation must remain valid and unchanged while
    /// this reader or any slices returned by it are in use.
    /// Subsequent `ScanSource` reads must satisfy this type's token requirements.
    pub unsafe fn from_slice(s: &[u8]) -> Self {
        FastInput {
            ptr: s.as_ptr(),
            end: unsafe { s.as_ptr().add(s.len() - 16) },
        }
    }

    /// Skips ASCII whitespace without advancing beyond the input.
    #[inline]
    pub fn skip_whitespace(&mut self) {
        unsafe {
            while self.ptr < self.end && (*self.ptr).is_ascii_whitespace() {
                self.ptr = self.ptr.add(1);
            }
        }
    }

    #[inline]
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

    #[inline]
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

    #[inline]
    pub unsafe fn u8(&mut self) -> u8 {
        unsafe { self.fetch_ud4() as u8 }
    }

    #[inline]
    pub unsafe fn u16(&mut self) -> u16 {
        unsafe { self.fetch_ud8() as u16 }
    }

    /// 0..=99_999_999
    #[inline]
    pub unsafe fn u32_small(&mut self) -> u32 {
        unsafe { self.fetch_ud8() }
    }

    #[inline]
    pub unsafe fn u32(&mut self) -> u32 {
        unsafe {
            let mut x = u64::from_le(ptr::read_unaligned(self.ptr.cast())) ^ 0x3030303030303030;
            let mask = x & 0xf0f0f0f0f0f0f0f0;
            if mask != 0 {
                let len = mask.trailing_zeros() >> 3;
                x <<= 64 - len * 8;
                self.ptr = self.ptr.add(len as usize + 1);
                x = x.wrapping_mul(10).wrapping_add(x >> 8) & 0x00ff00ff00ff00ff;
                x = x.wrapping_mul(100).wrapping_add(x >> 16) & 0x0000ffff0000ffff;
                x = x.wrapping_mul(10000).wrapping_add(x >> 32) & 0xffffffff;
                x as u32
            } else {
                x = x.wrapping_mul(10).wrapping_add(x >> 8) & 0x00ff00ff00ff00ff;
                x = x.wrapping_mul(100).wrapping_add(x >> 16) & 0x0000ffff0000ffff;
                x = x.wrapping_mul(10000).wrapping_add(x >> 32) & 0xffffffff;
                let y = u16::from_le(ptr::read_unaligned(self.ptr.add(8).cast())) ^ 0x3030;
                if y & 0xf0f0 == 0 {
                    self.ptr = self.ptr.add(11);
                    x as u32 * 100 + ((y.wrapping_mul(10).wrapping_add(y >> 8)) & 0xff) as u32
                } else if y & 0xf0 == 0 {
                    self.ptr = self.ptr.add(10);
                    x as u32 * 10 + (y & 0xff) as u32
                } else {
                    self.ptr = self.ptr.add(9);
                    x as u32
                }
            }
        }
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "ssse3"))]
    #[inline]
    pub unsafe fn u64(&mut self) -> u64 {
        unsafe { self.u64_simd::<20>() }
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "ssse3"))]
    #[inline]
    unsafe fn u64_simd<const MAX_DIGITS: usize>(&mut self) -> u64 {
        use std::arch::x86_64::*;
        #[inline]
        unsafe fn parse16(digits: __m128i) -> u64 {
            unsafe {
                let pairs = _mm_maddubs_epi16(digits, _mm_set1_epi16(0x010a));
                let quads = _mm_madd_epi16(pairs, _mm_set1_epi32(0x00010064));
                let octets = _mm_add_epi64(
                    _mm_mul_epu32(quads, _mm_set1_epi64x(10000)),
                    _mm_srli_epi64::<32>(quads),
                );
                (_mm_cvtsi128_si64(octets) as u64) * 100000000
                    + (_mm_cvtsi128_si64(_mm_srli_si128::<8>(octets)) as u64)
            }
        }
        const SHUFFLE: [[u8; 16]; 16] = const {
            let mut table = [[128; 16]; 16];
            let mut n = 1;
            while n < 16 {
                let mut i = 16 - n;
                while i < 16 {
                    table[n][i] = (i + n - 16) as u8;
                    i += 1;
                }
                n += 1;
            }
            table
        };
        unsafe {
            let mut digits =
                _mm_sub_epi8(_mm_loadu_si128(self.ptr.cast()), _mm_set1_epi8(b'0' as i8));
            let mask = _mm_movemask_epi8(digits) as u32;
            if mask != 0 {
                let len = mask.trailing_zeros();
                digits = _mm_shuffle_epi8(
                    digits,
                    _mm_loadu_si128(SHUFFLE[len as usize].as_ptr().cast()),
                );
                self.ptr = self.ptr.add(len as usize + 1);
                return parse16(digits);
            }
            self.ptr = self.ptr.add(16);
            let mut res = parse16(digits);
            let mut rem = ptr::read_unaligned(self.ptr.cast::<u32>()) ^ 0x30303030;
            if (rem & 0xf0f0f0) == 0 {
                if MAX_DIGITS == 19 {
                    res = res.wrapping_mul(1000).wrapping_add(
                        ((rem & 0xff) as u64)
                            .wrapping_mul(100)
                            .wrapping_add((((rem.wrapping_mul(2561)) & 0xff0000) >> 16) as u64),
                    );
                    self.ptr = self.ptr.add(4);
                } else {
                    let four = (rem & 0xf0f0f0f0) == 0;
                    rem = rem.wrapping_shl((!four as u32) << 3);
                    rem = rem.wrapping_mul(10).wrapping_add(rem >> 8) & 0x00ff00ff;
                    rem = rem.wrapping_mul(100).wrapping_add(rem >> 16) & 0x0000ffff;
                    res = res
                        .wrapping_mul(1000 + 9000 * four as u64)
                        .wrapping_add(rem as u64);
                    self.ptr = self.ptr.add(4 + four as usize);
                }
            } else if (rem & 0xf0f0) == 0 {
                res = res
                    .wrapping_mul(100)
                    .wrapping_add((((rem >> 8).wrapping_add(rem.wrapping_mul(10))) & 0xff) as u64);
                self.ptr = self.ptr.add(3);
            } else if (rem & 0xf0) == 0 {
                res = res.wrapping_mul(10).wrapping_add((rem & 0x0000000f) as u64);
                self.ptr = self.ptr.add(2);
            } else {
                self.ptr = self.ptr.add(1);
            }
            res
        }
    }

    #[cfg(not(all(target_arch = "x86_64", target_feature = "ssse3")))]
    #[inline]
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
                        res = res.wrapping_mul(10).wrapping_add(y & 0xff);
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

    #[inline]
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
            if (x & 0xf0f0f0f0) == 0 {
                rem >>= 32;
                x = x.wrapping_mul(10).wrapping_add(x >> 8) & 0x00ff00ff;
                x = x.wrapping_mul(100).wrapping_add(x >> 16) & 0x0000ffff;
                res2 = x;
                pow = 10000;
                self.ptr = self.ptr.add(4);
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
                    res2 = res2.wrapping_mul(10).wrapping_add(rem & 0xff);
                    pow = pow.wrapping_mul(10);
                }
                self.ptr = self.ptr.add(x as usize + 1);
            }
            res = res.wrapping_mul(pow as u128).wrapping_add(res2 as u128);
            res
        }
    }

    #[inline]
    pub unsafe fn usize(&mut self) -> usize {
        unsafe { self.u64() as usize }
    }

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
    pub unsafe fn i64(&mut self) -> i64 {
        unsafe {
            let b = *self.ptr == b'-';
            self.ptr = self.ptr.add(b as usize);
            #[cfg(all(target_arch = "x86_64", target_feature = "ssse3"))]
            let mut x = self.u64_simd::<19>() as i64;
            #[cfg(not(all(target_arch = "x86_64", target_feature = "ssse3")))]
            let mut x = self.u64() as i64;
            if b {
                x = x.wrapping_neg();
            }
            x
        }
    }

    #[inline]
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

    #[inline]
    pub unsafe fn isize(&mut self) -> isize {
        unsafe { self.i64() as isize }
    }

    #[inline]
    pub unsafe fn byte(&mut self) -> u8 {
        unsafe {
            let c = *self.ptr;
            self.ptr = self.ptr.add(2);
            c
        }
    }

    #[inline]
    pub unsafe fn bytes<'a>(&mut self) -> &'a [u8] {
        unsafe {
            let start = self.ptr;
            loop {
                // Padding permits this load; bytes above ASCII space cannot be separators.
                let x = self.ptr.cast::<u64>().read_unaligned();
                if x.wrapping_sub(0x2121_2121_2121_2121) & !x & 0x8080_8080_8080_8080 != 0 {
                    break;
                }
                self.ptr = self.ptr.add(8);
            }
            while !(*self.ptr).is_ascii_whitespace() {
                self.ptr = self.ptr.add(1);
            }
            let len = self.ptr.offset_from(start) as usize;
            self.ptr = self.ptr.add(1);
            std::slice::from_raw_parts(start, len)
        }
    }

    #[inline]
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

// The top two bits store the digit count minus one; the rest holds little-endian ASCII.
static DIGIT4_TRIMMED: [u32; 10000] = const {
    let mut arr = [0; 10000];
    let mut i = 0;
    while i < 10000 {
        let off = (i < 10) as usize + (i < 100) as usize + (i < 1000) as usize;
        arr[i] = (u32::from_le_bytes(DIGIT4[i]) >> (8 * off)) | (((3 - off) as u32) << 30);
        i += 1;
    }
    arr
};

pub struct FastOutput<W>
where
    W: Write,
{
    buf: Box<[u8]>,
    pos: usize,
    inner: W,
}

impl FastOutput<StdoutLock<'static>> {
    pub fn stdout() -> Self {
        Self::new(stdout().lock())
    }
}

impl<W> Drop for FastOutput<W>
where
    W: Write,
{
    fn drop(&mut self) {
        let _ = self.inner.write_all(&self.buf[..self.pos]);
    }
}

impl<W> FastOutput<W>
where
    W: Write,
{
    pub fn new(writer: W) -> Self {
        Self::with_capacity(1 << 18, writer)
    }

    pub fn with_capacity(capacity: usize, writer: W) -> Self {
        FastOutput {
            buf: vec![0; capacity.max(32)].into_boxed_slice(),
            pos: 0,
            inner: writer,
        }
    }

    pub fn flush(&mut self) {
        self.flush_buf();
        self.inner.flush().unwrap();
    }

    #[cold]
    fn flush_buf(&mut self) {
        if self.pos != 0 {
            self.inner.write_all(&self.buf[..self.pos]).unwrap();
            self.pos = 0;
        }
    }

    #[inline]
    fn ensure_capacity(&mut self, capacity: usize) {
        if self.buf.len() - self.pos < capacity {
            self.flush_buf();
        }
    }

    #[inline]
    unsafe fn write_byte_unchecked(&mut self, byte: u8) {
        unsafe {
            *self.buf.as_mut_ptr().add(self.pos) = byte;
        }
        self.pos += 1;
    }

    #[inline]
    unsafe fn write_digit4_unchecked(&mut self, x: usize) {
        debug_assert!(x < 10000);
        unsafe {
            ptr::write_unaligned(
                self.buf.as_mut_ptr().add(self.pos) as *mut u32,
                ptr::read_unaligned((DIGIT4.as_ptr() as *const u8).add(4 * x) as *const u32),
            );
        }
        self.pos += 4;
    }

    #[inline]
    unsafe fn write_digit4_trimmed_unchecked(&mut self, x: usize) {
        unsafe {
            let word = *DIGIT4_TRIMMED.get_unchecked(x);
            ptr::write_unaligned(
                self.buf.as_mut_ptr().add(self.pos).cast::<u32>(),
                (word & 0x3fffffff).to_le(),
            );
            self.pos += (word >> 30) as usize + 1;
        }
    }

    #[inline]
    unsafe fn write_u8_unchecked(&mut self, x: u8) {
        let off = (x < 10) as usize + (x < 100) as usize + 1;
        unsafe {
            ptr::write_unaligned(
                self.buf.as_mut_ptr().add(self.pos) as *mut u32,
                ptr::read_unaligned(
                    (DIGIT4.as_ptr() as *const u8).add(4 * x as usize + off) as *const u32
                ),
            );
        }
        self.pos += 4 - off;
    }

    #[inline]
    unsafe fn write_u16_unchecked(&mut self, x: u16) {
        unsafe {
            if x >= 10000 {
                self.write_digit4_trimmed_unchecked((x / 10000) as usize);
                self.write_digit4_unchecked((x % 10000) as usize);
            } else {
                self.write_digit4_trimmed_unchecked(x as usize);
            }
        }
    }

    #[inline]
    unsafe fn write_u32_unchecked(&mut self, x: u32) {
        unsafe {
            if x >= 1_0000_0000 {
                let b = x / 10000;
                let a = b / 10000;
                self.write_digit4_trimmed_unchecked(a as usize);
                self.write_digit4_unchecked((b % 10000) as usize);
                self.write_digit4_unchecked((x % 10000) as usize);
            } else if x >= 10000 {
                self.write_digit4_trimmed_unchecked((x / 10000) as usize);
                self.write_digit4_unchecked((x % 10000) as usize);
            } else {
                self.write_digit4_trimmed_unchecked(x as usize);
            }
        }
    }

    #[inline(always)]
    unsafe fn write_u64_unchecked(&mut self, x: u64) {
        unsafe {
            if x < 10000 {
                self.write_digit4_trimmed_unchecked(x as usize);
                return;
            }
            if x >= 1_0000_0000_0000_0000 {
                let d = x / 10000;
                let c = x / 100000000;
                let b = x / 1000000000000;
                let a = x / 10000000000000000;
                self.write_digit4_trimmed_unchecked(a as usize);
                self.write_digit4_unchecked((b - a * 10000) as usize);
                self.write_digit4_unchecked((c - b * 10000) as usize);
                self.write_digit4_unchecked((d - c * 10000) as usize);
                self.write_digit4_unchecked((x % 10000) as usize);
            } else if x >= 1_0000_0000_0000 {
                let c = x / 10000;
                let b = x / 100000000;
                let a = x / 1000000000000;
                self.write_digit4_trimmed_unchecked(a as usize);
                self.write_digit4_unchecked((b - a * 10000) as usize);
                self.write_digit4_unchecked((c - b * 10000) as usize);
                self.write_digit4_unchecked((x % 10000) as usize);
            } else if x >= 1_0000_0000 {
                let b = x / 10000;
                let a = x / 100000000;
                self.write_digit4_trimmed_unchecked(a as usize);
                self.write_digit4_unchecked((b - a * 10000) as usize);
                self.write_digit4_unchecked((x % 10000) as usize);
            } else {
                self.write_digit4_trimmed_unchecked((x / 10000) as usize);
                self.write_digit4_unchecked((x % 10000) as usize);
            }
        }
    }

    #[inline]
    pub fn u8(&mut self, x: u8) {
        self.ensure_capacity(4);
        unsafe { self.write_u8_unchecked(x) }
    }

    #[inline]
    pub fn u16(&mut self, x: u16) {
        self.ensure_capacity(5);
        unsafe { self.write_u16_unchecked(x) }
    }

    #[inline]
    pub fn u32(&mut self, x: u32) {
        self.ensure_capacity(10);
        unsafe { self.write_u32_unchecked(x) }
    }

    #[inline(always)]
    pub fn u64(&mut self, x: u64) {
        self.ensure_capacity(20);
        unsafe { self.write_u64_unchecked(x) }
    }

    #[inline]
    pub fn i8(&mut self, x: i8) {
        if x < 0 {
            self.ensure_capacity(5);
            unsafe {
                self.write_byte_unchecked(b'-');
                self.write_u8_unchecked(x.wrapping_neg() as u8);
            }
        } else {
            self.u8(x as u8);
        }
    }

    #[inline]
    pub fn i16(&mut self, x: i16) {
        if x < 0 {
            self.ensure_capacity(6);
            unsafe {
                self.write_byte_unchecked(b'-');
                self.write_u16_unchecked(x.wrapping_neg() as u16);
            }
        } else {
            self.u16(x as u16);
        }
    }

    #[inline]
    pub fn i32(&mut self, x: i32) {
        if x < 0 {
            self.ensure_capacity(11);
            unsafe {
                self.write_byte_unchecked(b'-');
                self.write_u32_unchecked(x.wrapping_neg() as u32);
            }
        } else {
            self.u32(x as u32);
        }
    }

    #[inline(always)]
    pub fn i64(&mut self, x: i64) {
        if x < 0 {
            self.ensure_capacity(21);
            unsafe {
                self.write_byte_unchecked(b'-');
                self.write_u64_unchecked(x.wrapping_neg() as u64);
            }
        } else {
            self.u64(x as u64);
        }
    }

    #[inline(always)]
    pub fn usize(&mut self, x: usize) {
        if usize::BITS == 64 {
            self.u64(x as u64);
        } else {
            self.u32(x as u32);
        }
    }

    #[inline(always)]
    pub fn isize(&mut self, x: isize) {
        if isize::BITS == 64 {
            self.i64(x as i64);
        } else {
            self.i32(x as i32);
        }
    }

    pub fn u128(&mut self, mut x: u128) {
        const BASE: u128 = 10_000_000_000_000_000;
        let mut groups = [0u64; 2];
        let mut len = 0;
        while x > u64::MAX as u128 {
            groups[len] = (x % BASE) as u64;
            x /= BASE;
            len += 1;
        }
        self.u64(x as u64);
        for &x in groups[..len].iter().rev() {
            self.ensure_capacity(16);
            unsafe {
                self.write_digit4_unchecked((x / 1_000_000_000_000) as usize);
                self.write_digit4_unchecked((x / 100_000_000 % 10000) as usize);
                self.write_digit4_unchecked((x / 10000 % 10000) as usize);
                self.write_digit4_unchecked((x % 10000) as usize);
            }
        }
    }

    pub fn i128(&mut self, x: i128) {
        if x < 0 {
            self.byte(b'-');
        }
        self.u128(x.unsigned_abs());
    }

    #[inline]
    pub fn byte(&mut self, b: u8) {
        self.ensure_capacity(1);
        unsafe { self.write_byte_unchecked(b) }
    }

    #[inline(always)]
    pub fn bytes(&mut self, s: &[u8]) {
        if s.len() > self.buf.len() {
            self.flush_buf();
            self.inner.write_all(s).unwrap();
        } else {
            self.ensure_capacity(s.len());
            unsafe {
                ptr::copy_nonoverlapping(s.as_ptr(), self.buf.as_mut_ptr().add(self.pos), s.len());
            }
            self.pos += s.len();
        }
    }
}

impl<W: Write> fmt::Write for FastOutput<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.bytes(s.as_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn test_fast_input_decimal_boundaries() {
        macro_rules! check {
            ($ty:ty, $method:ident, $max:expr) => {{
                let mut values: Vec<$ty> = vec![0, 1, $max];
                let mut power: $ty = 1;
                while let Some(next) = power.checked_mul(10).filter(|&x| x <= $max) {
                    values.extend([next - 1, next]);
                    if next < $max {
                        values.push(next + 1);
                    }
                    power = next;
                }
                for offset in 0..32 {
                    let mut input = vec![b' '; offset];
                    for (i, x) in values.iter().enumerate() {
                        write!(input, "{}{}", x, if i % 2 == 0 { ' ' } else { '\n' }).unwrap();
                    }
                    input.extend_from_slice(&[b' '; 32]);
                    let mut fi = unsafe { FastInput::from_slice(&input[offset..]) };
                    for &x in &values {
                        assert_eq!(unsafe { fi.$method() }, x, "offset={offset}");
                    }
                }
            }};
        }
        check!(u8, u8, u8::MAX);
        check!(u16, u16, u16::MAX);
        check!(u32, u32_small, 99_999_999);
        check!(u32, u32, u32::MAX);
        check!(u64, u64, u64::MAX);
        check!(u128, u128, u128::MAX);
    }

    #[test]
    fn test_fast_output_buffer_boundaries() {
        let mut rng = Xorshift::default();
        for capacity in 0..=80 {
            let mut output = Vec::new();
            let mut expected = String::new();
            {
                let mut fo = FastOutput::with_capacity(capacity, &mut output);
                for i in 0..1000 {
                    let x = match i % 5 {
                        0 => 0,
                        1 => u64::MAX,
                        2 => i64::MIN as u64,
                        3 => 10u64.pow(rng.random(0..=19)) - 1,
                        _ => rng.rand64(),
                    };
                    fo.u64(x);
                    fo.byte(b' ');
                    fo.i64(x as i64);
                    fo.byte(b'\n');
                    expected.push_str(&format!("{} {}\n", x, x as i64));
                }
            }
            assert_eq!(output, expected.as_bytes(), "capacity={capacity}");
        }
    }

    #[test]
    fn test_input_bytes() {
        let mut rng = Xorshift::new_with_seed(612971);
        for _ in 0..512 {
            let offset = rng.random(0..64);
            let mut input: Vec<u8> = (0..offset).map(|_| rng.random(..)).collect();
            let mut expected = Vec::new();
            for _ in 0..rng.random(1..32) {
                let len = if rng.rand(4) == 0 {
                    rng.random(0..10000)
                } else {
                    rng.random(0..128)
                };
                let ascii = rng.rand(2) == 0;
                let token: Vec<u8> = (0..len)
                    .map(|_| {
                        loop {
                            let byte: u8 = if ascii {
                                rng.random(33..127)
                            } else {
                                rng.random(..)
                            };
                            if !byte.is_ascii_whitespace() {
                                break byte;
                            }
                        }
                    })
                    .collect();
                input.extend_from_slice(&token);
                input.push([b' ', b'\t', b'\n', b'\r', 12][rng.random(0usize..5)]);
                expected.push(token);
            }
            input.extend([b' '; 16]);
            // SAFETY: input remains alive and every field has a delimiter followed by padding.
            let mut reader = unsafe { FastInput::from_slice(&input[offset..]) };
            for token in expected {
                // SAFETY: byte fields need not be UTF-8; each read consumes one delimited field.
                assert_eq!(unsafe { reader.bytes() }, token);
            }
        }
    }

    #[test]
    fn test_past_input_u8() {
        let mut a = vec![];
        let mut s = String::new();
        for i in 0..=u8::MAX {
            a.push(i);
            s.push_str(&format!("{}\n", i));
        }
        s.push_str("                                ");
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
        s.push_str("                                ");
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
        s.push_str("                                ");
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
        s.push_str("                                ");
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
        s.push_str("                                ");
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
        s.push_str("                                ");
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
        s.push_str("                                ");
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
        s.push_str("                                ");
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
        s.push_str("                                ");
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
        s.push_str("                                ");
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
        fo.flush();
        let s = std::str::from_utf8(&fo.inner).unwrap();
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
        fo.flush();
        let s = std::str::from_utf8(&fo.inner).unwrap();
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
        fo.flush();
        let s = std::str::from_utf8(&fo.inner).unwrap();
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
        fo.flush();
        let s = std::str::from_utf8(&fo.inner).unwrap();
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
        fo.flush();
        let s = std::str::from_utf8(&fo.inner).unwrap();
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
        fo.flush();
        let s = std::str::from_utf8(&fo.inner).unwrap();
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
        fo.flush();
        let s = std::str::from_utf8(&fo.inner).unwrap();
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
        fo.flush();
        let s = std::str::from_utf8(&fo.inner).unwrap();
        let mut lines = s.lines();
        for &a in &a {
            let line = lines.next().unwrap();
            assert_eq!(line, a.to_string());
        }
    }
}

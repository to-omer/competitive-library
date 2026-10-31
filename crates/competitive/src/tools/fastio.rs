use super::ScanSource;
use std::{
    fmt,
    fs::File,
    io::{Read, Write, stdout},
    marker::PhantomData,
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
pub struct FastInput<'input> {
    ptr: *const u8,
    end: *const u8,
    _marker: PhantomData<&'input [u8]>,
}

impl<'input> FastInput<'input> {
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
                            _marker: PhantomData,
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
            _marker: PhantomData,
        }
    }

    /// # Safety
    /// `s` must contain at least 16 initialized padding bytes after its final
    /// token delimiter. Its allocation must remain valid and unchanged while
    /// this reader or any slices returned by it are in use.
    /// Subsequent `ScanSource` reads must satisfy this type's token requirements.
    pub unsafe fn from_slice(s: &'input [u8]) -> Self {
        FastInput {
            ptr: s.as_ptr(),
            end: unsafe { s.as_ptr().add(s.len() - 16) },
            _marker: PhantomData,
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
                self.fetch_ud8()
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

    /// Appends `count` decimal integers to `values`, retaining its reserved capacity.
    ///
    /// # Safety
    /// The next `count` tokens must satisfy this reader's `u32` input requirements.
    pub unsafe fn extend_u32(&mut self, values: &mut Vec<u32>, count: usize) {
        values.reserve(count);
        let end = values.len() + count;
        unsafe {
            #[cfg(target_arch = "x86_64")]
            if count >= 544 && std::is_x86_feature_detected!("avx2") {
                self.extend_u32_avx2(values, count);
            }
            while values.len() < end {
                values.push(self.u32());
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn extend_u32_avx2(&mut self, values: &mut Vec<u32>, count: usize) {
        use std::arch::x86_64::*;
        const CHUNK: usize = 1 << 17;
        const CAPACITY: usize = CHUNK / 8 + 16;
        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn two_tokens(input: &mut *const u8) -> __m256i {
            unsafe {
                let bytes = _mm256_loadu_si256((*input).cast());
                let separators =
                    _mm256_movemask_epi8(_mm256_cmpgt_epi8(_mm256_set1_epi8(33), bytes)) as u32;
                let first = separators.trailing_zeros() as usize;
                let second = (separators & (separators - 1)).trailing_zeros() as usize;
                let windows = _mm256_inserti128_si256::<1>(
                    bytes,
                    _mm_loadu_si128((*input).add(first + 1).cast()),
                );
                let rows = _mm256_set_m128i(
                    _mm_loadu_si128(FastInput::SHUFFLE.as_ptr().add(second - first - 1).cast()),
                    _mm_loadu_si128(FastInput::SHUFFLE.as_ptr().add(first).cast()),
                );
                *input = (*input).add(second + 1);
                let digits = _mm256_shuffle_epi8(
                    _mm256_subs_epu8(windows, _mm256_set1_epi8(b'0' as i8)),
                    rows,
                );
                _mm256_madd_epi16(
                    _mm256_maddubs_epi16(digits, _mm256_set1_epi16(0x010a)),
                    _mm256_set1_epi32(0x00010064),
                )
            }
        }
        unsafe {
            let start = values.len();
            let mut scratch = Vec::<u32>::with_capacity(3 * CAPACITY);
            let scratch = scratch.as_mut_ptr();
            let mut written = 0;
            while count - written >= 544 {
                // At least two bytes per token; the extra 32 tokens cover boundary alignment.
                let chunk = CHUNK.min(2 * (count - written - 32)) & !3;
                let mut end = [self.ptr; 4];
                for (k, end) in end.iter_mut().enumerate() {
                    let mut p = self.ptr.add(chunk * (k + 1) / 4);
                    while *p > b' ' {
                        p = p.add(1);
                    }
                    *end = p.add(1);
                }
                let mut input = [self.ptr, end[0], end[1], end[2]];
                let output = [
                    values.as_mut_ptr().add(start + written),
                    scratch,
                    scratch.add(CAPACITY),
                    scratch.add(2 * CAPACITY),
                ];
                let mut done = 0;
                loop {
                    let left = (0..4).map(|k| end[k].offset_from(input[k])).min().unwrap();
                    if left < 33 {
                        break;
                    }
                    // Each pair consumes at most 22 bytes; leave 32 bytes for its SIMD load.
                    for _ in 0..(left - 32) / 22 + 1 {
                        let g0 = two_tokens(&mut input[0]);
                        let g1 = two_tokens(&mut input[1]);
                        let g2 = two_tokens(&mut input[2]);
                        let g3 = two_tokens(&mut input[3]);
                        let k = _mm256_set1_epi32(0x00012710);
                        let h01 =
                            _mm256_castsi256_ps(_mm256_madd_epi16(_mm256_packus_epi32(g0, g1), k));
                        let h23 =
                            _mm256_castsi256_ps(_mm256_madd_epi16(_mm256_packus_epi32(g2, g3), k));
                        let high = _mm256_castps_si256(_mm256_shuffle_ps::<0x88>(h01, h23));
                        let low = _mm256_castps_si256(_mm256_shuffle_ps::<0xdd>(h01, h23));
                        let v = _mm256_add_epi32(
                            _mm256_mullo_epi32(high, _mm256_set1_epi32(100000000)),
                            low,
                        );
                        let q = _mm256_permutevar8x32_epi32(
                            v,
                            _mm256_setr_epi32(0, 4, 1, 5, 2, 6, 3, 7),
                        );
                        ptr::write_unaligned(
                            output[0].add(done).cast::<u64>(),
                            _mm256_extract_epi64::<0>(q) as u64,
                        );
                        ptr::write_unaligned(
                            output[1].add(done).cast::<u64>(),
                            _mm256_extract_epi64::<1>(q) as u64,
                        );
                        ptr::write_unaligned(
                            output[2].add(done).cast::<u64>(),
                            _mm256_extract_epi64::<2>(q) as u64,
                        );
                        ptr::write_unaligned(
                            output[3].add(done).cast::<u64>(),
                            _mm256_extract_epi64::<3>(q) as u64,
                        );
                        done += 2;
                    }
                }
                let mut lengths = [done; 4];
                for k in 0..4 {
                    self.ptr = input[k];
                    while self.ptr < end[k] {
                        output[k].add(lengths[k]).write(self.u32());
                        lengths[k] += 1;
                    }
                }
                written += lengths[0];
                for k in 1..4 {
                    ptr::copy_nonoverlapping(
                        output[k],
                        values.as_mut_ptr().add(start + written),
                        lengths[k],
                    );
                    written += lengths[k];
                }
                self.ptr = end[3];
            }
            values.set_len(start + written);
        }
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "ssse3"))]
    #[inline]
    pub unsafe fn u64(&mut self) -> u64 {
        unsafe { self.u64_simd::<20>() }
    }

    // Loading at the digit count right-aligns the digits in a 16-byte lane.
    #[cfg(target_arch = "x86_64")]
    const SHUFFLE: [u8; 32] = const {
        let mut table = [128; 32];
        let mut i = 16;
        while i < 32 {
            table[i] = (i - 16) as u8;
            i += 1;
        }
        table
    };

    #[cfg(all(target_arch = "x86_64", target_feature = "ssse3"))]
    #[inline]
    unsafe fn parse_ud16(digits: std::arch::x86_64::__m128i) -> u64 {
        use std::arch::x86_64::*;
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

    #[cfg(all(target_arch = "x86_64", target_feature = "ssse3"))]
    #[inline]
    unsafe fn u64_simd<const MAX_DIGITS: usize>(&mut self) -> u64 {
        use std::arch::x86_64::*;
        unsafe {
            let mut digits =
                _mm_sub_epi8(_mm_loadu_si128(self.ptr.cast()), _mm_set1_epi8(b'0' as i8));
            let mask = _mm_movemask_epi8(digits) as u32;
            if mask != 0 {
                let len = mask.trailing_zeros();
                digits = _mm_shuffle_epi8(
                    digits,
                    _mm_loadu_si128(Self::SHUFFLE.as_ptr().add(len as usize).cast()),
                );
                self.ptr = self.ptr.add(len as usize + 1);
                return Self::parse_ud16(digits);
            }
            self.ptr = self.ptr.add(16);
            let mut res = Self::parse_ud16(digits);
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
                res = self.fetch_ud8() as u64;
            }
            res
        }
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "ssse3"))]
    #[inline]
    pub unsafe fn u128(&mut self) -> u128 {
        use std::arch::x86_64::*;
        const POW10: [u64; 16] = const {
            let mut powers = [1; 16];
            let mut i = 1;
            while i < 16 {
                powers[i] = powers[i - 1] * 10;
                i += 1;
            }
            powers
        };
        unsafe {
            let first = _mm_sub_epi8(_mm_loadu_si128(self.ptr.cast()), _mm_set1_epi8(b'0' as i8));
            let mask = _mm_movemask_epi8(first) as u32;
            if mask != 0 {
                let len = mask.trailing_zeros() as usize;
                self.ptr = self.ptr.add(len + 1);
                return Self::parse_ud16(_mm_shuffle_epi8(
                    first,
                    _mm_loadu_si128(Self::SHUFFLE.as_ptr().add(len).cast()),
                )) as u128;
            }
            let second = _mm_sub_epi8(
                _mm_loadu_si128(self.ptr.add(16).cast()),
                _mm_set1_epi8(b'0' as i8),
            );
            let mask = _mm_movemask_epi8(second) as u32;
            if mask != 0 {
                let len = mask.trailing_zeros() as usize;
                self.ptr = self.ptr.add(17 + len);
                return Self::parse_ud16(first) as u128 * POW10[len] as u128
                    + Self::parse_ud16(_mm_shuffle_epi8(
                        second,
                        _mm_loadu_si128(Self::SHUFFLE.as_ptr().add(len).cast()),
                    )) as u128;
            }
            let mut tail = ptr::read_unaligned(self.ptr.add(32).cast::<u64>()) ^ 0x3030303030303030;
            let len = (tail & 0xf0f0f0f0f0f0f0f0).trailing_zeros() as usize / 8;
            tail = if len == 0 { 0 } else { tail << ((8 - len) * 8) };
            tail = tail.wrapping_mul(10).wrapping_add(tail >> 8) & 0x00ff00ff00ff00ff;
            tail = tail.wrapping_mul(100).wrapping_add(tail >> 16) & 0x0000ffff0000ffff;
            tail = tail.wrapping_mul(10000).wrapping_add(tail >> 32) & 0xffffffff;
            self.ptr = self.ptr.add(33 + len);
            (Self::parse_ud16(first) as u128 * 10_000_000_000_000_000
                + Self::parse_ud16(second) as u128)
                * POW10[len] as u128
                + tail as u128
        }
    }

    #[cfg(not(all(target_arch = "x86_64", target_feature = "ssse3")))]
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
            'token: {
                #[cfg(all(
                    target_arch = "x86_64",
                    any(target_feature = "avx2", target_feature = "avx512bw")
                ))]
                {
                    // Short fields avoid the SIMD mask-to-pointer dependency between reads.
                    let x = self.ptr.cast::<u64>().read_unaligned();
                    if x.wrapping_sub(0x2121_2121_2121_2121) & !x & 0x8080_8080_8080_8080 != 0 {
                        break 'token;
                    }
                }
                #[cfg(all(target_arch = "x86_64", target_feature = "avx512bw"))]
                {
                    use std::arch::x86_64::*;
                    let space = _mm512_set1_epi8(32);
                    // The 16-byte padding contract does not permit a full SIMD load at EOF.
                    while self.end.offset_from(self.ptr) >= 64 {
                        let bytes = _mm512_loadu_si512(self.ptr.cast());
                        let low = _mm512_cmple_epu8_mask(bytes, space);
                        if low != 0 {
                            self.ptr = self.ptr.add(low.trailing_zeros() as usize);
                            break 'token;
                        }
                        self.ptr = self.ptr.add(64);
                    }
                }
                #[cfg(all(
                    target_arch = "x86_64",
                    target_feature = "avx2",
                    not(target_feature = "avx512bw")
                ))]
                {
                    use std::arch::x86_64::*;
                    let space = _mm256_set1_epi8(32);
                    // The public contract guarantees only 16 padding bytes, so full SIMD
                    // loads stay within the input; the scalar loop handles its final token.
                    while self.end.offset_from(self.ptr) >= 32 {
                        let bytes = _mm256_loadu_si256(self.ptr.cast());
                        let low = _mm256_cmpeq_epi8(_mm256_min_epu8(bytes, space), bytes);
                        if _mm256_movemask_epi8(low) != 0 {
                            break;
                        }
                        self.ptr = self.ptr.add(32);
                    }
                }
                loop {
                    // Padding permits this load; bytes above ASCII space cannot be separators.
                    let x = self.ptr.cast::<u64>().read_unaligned();
                    if x.wrapping_sub(0x2121_2121_2121_2121) & !x & 0x8080_8080_8080_8080 != 0 {
                        break 'token;
                    }
                    self.ptr = self.ptr.add(8);
                }
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

/// Buffered token reader with the same token requirements as [`FastInput`].
/// Tokens may span reads and borrowed tokens remain valid until the next scan.
pub struct BufferedInput<R> {
    reader: R,
    buffer: Vec<u8>,
    input: FastInput<'static>,
    filled: usize,
    eof: bool,
}

impl<R: Read> BufferedInput<R> {
    /// # Safety
    /// Every scan must satisfy [`FastInput`]'s token requirements, including UTF-8.
    pub unsafe fn new(reader: R) -> Self {
        let buffer = vec![b' '; (1 << 16) + 16];
        let input = FastInput {
            ptr: buffer.as_ptr(),
            end: buffer.as_ptr(),
            _marker: PhantomData,
        };
        Self {
            reader,
            buffer,
            input,
            filled: 0,
            eof: false,
        }
    }

    #[cold]
    fn refill(&mut self) -> bool {
        if self.eof {
            return false;
        }
        let position = unsafe { self.input.ptr.offset_from(self.buffer.as_ptr()) as usize };
        self.buffer.copy_within(position..self.filled, 0);
        self.filled -= position;
        self.input = FastInput {
            ptr: self.buffer.as_ptr(),
            end: self.buffer.as_ptr(),
            _marker: PhantomData,
        };
        loop {
            if self.filled == self.buffer.len() - 16 {
                self.buffer.resize(2 * self.filled + 16, b' ');
                self.input = FastInput {
                    ptr: self.buffer.as_ptr(),
                    end: self.buffer.as_ptr(),
                    _marker: PhantomData,
                };
            }
            let end = self.buffer.len() - 16;
            let start = self.filled;
            match self.reader.read(&mut self.buffer[self.filled..end]) {
                Ok(0) => {
                    self.eof = true;
                    if self.filled != 0 {
                        self.buffer[self.filled] = b' ';
                        self.filled += 1;
                    }
                }
                Ok(n) => self.filled += n,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => panic!("io error: {e}"),
            }
            let (prefix, chunks) = self.buffer[start..self.filled].as_rchunks();
            let end = prefix.len()
                + 8 * chunks
                    .iter()
                    .rposition(|&chunk| {
                        let x = u64::from_le_bytes(chunk);
                        x.wrapping_sub(0x2121_2121_2121_2121) & !x & 0x8080_8080_8080_8080 != 0
                    })
                    .map_or(0, |i| i + 1);
            if let Some(end) = self.buffer[start..start + end]
                .iter()
                .rposition(u8::is_ascii_whitespace)
            {
                self.buffer[self.filled..self.filled + 16].fill(b' ');
                self.input = FastInput {
                    ptr: self.buffer.as_ptr(),
                    end: unsafe { self.buffer.as_ptr().add(start + end + 1) },
                    _marker: PhantomData,
                };
                return true;
            }
            if self.eof {
                self.input = FastInput {
                    ptr: self.buffer.as_ptr(),
                    end: self.buffer.as_ptr(),
                    _marker: PhantomData,
                };
                return false;
            }
        }
    }
}

macro_rules! impl_buffered_scan_integer {
    ($($ty:ty, $read:ident, $method:ident);* $(;)?) => {$(
        #[inline]
        fn $read(&mut self) -> Option<$ty> {
            if self.input.ptr >= self.input.end && !self.refill() {
                return None;
            }
            Some(unsafe { self.input.$method() })
        }
    )*};
}

impl<R: Read> ScanSource for BufferedInput<R> {
    #[inline]
    fn skip_whitespace(&mut self) {
        loop {
            self.input.skip_whitespace();
            if self.input.ptr < self.input.end || !self.refill() {
                break;
            }
        }
    }

    #[inline]
    fn next_token(&mut self) -> Option<&str> {
        if self.input.ptr >= self.input.end && !self.refill() {
            return None;
        }
        Some(unsafe { std::str::from_utf8_unchecked(self.input.bytes()) })
    }

    impl_buffered_scan_integer!(
        u8, read_u8, u8;
        u16, read_u16, u16;
        u32, read_u32, u32;
        u64, read_u64, u64;
        u128, read_u128, u128;
        usize, read_usize, usize;
        i8, read_i8, i8;
        i16, read_i16, i16;
        i32, read_i32, i32;
        i64, read_i64, i64;
        i128, read_i128, i128;
        isize, read_isize, isize;
    );
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

impl FastOutput<File> {
    pub fn stdout() -> Self {
        let mut stdout = stdout().lock();
        stdout.flush().unwrap();
        Self::new(File::from(stdout.as_fd().try_clone_to_owned().unwrap()))
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
                let a = x / 100000000;
                self.write_u8_unchecked(a as u8);
                self.write_digit4_unchecked((b - a * 10000) as usize);
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

    /// Writes each integer right-aligned in ten ASCII columns, followed by a space.
    pub fn u32_slice_padded(&mut self, values: &[u32]) {
        #[cfg(target_arch = "x86_64")]
        let written = if self.buf.len() >= 92 && std::is_x86_feature_detected!("avx2") {
            unsafe { self.u32_slice_padded_avx2(values) }
        } else {
            0
        };
        #[cfg(not(target_arch = "x86_64"))]
        let written = 0;
        for &value in &values[written..] {
            let len = value.checked_ilog10().map_or(1, |x| x as usize + 1);
            self.bytes(&[b' '; 9][..10 - len]);
            self.u32(value);
            self.byte(b' ');
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn u32_slice_padded_avx2(&mut self, values: &[u32]) -> usize {
        use std::arch::x86_64::*;
        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn divide<const MAGIC: i64, const SHIFT: i32>(x: __m256i) -> __m256i {
            let m = _mm256_set1_epi64x(MAGIC);
            let even = _mm256_srli_epi64::<SHIFT>(_mm256_mul_epu32(x, m));
            let odd = _mm256_srli_epi64::<SHIFT>(_mm256_mul_epu32(_mm256_srli_epi64::<32>(x), m));
            _mm256_blend_epi32::<170>(even, _mm256_slli_epi64::<32>(odd))
        }
        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn two_digits(x: __m256i) -> __m256i {
            let tens = _mm256_mulhi_epu16(x, _mm256_set1_epi16(6554));
            _mm256_sub_epi16(
                _mm256_slli_epi16::<8>(x),
                _mm256_mullo_epi16(tens, _mm256_set1_epi16(2559)),
            )
        }
        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn text(digits: __m256i) -> __m256i {
            let leading = _mm256_xor_si256(digits, _mm256_sub_epi64(digits, _mm256_set1_epi64x(1)));
            _mm256_blendv_epi8(
                _mm256_or_si256(digits, _mm256_set1_epi8(b'0' as i8)),
                _mm256_set1_epi8(b' ' as i8),
                leading,
            )
        }
        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn divide8(p: *const u32) -> (__m256i, __m256i) {
            unsafe {
                let v = _mm256_loadu_si256(p.cast());
                let high = divide::<1374389535, 37>(v); // v / 100
                let top = divide::<1125899907, 50>(v); // v / 1_000_000
                let halves = _mm256_add_epi32(
                    _mm256_slli_epi32::<16>(high),
                    _mm256_mullo_epi32(top, _mm256_set1_epi32(1 - 10000 * 65536)),
                );
                let remainder =
                    _mm256_sub_epi32(v, _mm256_mullo_epi32(high, _mm256_set1_epi32(100)));
                let tail = _mm256_or_si256(two_digits(remainder), _mm256_set1_epi32(0x203030));
                let tiny = _mm256_cmpeq_epi32(_mm256_min_epu32(v, _mm256_set1_epi32(9)), v);
                let tail = _mm256_sub_epi32(tail, _mm256_and_si256(tiny, _mm256_set1_epi32(16)));
                (halves, tail)
            }
        }
        const fn mask(chunk: usize, source: usize) -> [u8; 32] {
            // Each lane emits four 11-byte fields from two digit vectors and one tail vector.
            let mut mask = [128; 32];
            let mut i = 0;
            while i < 32 {
                let pos = chunk * 16 + i % 16;
                let field = pos / 11;
                let digit = pos % 11;
                if field < 4 && (if digit < 8 { field / 2 } else { 2 }) == source {
                    mask[i] = (if digit < 8 {
                        field % 2 * 8 + digit
                    } else {
                        field * 4 + digit - 8
                    }) as u8;
                }
                i += 1;
            }
            mask
        }
        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn shuffle<const C: usize, const S: usize>(v: __m256i) -> __m256i {
            unsafe {
                _mm256_shuffle_epi8(v, _mm256_loadu_si256(const { mask(C, S) }.as_ptr().cast()))
            }
        }
        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn store8(p: *mut u8, halves: __m256i, tail: __m256i) {
            unsafe {
                let hundreds = {
                    // Keep the 16-bit multiply; LLVM otherwise widens it around the following shift.
                    let q;
                    std::arch::asm!(
                        "vpmulhuw {q}, {x}, {m}",
                        q = lateout(ymm_reg) q,
                        x = in(ymm_reg) halves,
                        m = in(ymm_reg) _mm256_set1_epi16(5243),
                        options(pure, nomem, nostack, preserves_flags),
                    );
                    _mm256_srli_epi16::<3>(q)
                };
                let rest =
                    _mm256_sub_epi16(halves, _mm256_mullo_epi16(hundreds, _mm256_set1_epi16(100)));
                let lo = text(two_digits(_mm256_unpacklo_epi16(hundreds, rest)));
                let hi = text(two_digits(_mm256_unpackhi_epi16(hundreds, rest)));
                let c0 = _mm256_or_si256(shuffle::<0, 0>(lo), shuffle::<0, 2>(tail));
                let c1 = _mm256_or_si256(
                    _mm256_or_si256(shuffle::<1, 0>(lo), shuffle::<1, 1>(hi)),
                    shuffle::<1, 2>(tail),
                );
                let c2 = _mm256_or_si256(shuffle::<2, 1>(hi), shuffle::<2, 2>(tail));
                // Place high lanes first; the stores also touch four unused trailing bytes.
                _mm256_storeu_si256(p.add(60).cast(), c2);
                _mm256_storeu_si256(p.add(44).cast(), c1);
                _mm256_storeu_si256(p.add(28).cast(), c0);
                _mm_storeu_si128(p.cast(), _mm256_castsi256_si128(c0));
                _mm_storeu_si128(p.add(16).cast(), _mm256_castsi256_si128(c1));
                _mm_storel_epi64(p.add(32).cast(), _mm256_castsi256_si128(c2));
                ptr::write_unaligned(
                    p.add(40).cast(),
                    _mm_extract_epi32::<2>(_mm256_castsi256_si128(c2)),
                );
            }
        }
        let count = values.len() / 8 * 8;
        unsafe {
            let mut i = 0;
            while i < count {
                self.ensure_capacity(92);
                let n = ((self.buf.len() - self.pos - 4) / 88 * 8).min(count - i);
                let mut p = self.buf.as_mut_ptr().add(self.pos);
                let mut previous = divide8(values.as_ptr().add(i));
                for j in (8..n).step_by(8) {
                    let next = divide8(values.as_ptr().add(i + j));
                    store8(p, previous.0, previous.1);
                    previous = next;
                    p = p.add(88);
                }
                store8(p, previous.0, previous.1);
                i += n;
                self.pos += n * 11;
            }
        }
        count
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
        const BASE: u128 = 10_000_000_000_000_000_000;
        let mut groups = [0u64; 2];
        let mut len = 0;
        while x > u64::MAX as u128 {
            groups[len] = (x % BASE) as u64;
            x /= BASE;
            len += 1;
        }
        self.u64(x as u64);
        for &x in groups[..len].iter().rev() {
            self.ensure_capacity(19);
            unsafe {
                ptr::copy_nonoverlapping(
                    DIGIT4[(x / 10_000_000_000_000_000) as usize]
                        .as_ptr()
                        .add(1),
                    self.buf.as_mut_ptr().add(self.pos),
                    3,
                );
                self.pos += 3;
                self.write_digit4_unchecked((x / 1_000_000_000_000 % 10000) as usize);
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

#[cfg(kani)]
mod verification {
    use super::*;

    macro_rules! proof_integer_input {
        ($name:ident, $ty:ident, $digits:expr, $negative:expr) => {
            #[kani::proof]
            #[kani::unwind(6)]
            fn $name() {
                let bytes: [u8; 40] = kani::any();
                let len = kani::any::<u8>() as usize;
                let offset = kani::any::<u8>() as usize;
                let negative: bool = $negative;
                let start = offset + negative as usize;
                kani::assume((1..=$digits).contains(&len) && offset < 8);
                kani::assume(!negative || bytes[offset] == b'-');
                kani::assume(bytes[start + len].is_ascii_whitespace());
                let mut expected = 0i64;
                for i in 0..len {
                    let digit = bytes[start + i];
                    kani::assume(digit.is_ascii_digit());
                    expected = expected * 10 + (digit - b'0') as i64;
                }
                if negative {
                    expected = -expected;
                }
                kani::assume(($ty::MIN as i64..=$ty::MAX as i64).contains(&expected));
                // One valid token, one delimiter, and 16 initialized padding bytes.
                let mut input =
                    unsafe { FastInput::from_slice(&bytes[offset..start + len + 17]) };
                let actual = unsafe { input.$ty() };
                kani::assert(
                    actual as i64 == expected,
                    concat!(
                        "For every valid ", stringify!($ty), " decimal token of one through ",
                        stringify!($digits),
                        " digits, with any ASCII whitespace delimiter and initialized padding, the little-endian SWAR parser returns its value, including leading zeros and every byte alignment"
                    ),
                );
                kani::assert(
                    input.ptr == unsafe { bytes.as_ptr().add(start + len + 1) },
                    concat!("Parsing a ", stringify!($ty), " consumes exactly its optional minus sign, digits and one delimiter"),
                );
                kani::cover!(len == $digits && expected == $ty::MAX as i64 && offset == 7);
                kani::cover!(expected == $ty::MIN as i64);
                kani::cover!(len == 1 && expected == 0);
            }
        };
    }

    proof_integer_input!(proof_u8_input, u8, 3, false);
    proof_integer_input!(proof_u16_input, u16, 5, false);
    proof_integer_input!(proof_i8_input, i8, 3, kani::any());
    proof_integer_input!(proof_i16_input, i16, 5, kani::any());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    use crate::tools::testutil::integer_boundary_values;

    #[test]
    fn test_integer_io() {
        let mut rng = Xorshift::default();
        macro_rules! check {
            ($($ty:ident),*) => {$(
                for offset in 0..64 {
                    let mut values = integer_boundary_values!($ty);
                    if $ty::BITS <= 16 && offset == 0 {
                        values.extend($ty::MIN..=$ty::MAX);
                    }
                    let n = rng.random(0..=1000);
                    values.extend((0..n).map(|_| {
                        let x: $ty = rng.random(..);
                        x >> rng.random(0..$ty::BITS)
                    }));
                    let mut input = vec![b' '; offset];
                    for x in &values {
                        write!(input, "{}{}", x, char::from([b' ', b'\n', b'\t'][rng.random(0usize..3)])).unwrap();
                    }
                    input.extend_from_slice(&[b' '; 16]);
                    let input = input.into_boxed_slice();
                    let mut reader = unsafe { FastInput::from_slice(&input[offset..]) };
                    for &x in &values { assert_eq!(unsafe { reader.$ty() }, x); }
                    let mut output = Vec::new();
                    {
                        let mut writer = FastOutput::with_capacity(offset, &mut output);
                        for &x in &values { writer.$ty(x); writer.byte(b'\n'); }
                    }
                    let expected: String = values.iter().map(|x| format!("{x}\n")).collect();
                    assert_eq!(output, expected.as_bytes());
                }
            )*};
        }
        check!(
            u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
        );
        for value in integer_boundary_values!(u32)
            .into_iter()
            .filter(|&v| v < 100_000_000)
            .chain(rng.random_iter(0..100_000_000u32).take(1000))
        {
            let input = format!("{value}                 ");
            let mut reader = unsafe { FastInput::from_slice(input.as_bytes()) };
            assert_eq!(unsafe { reader.u32_small() }, value);
        }
    }

    #[test]
    fn test_extend_u32() {
        let mut rng = Xorshift::default();
        let boundaries = integer_boundary_values!(u32);
        for n in [0, 1, 7, 8, 9, 543, 544, 545, 65567, 65568, 65569] {
            for mode in 0..4 {
                let values: Vec<u32> = (0..n)
                    .map(|i| {
                        if i < boundaries.len() {
                            boundaries[i]
                        } else {
                            match mode {
                                1 => rng.random(0..10),
                                3 if i < n / 2 => rng.random(0..10),
                                _ => rng.random(..),
                            }
                        }
                    })
                    .collect();
                let mut bytes = Vec::new();
                for &value in &values {
                    if mode == 2 {
                        write!(bytes, "{value:010}").unwrap();
                    } else {
                        write!(bytes, "{value}").unwrap();
                    }
                    bytes.push([b' ', b'\n', b'\t', b'\r', 12][rng.random(0usize..5)]);
                }
                bytes.extend_from_slice(b"-12345 END                 ");
                let mut input = unsafe { FastInput::from_slice(&bytes) };
                let mut actual = vec![u32::MAX; 17];
                unsafe { input.extend_u32(&mut actual, n) };
                assert_eq!(&actual[..17], &[u32::MAX; 17]);
                assert_eq!(&actual[17..], values);
                assert_eq!(unsafe { input.i32() }, -12345);
                assert_eq!(unsafe { input.bytes() }, b"END");
            }
        }
    }

    #[test]
    fn test_u32_slice_padded() {
        let mut rng = Xorshift::default();
        let mut values = integer_boundary_values!(u32);
        values.extend(0..10000);
        values.extend(rng.random_iter(0..=u32::MAX).take(65536));
        for len in [0, 1, 7, 8, 9, 15, 16, 17, 31, values.len()] {
            let values = &values[..len];
            let expected: String = values.iter().map(|x| format!("{x:10} ")).collect();
            for capacity in [32, 88, 89, 91, 92, 93, 4096, 1 << 18] {
                for prefix in [0, 1, 31] {
                    let mut actual = Vec::new();
                    {
                        let mut output = FastOutput::with_capacity(capacity, &mut actual);
                        output.bytes(&[b'!'; 31][..prefix]);
                        output.u32_slice_padded(values);
                        output.byte(b'\n');
                    }
                    assert_eq!(&actual[..prefix], &[b'!'; 31][..prefix]);
                    assert_eq!(&actual[prefix..actual.len() - 1], expected.as_bytes());
                    assert_eq!(actual.last(), Some(&b'\n'));
                }
            }
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
}

use super::{Xorshift, miller_rabin, primitive_root};

const K: usize = 1 << 21;
const POW_BLOCK_BITS: u32 = 15;
const POW_BLOCK: usize = 1 << POW_BLOCK_BITS;
const FRAC_SHIFT: u32 = 10;
const FRAC_LEN: usize = 1 + (1 << (30 - FRAC_SHIFT));
const FRAC_DEN_LIMIT: u16 = 1 << 11;
const BSGS_SIZE: usize = 1 << 17;

const fn is_direct_table_mod<const P: u32>() -> bool {
    P as usize <= K
}

const fn table_k<const P: u32>() -> usize {
    if is_direct_table_mod::<P>() {
        P as usize - 1
    } else {
        K
    }
}

const fn table_len<const P: u32>() -> usize {
    2 * table_k::<P>() + 1
}

/// Fast online inverse and power queries for a prime modulus.
pub struct FastPrimeMod<const P: u32, const BUILD_INV: bool = true, const BUILD_POW: bool = true> {
    root: u32,
    pow_lo: Box<[u32]>,
    pow_hi: Box<[u32]>,
    frac: Box<[u32]>,
    log: Box<[u32]>,
    inv: Box<[u32]>,
}

impl<const P: u32, const BUILD_INV: bool, const BUILD_POW: bool>
    FastPrimeMod<P, BUILD_INV, BUILD_POW>
{
    /// Builds the tables enabled by the const generic mode.
    ///
    /// # Panics
    ///
    /// Panics if `P` is not an odd prime or if `P >= 2^30`.
    pub fn new() -> Self {
        assert!(
            BUILD_INV || BUILD_POW,
            "at least one of BUILD_INV or BUILD_POW must be true"
        );
        assert!(P < 1 << 30, "P must be less than 2^30");
        assert!(
            P % 2 == 1 && miller_rabin(P as u64),
            "P must be an odd prime"
        );

        let (root, pow_lo, pow_hi, log) = if BUILD_POW {
            let root = if P == 998_244_353 {
                3
            } else {
                primitive_root(P as u64) as u32
            };
            let (pow_lo, pow_hi) = build_pow::<P>(root);
            let log = build_log::<P>(root, &pow_lo, &pow_hi);
            (root, pow_lo, pow_hi, log)
        } else {
            (
                0,
                Vec::new().into_boxed_slice(),
                Vec::new().into_boxed_slice(),
                Vec::new().into_boxed_slice(),
            )
        };
        let inv = if BUILD_INV {
            build_inv::<P>()
        } else {
            Vec::new().into_boxed_slice()
        };
        let frac = if is_direct_table_mod::<P>() {
            Vec::new().into_boxed_slice()
        } else {
            build_frac::<P>()
        };
        Self {
            root,
            pow_lo,
            pow_hi,
            frac,
            log,
            inv,
        }
    }

    /// Returns the prime modulus.
    #[inline]
    pub fn modulus(&self) -> u32 {
        P
    }

    #[inline(always)]
    fn small_fraction(&self, x: u32) -> (usize, u32) {
        let k = table_k::<P>();
        if is_direct_table_mod::<P>() {
            debug_assert!(1 <= x && x < P);
            return (k + x as usize, 1);
        }
        let packed = self.frac[(x >> FRAC_SHIFT) as usize];
        let a = packed >> 16;
        let b = packed & 0xffff;
        let t = x.wrapping_mul(b).wrapping_sub(a.wrapping_mul(P));
        debug_assert!({
            let t = x as i64 * b as i64 - a as i64 * P as i64;
            t != 0 && -(k as i64) <= t && t <= k as i64
        });
        ((k as u32).wrapping_add(t) as usize, b)
    }
}

impl<const P: u32, const BUILD_POW: bool> FastPrimeMod<P, true, BUILD_POW> {
    /// Returns `x^{-1} mod P`.
    ///
    /// # Panics
    ///
    /// Panics unless `1 <= x < P`.
    #[inline]
    pub fn inverse(&self, x: u32) -> u32 {
        assert!(1 <= x && x < P);
        let (i, b) = self.small_fraction(x);
        mul_mod_raw::<P>(self.inv[i], b)
    }
}

impl<const P: u32, const BUILD_INV: bool> FastPrimeMod<P, BUILD_INV, true> {
    /// Returns the primitive root used by this table.
    #[inline]
    pub fn primitive_root(&self) -> u32 {
        self.root
    }

    /// Returns `a^exp mod P`.
    ///
    /// `0^0` is defined as `1`.
    ///
    /// # Panics
    ///
    /// Panics unless `a < P`.
    #[inline]
    pub fn pow(&self, a: u32, exp: u64) -> u32 {
        assert!(a < P);
        if a == 0 {
            return if exp == 0 { 1 } else { 0 };
        }
        let ord = (P - 1) as u64;
        self.pow_nonzero_reduced(a, (exp % ord) as u32)
    }

    /// Returns `a^exp_mod mod P` for a non-zero base and a reduced exponent.
    ///
    /// # Panics
    ///
    /// Panics unless `1 <= a < P` and `exp_mod < P - 1`.
    #[inline]
    pub fn pow_nonzero_reduced(&self, a: u32, exp_mod: u32) -> u32 {
        assert!(1 <= a && a < P);
        assert!(exp_mod < P - 1);
        let exp = (self.log_r(a) as u64 * exp_mod as u64 % (P - 1) as u64) as u32;
        self.pow_root_reduced(exp)
    }

    /// Returns `r^exp_mod mod P`, where `r` is this table's primitive root.
    ///
    /// # Panics
    ///
    /// Panics unless `exp_mod < P - 1`.
    #[inline]
    pub fn pow_root_reduced(&self, exp_mod: u32) -> u32 {
        assert!(exp_mod < P - 1);
        pow_root_raw::<P>(exp_mod, &self.pow_lo, &self.pow_hi)
    }

    #[inline]
    fn log_r(&self, x: u32) -> u32 {
        let (i, b) = self.small_fraction(x);
        let k = table_k::<P>();
        let ord = P - 1;
        self.log[i] + ord - self.log[k + b as usize]
    }
}

impl<const P: u32, const BUILD_INV: bool, const BUILD_POW: bool> Default
    for FastPrimeMod<P, BUILD_INV, BUILD_POW>
{
    fn default() -> Self {
        Self::new()
    }
}

fn build_pow<const P: u32>(root: u32) -> (Box<[u32]>, Box<[u32]>) {
    let mut pow_lo = vec![0; POW_BLOCK + 1].into_boxed_slice();
    let mut pow_hi = vec![0; POW_BLOCK + 1].into_boxed_slice();
    pow_lo[0] = 1;
    pow_hi[0] = 1;
    for i in 0..POW_BLOCK {
        pow_lo[i + 1] = mul_mod_raw::<P>(pow_lo[i], root);
    }
    let block_power = pow_lo[POW_BLOCK];
    for i in 0..POW_BLOCK {
        pow_hi[i + 1] = mul_mod_raw::<P>(pow_hi[i], block_power);
    }
    (pow_lo, pow_hi)
}

fn build_inv<const P: u32>() -> Box<[u32]> {
    let k = table_k::<P>();
    let mut inv = vec![0; table_len::<P>()].into_boxed_slice();
    inv[k + 1] = 1;
    for i in 2..=k {
        let q = P.div_ceil(i as u32);
        let r = i as u32 * q - P;
        inv[k + i] = mul_mod_raw::<P>(inv[k + r as usize], q);
    }
    for i in 1..=k {
        inv[k - i] = P - inv[k + i];
    }
    inv
}

fn build_log<const P: u32>(root: u32, pow_lo: &[u32], pow_hi: &[u32]) -> Box<[u32]> {
    let k = table_k::<P>();
    let ord = P - 1;
    let mut lpf = vec![0; k + 1].into_boxed_slice();
    let mut primes = vec![];
    lpf[1] = 1;
    for i in 2..=k {
        if lpf[i] == 0 {
            lpf[i] = i as u32;
            primes.push(i as u32);
        }
        for &p in primes.iter() {
            let p = p as usize;
            if p > lpf[i] as usize || p > k / i {
                break;
            }
            lpf[i * p] = p as u32;
        }
    }

    let baby_size = (BSGS_SIZE as u32).min(ord);
    let mut baby = U32Map::new(baby_size as usize);
    let mut pw = 1;
    for i in 0..baby_size {
        baby.insert(pw, i);
        pw = mul_mod_raw::<P>(pw, root);
    }
    let q = pow_root_raw::<P>(ord - baby_size, pow_lo, pow_hi);

    let mut log = vec![0; table_len::<P>()].into_boxed_slice();
    log[k + 1] = 0;
    let mut rng = Xorshift::default();
    let small_primes = [2, 3, 5, 7, 11, 13, 17, 19];
    for i in 2..=k {
        let p = lpf[i] as usize;
        if p < i {
            log[k + i] = add_mod(log[k + p], log[k + i / p], ord);
        } else if i < 100 {
            let mut x = i as u32;
            let mut ans = 0;
            loop {
                if let Some(v) = baby.get(x) {
                    log[k + i] = ans + v;
                    break;
                }
                ans += baby_size;
                x = mul_mod_raw::<P>(x, q);
            }
        } else if i > P as usize / i {
            let j = (P as usize) / i;
            let r = (P as usize) % i;
            let x = add_mod(log[k + r], ord / 2, ord);
            let y = log[k + j];
            log[k + i] = if x >= y { x - y } else { x + ord - y };
        } else {
            loop {
                let exp = rng.rand(ord as u64) as u32;
                let mut ans = if exp == 0 { 0 } else { ord - exp };
                let mut x = mul_mod_raw::<P>(i as u32, pow_root_raw::<P>(exp, pow_lo, pow_hi));
                for q in small_primes {
                    while x.is_multiple_of(q) {
                        x /= q;
                        ans = add_mod(ans, log[k + q as usize], ord);
                    }
                }
                if x as usize >= k {
                    continue;
                }
                while (i as u32) < x && lpf[x as usize] < i as u32 {
                    let q = lpf[x as usize];
                    x /= q;
                    ans = add_mod(ans, log[k + q as usize], ord);
                }
                if 1 < x && x < i as u32 {
                    ans = add_mod(ans, log[k + x as usize], ord);
                    x = 1;
                }
                if x == 1 {
                    log[k + i] = ans;
                    break;
                }
            }
        }
    }
    for i in 1..=k {
        log[k - i] = add_mod(log[k + i], ord / 2, ord);
    }
    log
}

fn build_frac<const P: u32>() -> Box<[u32]> {
    let mut frac = vec![0; FRAC_LEN].into_boxed_slice();
    let mut stack = vec![(0u16, 1u16, 1u16, 1u16)];
    while let Some((a, b, c, d)) = stack.pop() {
        let nb = b + d;
        if nb < FRAC_DEN_LIMIT {
            stack.push((a + c, nb, c, d));
            stack.push((a, b, a + c, nb));
        } else {
            let s = (a as u64 * P as u64 / ((1u64 << FRAC_SHIFT) * b as u64)) as usize;
            let t = (c as u64 * P as u64 / ((1u64 << FRAC_SHIFT) * d as u64)) as usize;
            frac[s] = pack_frac(a, b);
            frac[t] = pack_frac(c, d);
            let a = a.min(c);
            let b = b.min(d);
            if s + 1 < t {
                for x in &mut frac[s + 1..t] {
                    *x = pack_frac(a, b);
                }
            }
        }
    }
    frac
}

#[inline(always)]
fn pow_root_raw<const P: u32>(exp: u32, pow_lo: &[u32], pow_hi: &[u32]) -> u32 {
    let lo = exp as usize & (POW_BLOCK - 1);
    let hi = exp as usize >> POW_BLOCK_BITS;
    mul_mod_raw::<P>(pow_lo[lo], pow_hi[hi])
}

#[inline(always)]
fn mul_mod_raw<const P: u32>(a: u32, b: u32) -> u32 {
    (a as u64 * b as u64 % P as u64) as u32
}

#[inline]
fn add_mod(a: u32, b: u32, m: u32) -> u32 {
    let c = a + b;
    if c >= m { c - m } else { c }
}

#[inline]
fn pack_frac(a: u16, b: u16) -> u32 {
    (a as u32) << 16 | b as u32
}

struct U32Map {
    keys: Box<[u32]>,
    values: Box<[u32]>,
    mask: usize,
}

impl U32Map {
    fn new(capacity: usize) -> Self {
        let len = (capacity * 2).next_power_of_two();
        Self {
            keys: vec![0; len].into_boxed_slice(),
            values: vec![0; len].into_boxed_slice(),
            mask: len - 1,
        }
    }

    fn insert(&mut self, key: u32, value: u32) {
        debug_assert_ne!(key, 0);
        let mut i = self.index(key);
        while self.keys[i] != 0 && self.keys[i] != key {
            i = (i + 1) & self.mask;
        }
        self.keys[i] = key;
        self.values[i] = value;
    }

    fn get(&self, key: u32) -> Option<u32> {
        debug_assert_ne!(key, 0);
        let mut i = self.index(key);
        while self.keys[i] != 0 {
            if self.keys[i] == key {
                return Some(self.values[i]);
            }
            i = (i + 1) & self.mask;
        }
        None
    }

    #[inline]
    fn index(&self, key: u32) -> usize {
        let mut x = key as u64;
        x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        ((x ^ (x >> 31)) as usize) & self.mask
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{math::modinv, tools::RandomSpec};

    fn mod_pow<const P: u32>(a: u32, mut exp: u64) -> u32 {
        let mut x = a;
        let mut y = 1;
        while exp > 0 {
            if exp & 1 == 1 {
                y = mul_mod_raw::<P>(y, x);
            }
            x = mul_mod_raw::<P>(x, x);
            exp >>= 1;
        }
        y
    }

    fn check_prime_mod<const P: u32>() {
        let all = FastPrimeMod::<P>::new();
        let inv_only = FastPrimeMod::<P, true, false>::new();
        let pow_only = FastPrimeMod::<P, false, true>::new();
        assert_eq!(all.modulus(), P);
        assert_eq!(inv_only.modulus(), P);
        assert_eq!(pow_only.modulus(), P);
        assert!(1 <= all.primitive_root() && all.primitive_root() < P);
        assert_eq!(all.primitive_root(), pow_only.primitive_root());

        for x in [1, 2, 3, P / 2, P - 2, P - 1, 1_234_567] {
            if x < P {
                let expected = modinv(x as u64, P as u64) as u32;
                assert_eq!(all.inverse(x), expected);
                assert_eq!(inv_only.inverse(x), expected);
                assert_eq!(mul_mod_raw::<P>(x, all.inverse(x)), 1);
            }
        }

        let fixed = [
            (0, 0),
            (0, 1),
            (1, u64::MAX),
            (2, 0),
            (2, 1),
            (2, 123_456_789_012_345),
            (P - 1, 0),
            (P - 1, 1),
            (P - 1, 2),
            (P - 1, 123_456_789),
        ];
        for (a, exp) in fixed {
            let expected = mod_pow::<P>(a, exp);
            assert_eq!(all.pow(a, exp), expected);
            assert_eq!(pow_only.pow(a, exp), expected);
            if a != 0 {
                let exp_mod = (exp % (P - 1) as u64) as u32;
                assert_eq!(all.pow_nonzero_reduced(a, exp_mod), expected);
                assert_eq!(pow_only.pow_nonzero_reduced(a, exp_mod), expected);
            }
        }

        let mut rng = Xorshift::default();
        for x in (1..P).rand_iter(&mut rng).take(2_000) {
            let expected = modinv(x as u64, P as u64) as u32;
            assert_eq!(all.inverse(x), expected);
            assert_eq!(inv_only.inverse(x), expected);
        }
        for (a, exp) in (0..P, 0u64..).rand_iter(&mut rng).take(2_000) {
            let expected = mod_pow::<P>(a, exp);
            assert_eq!(all.pow(a, exp), expected);
            assert_eq!(pow_only.pow(a, exp), expected);
            if a != 0 {
                let exp_mod = (exp % (P - 1) as u64) as u32;
                assert_eq!(all.pow_nonzero_reduced(a, exp_mod), expected);
                assert_eq!(pow_only.pow_nonzero_reduced(a, exp_mod), expected);
            }
        }
        for exp_mod in (0..P - 1).rand_iter(&mut rng).take(2_000) {
            assert_eq!(
                pow_only.pow_root_reduced(exp_mod),
                mod_pow::<P>(pow_only.primitive_root(), exp_mod as u64)
            );
        }
    }

    #[test]
    fn test_fast_prime_mod() {
        check_prime_mod::<998_244_353>();
        check_prime_mod::<1_000_000_007>();
        check_prime_mod::<3>();
        check_prime_mod::<101>();
        check_prime_mod::<2_017>();
        check_prime_mod::<1_000_003>();
        check_prime_mod::<2_097_143>();
        check_prime_mod::<2_097_169>();
    }
}

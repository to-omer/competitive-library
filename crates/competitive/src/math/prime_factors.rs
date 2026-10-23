use super::{BarrettReduction, Xorshift, gcd, miller_rabin_with_br};
#[cfg(verus_keep_ghost)]
use verification::*;
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

#[cfg_attr(verus_keep_ghost, verus_verify)]
struct MontgomeryReduction64 {
    modulus: u64,
    inverse: u64,
}

#[cfg_attr(verus_keep_ghost, verus_verify)]
impl MontgomeryReduction64 {
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires modulus % 2 == 1,
        ensures result.modulus == modulus,
            (modulus as int * result.inverse as int) % radix() == 1,
    ))]
    fn new(modulus: u64) -> Self {
        let mut inverse = modulus;
        #[cfg(verus_keep_ghost)]
        proof! {
            reveal(vstd::arithmetic::power::pow);
            vstd::arithmetic::div_mod::lemma_mul_mod_noop(modulus as int, modulus as int, 2);
            vstd::arithmetic::div_mod::lemma_sub_mod_noop(modulus as int * modulus as int, 1, 2);
            assert(pow2(pow2(0)) == 2) by(compute);
            assert((modulus as int * inverse as int - 1) % 2 == 0);
        }
        #[cfg_attr(verus_keep_ghost, verus_spec(iter => invariant
            modulus % 2 == 1,
            (modulus as int * inverse as int - 1) % (pow2(pow2(iter.index() as nat)) as int) == 0,
        ))]
        for _ in 0..6 {
            #[cfg(verus_keep_ghost)]
            proof_decl! { let ghost previous = inverse; }
            inverse = inverse.wrapping_mul(2u64.wrapping_sub(modulus.wrapping_mul(inverse)));
            #[cfg(verus_keep_ghost)]
            proof! {
                inverse_precision(iter.index() as nat);
                inverse_refinement(modulus, previous, pow2(pow2(iter.index() as nat)) as int);
            }
        }
        #[cfg(verus_keep_ghost)]
        proof! {
            reveal(vstd::arithmetic::power::pow);
            assert(pow2(pow2(6)) == radix()) by(compute);
            vstd::arithmetic::div_mod::lemma_mod_adds(modulus as int * inverse as int - 1, 1, radix());
            #[verifier::proof_note("For every odd u64 modulus, six Newton iterations with wrapping arithmetic compute its exact multiplicative inverse modulo 2^64")]
            assert((modulus as int * inverse as int) % radix() == 1);
        }
        Self { modulus, inverse }
    }
}

impl MontgomeryReduction64 {
    fn sub(&self, lhs: u64, rhs: u64) -> u64 {
        let (value, borrow) = lhs.overflowing_sub(rhs);
        value.wrapping_add((borrow as u64).wrapping_neg() & self.modulus)
    }

    fn mul(&self, lhs: u64, rhs: u64) -> u64 {
        let product = lhs as u128 * rhs as u128;
        let (value, borrow) = ((product >> 64) as u64).overflowing_sub(
            (((product as u64).wrapping_mul(self.inverse) as u128 * self.modulus as u128) >> 64)
                as u64,
        );
        value.wrapping_add((borrow as u64).wrapping_neg() & self.modulus)
    }
}

fn find_factor(n: u64) -> Option<u64> {
    let br = BarrettReduction::<u128>::new(n as u128);
    if miller_rabin_with_br(n, &br) {
        return None;
    }
    let mr = MontgomeryReduction64::new(n);
    let mut rng = Xorshift::default();
    let (mut y0, mut c) = (0, n - 1);
    loop {
        let (mut x, mut y, mut ys, mut g, mut q, mut r, mut k) = (0, y0, 0, 1, 1, 1, 0);
        while g == 1 && r <= 1 << 20 {
            x = y;
            while k < r && g == 1 {
                ys = y;
                for _ in 0..1024.min(r - k) {
                    y = mr.sub(mr.mul(y, y), c);
                    q = mr.mul(q, mr.sub(x, y));
                }
                g = gcd(q, n);
                k += 1024;
            }
            k = r;
            r <<= 1;
        }
        if g == n {
            g = 1;
            y = ys;
            while g == 1 {
                y = mr.sub(mr.mul(y, y), c);
                g = gcd(mr.sub(x, y), n);
            }
        }
        if g != 1 && g != n {
            return Some(g);
        }
        y0 = ((rng.rand64() as u128 * (n - 2) as u128) >> 64) as u64 + 2;
        c = ((rng.rand64() as u128 * (n - 1) as u128) >> 64) as u64 + 1;
    }
}

pub fn prime_factors_flatten(mut n: u64) -> Vec<u64> {
    if n == 0 {
        return vec![];
    }
    let k = n.trailing_zeros();
    let mut res = vec![2; k as usize];
    n >>= k;
    while n.is_multiple_of(3) {
        res.push(3);
        n /= 3;
    }
    if n != 1 {
        let mut c = vec![n];
        while let Some(n) = c.pop() {
            if let Some(m) = find_factor(n) {
                c.push(m);
                c.push(n / m);
            } else {
                res.push(n);
            }
        }
    }
    res.sort_unstable();
    res
}

pub fn prime_factors(n: u64) -> Vec<(u64, u32)> {
    let mut res = Vec::new();
    for a in prime_factors_flatten(n) {
        if let Some((p, len)) = res.last_mut()
            && p == &a
        {
            *len += 1;
            continue;
        }
        res.push((a, 1));
    }
    res
}

pub fn divisors(n: u64) -> Vec<u64> {
    let mut d = vec![1u64];
    for (p, c) in prime_factors(n) {
        let k = d.len();
        let mut acc = 1;
        for _ in 0..c {
            acc *= p;
            for i in 0..k {
                d.push(d[i] * acc);
            }
        }
    }
    d.sort_unstable();
    d
}

#[cfg(any(kani, verus_keep_ghost))]
mod verification {
    #[cfg(verus_keep_ghost)]
    pub use vstd::arithmetic::power2::*;
    #[cfg(verus_keep_ghost)]
    use vstd::{arithmetic::div_mod::*, prelude::*, wrapping::u64_specs};

    #[cfg(kani)]
    #[kani::proof]
    fn proof_modular_subtraction() {
        let modulus: u64 = kani::any();
        let lhs: u64 = kani::any();
        let rhs: u64 = kani::any();
        kani::assume(modulus > 0 && lhs < modulus && rhs < modulus);
        let reduction = super::MontgomeryReduction64 {
            modulus,
            inverse: 0,
        };
        let expected = if lhs >= rhs {
            lhs - rhs
        } else {
            (modulus as u128 + lhs as u128 - rhs as u128) as u64
        };
        kani::assert(
            reduction.sub(lhs, rhs) == expected,
            "For every positive u64 modulus and pair of reduced residues, Montgomery subtraction returns their canonical modular difference, including borrowing at u64::MAX",
        );
        kani::cover!(modulus == u64::MAX && lhs == 0 && rhs == u64::MAX - 1);
        kani::cover!(modulus == 1);
        kani::cover!(lhs == rhs);
    }

    #[cfg(verus_keep_ghost)]
    verus! {
        pub open spec fn radix() -> int { 0x1_0000_0000_0000_0000 }

        pub open spec fn inverse_step(m: u64, x: u64) -> u64 {
            u64_specs::wrapping_mul(x, u64_specs::wrapping_sub(2, u64_specs::wrapping_mul(m, x)))
        }

        pub proof fn inverse_refinement(m: u64, x: u64, p: int)
            requires p > 1, radix() % (p * p) == 0, (m as int * x as int - 1) % p == 0,
            ensures (m as int * inverse_step(m, x) as int - 1) % (p * p) == 0,
        {
            let big = radix();
            let a = u64_specs::wrapping_mul(m, x);
            let b = u64_specs::wrapping_sub(2, a);
            let next = inverse_step(m, x);
            let qa = m as int * x as int / big;
            let qx = x as int * b as int / big;
            let c = if a > 2 { 1int } else { 0int };
            let t = (m as int * x as int - 1) / p;
            let co = big / (p * p);
            lemma_mod_bound(m as int * x as int, big);
            lemma_mod_bound(x as int * b as int, big);
            lemma_fundamental_div_mod(m as int * x as int, big);
            lemma_fundamental_div_mod(x as int * b as int, big);
            lemma_fundamental_div_mod(m as int * x as int - 1, p);
            assert(p * p > 0) by(nonlinear_arith) requires p > 1;
            lemma_fundamental_div_mod(big, p * p);
            assert(m as int * x as int == qa * big + a);
            assert(b as int == 2 - a + big * c);
            assert(x as int * b as int == qx * big + next);
            let correction = x as int * (qa + c) - qx;
            assert(next as int == x as int * (2 - m as int * x as int) + big * correction) by(nonlinear_arith)
                requires
                    m as int * x as int == qa * big + a,
                    b as int == 2 - a + big * c,
                    x as int * b as int == qx * big + next,
                    correction == x as int * (qa + c) - qx;
            let error = m as int * x as int - 1;
            assert(m as int * next as int - 1 == -error * error + big * m * correction) by(nonlinear_arith)
                requires next as int == x as int * (2 - m as int * x as int) + big * correction,
                    error == m as int * x as int - 1;
            let witness = co * m * correction - t * t;
            assert(m as int * next as int - 1 == (p * p) * witness) by(nonlinear_arith)
                requires
                    m as int * next as int - 1 == -error * error + big * m * correction,
                    error == p * t,
                    big == co * (p * p),
                    witness == co * m * correction - t * t;
            lemma_mod_multiples_basic(witness, p * p);
            #[verifier::proof_note("A wrapping Newton step squares the inverse-error divisor whenever its square divides 2^64")]
            assert((m as int * next as int - 1) % (p * p) == 0);
        }

        pub proof fn inverse_precision(i: nat)
            requires i < 6,
            ensures
                pow2(pow2(i + 1)) == pow2(pow2(i)) * pow2(pow2(i)),
                pow2(pow2(i)) > 1,
                radix() % ((pow2(pow2(i)) * pow2(pow2(i))) as int) == 0,
        {
            reveal(vstd::arithmetic::power::pow);
            lemma_pow2_unfold(i + 1);
            lemma_pow2_adds(pow2(i), pow2(i));
            assert(pow2(5) == 32 && pow2(64) == radix()) by(compute);
            if i < 5 { lemma_pow2_strictly_increases(i, 5); }
            lemma_pow2_pos(i);
            lemma_pow2_strictly_increases(0, pow2(i));
            assert(pow2(0) == 1) by(compute);
            let e = 2 * pow2(i);
            lemma_pow2_adds(e, (64 - e) as nat);
            lemma_pow2_pos(e);
            assert(pow2(e) == pow2(pow2(i)) * pow2(pow2(i)));
            assert(radix() == pow2(e) * pow2((64 - e) as nat));
            vstd::arithmetic::mul::lemma_mul_is_commutative(pow2(e) as int, pow2((64 - e) as nat) as int);
            lemma_mod_multiples_basic(pow2((64 - e) as nat) as int, pow2(e) as int);
            #[verifier::proof_note("The six Newton steps double inverse precision from 1 bit to 64 bits without requiring a modulus larger than 2^64")]
            assert(pow2(pow2(i + 1)) == pow2(pow2(i)) * pow2(pow2(i))
                && radix() % (pow2(pow2(i + 1)) as int) == 0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    pub fn naive_divisors(n: u64) -> Vec<u64> {
        let mut res = vec![];
        for i in 1..(n as f32).sqrt() as u64 + 1 {
            if n.is_multiple_of(i) {
                res.push(i);
                if i * i != n {
                    res.push(n / i);
                }
            }
        }
        res.sort_unstable();
        res
    }

    #[test]
    fn test_prime_factors_rho() {
        use crate::{math::miller_rabin, tools::Xorshift};
        const Q: usize = 2_000;
        let mut rng = Xorshift::default();
        for _ in 0..Q {
            let x = rng.rand64();
            let factors = prime_factors_flatten(x);
            assert!(factors.iter().all(|&p| miller_rabin(p)));
            let p = factors.into_iter().product::<u64>();
            assert_eq!(x, p);
        }
    }

    #[test]
    fn test_divisors() {
        let mut rng = Xorshift::default();
        for n in (1..1000).chain(rng.random_iter(1..=20000000).take(100)) {
            assert_eq!(divisors(n), naive_divisors(n));
        }
    }
}

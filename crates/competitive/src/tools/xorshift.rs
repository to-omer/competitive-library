use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hasher},
    iter::repeat_with,
};
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

#[cfg_attr(verus_keep_ghost, verus_verify)]
#[cfg_attr(verus_keep_ghost, verifier::allow(autoderive_clone_without_spec))]
#[derive(Clone, Debug)]
pub struct Xorshift {
    y: u64,
}

#[cfg_attr(verus_keep_ghost, verus_verify)]
impl Xorshift {
    #[cfg_attr(verus_keep_ghost, verus_spec(result => ensures result@ == seed))]
    pub fn new_with_seed(seed: u64) -> Self {
        Xorshift { y: seed }
    }
    #[cfg_attr(verus_keep_ghost, verus_verify(external))]
    pub fn new() -> Self {
        Xorshift::new_with_seed(RandomState::new().build_hasher().finish())
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        ensures result == step(old(self)@), final(self)@ == result,
            (result == 0) == (old(self)@ == 0),
    ))]
    pub fn rand64(&mut self) -> u64 {
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost seed = self.y; }
        self.y ^= self.y << 5;
        self.y ^= self.y >> 17;
        self.y ^= self.y << 11;
        #[cfg(verus_keep_ghost)]
        proof! {
            #[verifier::proof_note("For every u64 state, rand64 performs the specified XOR-shift transition and preserves whether the state is zero; rand with a positive bound returns this output modulo the bound and leaves the same next state; rands, floating-point generation, shuffle, distribution, period, independence, and cryptographic properties are outside this proof")]
            assert((step(seed) == 0) == (seed == 0)) by(bit_vector);
        }
        self.y
    }

    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires k > 0,
        ensures result < k, result == step(old(self)@) % k,
            final(self)@ == step(old(self)@),
            (final(self)@ == 0) == (old(self)@ == 0),
    ))]
    pub fn rand(&mut self, k: u64) -> u64 {
        self.rand64() % k
    }
    #[cfg_attr(verus_keep_ghost, verus_verify(external))]
    pub fn rands(&mut self, k: u64, n: usize) -> Vec<u64> {
        repeat_with(|| self.rand(k)).take(n).collect()
    }

    #[cfg_attr(verus_keep_ghost, verus_verify(external))]
    pub fn randf(&mut self) -> f64 {
        const UPPER_MASK: u64 = 0x3FF0_0000_0000_0000;
        const LOWER_MASK: u64 = 0x000F_FFFF_FFFF_FFFF;
        let x = self.rand64();
        let tmp = UPPER_MASK | (x & LOWER_MASK);
        let result: f64 = f64::from_bits(tmp);
        f64::from_bits(f64::to_bits(result - 1.0) ^ (x >> 63))
    }

    #[cfg_attr(verus_keep_ghost, verus_verify(external))]
    pub fn gen_bool(&mut self, p: f64) -> bool {
        self.randf() < p
    }

    #[cfg_attr(verus_keep_ghost, verus_verify(external))]
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        let mut n = slice.len();
        while n > 1 {
            let i = self.rand(n as _) as usize;
            n -= 1;
            slice.swap(i, n);
        }
    }
}

#[cfg_attr(verus_keep_ghost, verus_verify)]
impl Default for Xorshift {
    #[cfg_attr(verus_keep_ghost, verus_spec(result => ensures result@ == 0x2b99_2ddf_a232_49d6u64))]
    fn default() -> Self {
        Xorshift::new_with_seed(0x2b99_2ddf_a232_49d6)
    }
}

#[cfg(verus_keep_ghost)]
verus! {
    impl View for Xorshift {
        type V = u64;
        closed spec fn view(&self) -> u64 { self.y }
    }

    pub open spec fn step(seed: u64) -> u64 {
        let a = seed ^ (seed << 5);
        let b = a ^ (a >> 17);
        b ^ (b << 11)
    }
}

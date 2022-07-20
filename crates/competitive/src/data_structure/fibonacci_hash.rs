use std::{
    collections::{HashMap, HashSet},
    convert::TryInto,
    hash::{BuildHasherDefault, Hasher},
};

#[cfg(target_pointer_width = "32")]
pub type FibonacciHasher = FibonacciHasheru32;
#[cfg(not(target_pointer_width = "32"))]
pub type FibonacciHasher = FibonacciHasheru64;
pub type FibHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FibonacciHasher>>;
pub type FibHashSet<V> = HashSet<V, BuildHasherDefault<FibonacciHasher>>;

#[derive(Debug, Default)]
pub struct FibonacciHasheru32 {
    hash: u32,
}
impl FibonacciHasheru32 {
    const A: u32 = 2654435769;
    const B: u32 = 5;
    fn push(&mut self, x: u32) {
        self.hash = (self.hash.rotate_right(Self::B) ^ x).wrapping_mul(Self::A);
    }
}
impl Hasher for FibonacciHasheru32 {
    fn finish(&self) -> u64 {
        self.hash as u64
    }
    fn write(&mut self, mut bytes: &[u8]) {
        if bytes.len() % 4 >= 2 {
            self.push(u16::from_ne_bytes(bytes[..2].try_into().unwrap()) as u32);
            bytes = &bytes[2..];
        }
        if bytes.len() % 2 >= 1 {
            self.push(u16::from_ne_bytes(bytes[..1].try_into().unwrap()) as u32);
            bytes = &bytes[1..];
        }
        for chunk in bytes.chunks(4) {
            self.push(u32::from_ne_bytes(chunk.try_into().unwrap()));
        }
    }
    fn write_u8(&mut self, i: u8) {
        self.push(i as u32)
    }
    fn write_u16(&mut self, i: u16) {
        self.push(i as u32)
    }
    fn write_u32(&mut self, i: u32) {
        self.push(i)
    }
    fn write_u64(&mut self, i: u64) {
        self.push(i as u32);
        self.push((i >> 32) as u32);
    }
    fn write_u128(&mut self, i: u128) {
        self.push(i as u32);
        self.push((i >> 32) as u32);
        self.push((i >> 64) as u32);
        self.push((i >> 96) as u32);
    }
    #[cfg(target_pointer_width = "32")]
    fn write_usize(&mut self, i: usize) {
        self.write_u32(i as u32)
    }
    #[cfg(target_pointer_width = "64")]
    fn write_usize(&mut self, i: usize) {
        self.write_u64(i as u64)
    }
    fn write_i8(&mut self, i: i8) {
        self.write_u8(i as u8)
    }
    fn write_i16(&mut self, i: i16) {
        self.write_u16(i as u16)
    }
    fn write_i32(&mut self, i: i32) {
        self.write_u32(i as u32)
    }
    fn write_i64(&mut self, i: i64) {
        self.write_u64(i as u64)
    }
    fn write_i128(&mut self, i: i128) {
        self.write_u128(i as u128)
    }
    fn write_isize(&mut self, i: isize) {
        self.write_usize(i as usize)
    }
}

#[derive(Debug, Default)]
pub struct FibonacciHasheru64 {
    hash: u64,
}
impl FibonacciHasheru64 {
    const A: u64 = 11400714819323198485;
    const B: u32 = 5;
    fn push(&mut self, x: u64) {
        self.hash = (self.hash.rotate_right(Self::B) ^ x).wrapping_mul(Self::A);
    }
}
impl Hasher for FibonacciHasheru64 {
    fn finish(&self) -> u64 {
        self.hash
    }
    fn write(&mut self, mut bytes: &[u8]) {
        if bytes.len() % 8 >= 4 {
            self.push(u16::from_ne_bytes(bytes[..4].try_into().unwrap()) as u64);
            bytes = &bytes[4..];
        }
        if bytes.len() % 4 >= 2 {
            self.push(u16::from_ne_bytes(bytes[..2].try_into().unwrap()) as u64);
            bytes = &bytes[2..];
        }
        if bytes.len() % 2 >= 1 {
            self.push(u16::from_ne_bytes(bytes[..1].try_into().unwrap()) as u64);
            bytes = &bytes[1..];
        }
        for chunk in bytes.chunks(8) {
            self.push(u64::from_ne_bytes(chunk.try_into().unwrap()));
        }
    }
    fn write_u8(&mut self, i: u8) {
        self.push(i as u64)
    }
    fn write_u16(&mut self, i: u16) {
        self.push(i as u64)
    }
    fn write_u32(&mut self, i: u32) {
        self.push(i as u64)
    }
    fn write_u64(&mut self, i: u64) {
        self.push(i);
    }
    fn write_u128(&mut self, i: u128) {
        self.push(i as u64);
        self.push((i >> 64) as u64);
    }
    fn write_usize(&mut self, i: usize) {
        self.write_u64(i as u64)
    }
    fn write_i8(&mut self, i: i8) {
        self.write_u8(i as u8)
    }
    fn write_i16(&mut self, i: i16) {
        self.write_u16(i as u16)
    }
    fn write_i32(&mut self, i: i32) {
        self.write_u32(i as u32)
    }
    fn write_i64(&mut self, i: i64) {
        self.write_u64(i as u64)
    }
    fn write_i128(&mut self, i: i128) {
        self.write_u128(i as u128)
    }
    fn write_isize(&mut self, i: isize) {
        self.write_usize(i as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    use std::collections::BTreeSet;

    #[test]
    fn test_hash_slice() {
        const Q: usize = 50_000;

        let mut rng = Xorshift::default();
        let mut fh = FibHashSet::default();
        let mut rh = HashSet::new();
        for _ in 0..Q {
            let n = rng.gen(0..20);
            let a: Vec<_> = rng.gen_iter(0u64..).take(n).collect();
            fh.insert(a.to_vec());
            rh.insert(a.to_vec());
        }

        let fh = fh.into_iter().collect::<BTreeSet<_>>();
        let rh = rh.into_iter().collect::<BTreeSet<_>>();
        assert_eq!(fh, rh);
    }
}

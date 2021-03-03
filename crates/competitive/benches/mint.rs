#![feature(test)]
extern crate test;

use competitive::{num::*, rand, tools::Xorshift};

type M = mint_basic::MInt998244353;
const Q: usize = 100_000;

#[bench]
fn bench_base_mul(b: &mut test::Bencher) {
    let mut rng = Xorshift::default();
    rand!(rng, iter: [..M::get_mod()]);
    let v: Vec<_> = iter.take(Q).map(M::from).collect();
    b.iter(|| v.iter().product::<M>())
}

#[bench]
fn bench_montgomery_mul(b: &mut test::Bencher) {
    let mut rng = Xorshift::default();
    rand!(rng, iter: [..M::get_mod()]);
    let v: Vec<_> = iter.take(Q).map(M::from).collect();
    b.iter(|| v.iter().product::<M>())
}

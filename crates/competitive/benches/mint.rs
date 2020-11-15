#![feature(test)]
extern crate test;

use competitive::num::*;

#[bench]
fn bench_base_mul(b: &mut test::Bencher) {
    use competitive::tools::Xorshift;
    let mut xor = Xorshift::default();
    type M = mint_base::MInt998244353;
    const Q: usize = 100_000;
    let v = (0..Q).map(|_| M::from(xor.rand64())).collect::<Vec<_>>();
    b.iter(|| v.iter().product::<M>())
}

#[bench]
fn bench_montgomery_mul(b: &mut test::Bencher) {
    use competitive::tools::Xorshift;
    let mut xor = Xorshift::default();
    type M = montgomery::MInt998244353;
    const Q: usize = 100_000;
    let v = (0..Q).map(|_| M::from(xor.rand64())).collect::<Vec<_>>();
    b.iter(|| v.iter().product::<M>())
}

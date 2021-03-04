#![feature(test)]
extern crate test;

use competitive::{math::*, rand, tools::Xorshift};

#[bench]
fn bench_gcd(b: &mut test::Bencher) {
    let mut rng = Xorshift::default();
    const Q: usize = 10_000;
    rand!(rng, v: [(0u64.., 0u64..); Q]);
    b.iter(|| {
        let mut x = 0;
        for &(a, b) in &v {
            x ^= gcd(a, b);
        }
        x
    })
}

#[bench]
fn bench_gcd_binary(b: &mut test::Bencher) {
    let mut rng = Xorshift::default();
    const Q: usize = 10_000;
    rand!(rng, v: [(0u64.., 0u64..); Q]);
    b.iter(|| {
        let mut x = 0;
        for &(a, b) in &v {
            x ^= gcd_binary(a, b);
        }
        x
    })
}

#[bench]
fn bench_extgcd(b: &mut test::Bencher) {
    let mut rng = Xorshift::default();
    const Q: usize = 10_000;
    const M: i64 = 1_000_000_007;
    rand!(rng, v: [(0..M, 0..M); Q]);
    b.iter(|| {
        let mut x = 0;
        for &(a, b) in &v {
            x ^= extgcd_loop(a, b).0;
        }
        x
    })
}

#[bench]
fn bench_extgcd_binary(b: &mut test::Bencher) {
    let mut rng = Xorshift::default();
    const Q: usize = 10_000;
    const M: i64 = 1_000_000_007;
    rand!(rng, v: [(0..M, 0..M); Q]);
    b.iter(|| {
        let mut x = 0;
        for &(a, b) in &v {
            x ^= extgcd_binary(a, b).0;
        }
        x
    })
}

#[bench]
fn bench_modinv(b: &mut test::Bencher) {
    let mut rng = Xorshift::default();
    const Q: usize = 10_000;
    const M: i64 = 1_000_000_007;
    rand!(rng, v: [1..M; Q]);
    b.iter(|| {
        let mut x = 0;
        for &a in &v {
            x ^= modinv(a, M);
        }
        x
    })
}

#[bench]
fn bench_modinv_loop(b: &mut test::Bencher) {
    const M: i64 = 1_000_000_007;
    let mut rng = Xorshift::default();
    const Q: usize = 10_000;
    rand!(rng, v: [1..M; Q]);
    b.iter(|| {
        let mut x = 0;
        for &a in &v {
            x ^= modinv_loop(a, M);
        }
        x
    })
}

#[bench]
fn bench_modinv_extgcd_binary(b: &mut test::Bencher) {
    const M: u64 = 1_000_000_007;
    let mut rng = Xorshift::default();
    const Q: usize = 10_000;
    rand!(rng, v: [1..M; Q]);
    b.iter(|| {
        let mut x = 0;
        for &a in &v {
            x ^= modinv_extgcd_binary(a, M);
        }
        x
    })
}

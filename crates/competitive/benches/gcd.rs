#![feature(test)]
extern crate test;

use competitive::math::*;

#[bench]
fn bench_gcd(b: &mut test::Bencher) {
    use competitive::tools::Xorshift;
    let mut xor = Xorshift::default();
    const Q: usize = 10_000;
    let v = (0..Q)
        .map(|_| (xor.rand64(), xor.rand64()))
        .collect::<Vec<_>>();
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
    use competitive::tools::Xorshift;
    let mut xor = Xorshift::default();
    const Q: usize = 10_000;
    let v = (0..Q)
        .map(|_| (xor.rand64(), xor.rand64()))
        .collect::<Vec<_>>();
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
    use competitive::tools::Xorshift;
    let mut xor = Xorshift::default();
    const Q: usize = 10_000;
    let v = (0..Q)
        .map(|_| {
            (
                xor.rand(1_000_000_007) as i64,
                xor.rand(1_000_000_007) as i64,
            )
        })
        .collect::<Vec<_>>();
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
    use competitive::tools::Xorshift;
    let mut xor = Xorshift::default();
    const Q: usize = 10_000;
    let v = (0..Q)
        .map(|_| {
            (
                xor.rand(1_000_000_007) as i64,
                xor.rand(1_000_000_007) as i64,
            )
        })
        .collect::<Vec<_>>();
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
    use competitive::tools::Xorshift;
    const M: i64 = 1_000_000_007;
    let mut xor = Xorshift::default();
    const Q: usize = 10_000;
    let v = (0..Q)
        .map(|_| xor.rand(M as u64 - 1) as i64 + 1)
        .collect::<Vec<_>>();
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
    use competitive::tools::Xorshift;
    const M: i64 = 1_000_000_007;
    let mut xor = Xorshift::default();
    const Q: usize = 10_000;
    let v = (0..Q)
        .map(|_| xor.rand(M as u64 - 1) as i64 + 1)
        .collect::<Vec<_>>();
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
    use competitive::tools::Xorshift;
    const M: u64 = 1_000_000_007;
    let mut xor = Xorshift::default();
    const Q: usize = 10_000;
    let v = (0..Q).map(|_| xor.rand(M - 1) + 1).collect::<Vec<_>>();
    b.iter(|| {
        let mut x = 0;
        for &a in &v {
            x ^= modinv_extgcd_binary(a, M);
        }
        x
    })
}

use competitive::{math::*, tools::Xorshift};
use criterion::{BatchSize, Criterion};

pub fn bench_gcd(c: &mut Criterion) {
    let spec = (0u64.., 0u64..);
    let mut group = c.benchmark_group("gcd");
    group.bench_function("gcd", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(|| rng.gen(&spec), |(a, b)| gcd(a, b), BatchSize::SmallInput)
    });
    group.bench_function("gcd_binary", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(
            || rng.gen(&spec),
            |(a, b)| gcd_binary(a, b),
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

pub fn bench_extgcd(c: &mut Criterion) {
    const M: i64 = 1_000_000_007;
    let spec = (0..M, 0..M);
    let mut group = c.benchmark_group("extgcd");
    group.bench_function("extgcd", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(
            || rng.gen(&spec),
            |(a, b)| extgcd(a, b),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("extgcd_loop", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(
            || rng.gen(&spec),
            |(a, b)| extgcd_loop(a, b),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("extgcd_binary", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(
            || rng.gen(&spec),
            |(a, b)| extgcd_binary(a, b),
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

pub fn bench_modinv(c: &mut Criterion) {
    const M: i64 = 1_000_000_007;
    let spec = 1..M;
    let mut group = c.benchmark_group("modinv");
    group.bench_function("modinv", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(|| rng.gen(&spec), |a| modinv(a, M), BatchSize::SmallInput)
    });
    group.bench_function("modinv_loop", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(
            || rng.gen(&spec),
            |a| modinv_loop(a, M),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("modinv_extgcd_binary", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(
            || rng.gen(&spec) as u64,
            |a| modinv_extgcd_binary(a, M as u64),
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

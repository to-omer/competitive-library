use competitive::{math::*, tools::Xorshift};
use criterion::{BatchSize, Criterion};

pub fn bench_gcd(c: &mut Criterion) {
    let spec = (0u64.., 0u64..);
    let mut group = c.benchmark_group("gcd_loop");
    group.bench_function("gcd_loop", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(
            || rng.random(&spec),
            |(a, b)| gcd_loop(a, b),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("gcd_binary", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(
            || rng.random(&spec),
            |(a, b)| gcd(a, b),
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

pub fn bench_extgcd(c: &mut Criterion) {
    const M: i64 = 1_000_000_007;
    let spec = (0..M, 0..M);
    let mut group = c.benchmark_group("extgcd");
    group.bench_function("extgcd_recurse", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(
            || rng.random(&spec),
            |(a, b)| extgcd_recurse(a, b),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("extgcd_loop", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(
            || rng.random(&spec),
            |(a, b)| extgcd(a, b),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("extgcd_binary", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(
            || rng.random(&spec),
            |(a, b)| extgcd_binary(a, b),
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

pub fn bench_modinv(c: &mut Criterion) {
    const M: u64 = 1_000_000_007;
    let spec = 1..M;
    let mut group = c.benchmark_group("modinv");
    group.bench_function("modinv_recurse", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(
            || rng.random(&spec),
            |a| modinv_recurse(a, M),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("modinv_loop", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(
            || rng.random(&spec),
            |a| modinv(a, M),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("modinv_extgcd_binary", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(
            || rng.random(&spec),
            |a| modinv_extgcd_binary(a, M),
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

use competitive::{num::BarrettReduction, tools::Xorshift};
use criterion::{BatchSize, Criterion};

pub fn bench_barrett_reduction_u32(c: &mut Criterion) {
    let spec = (0u32.., Xorshift::default().random(1u32..));
    let mut group = c.benchmark_group("barrett_reduction_u32");
    group.bench_function("barrett_reduction", |b| {
        let mut rng = Xorshift::default();
        let br = BarrettReduction::<u32>::new(spec.1);
        b.iter_batched(
            || rng.random(&spec),
            |(a, _b)| br.div_rem(a),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("naive_div_rem", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(
            || rng.random(&spec),
            |(a, b)| (a / b, a % b),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("naive_div", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(|| rng.random(&spec), |(a, b)| a / b, BatchSize::SmallInput)
    });
    group.bench_function("naive_rem", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(|| rng.random(&spec), |(a, b)| a % b, BatchSize::SmallInput)
    });
    group.finish();
}

pub fn bench_barrett_reduction_u64(c: &mut Criterion) {
    let spec = (0u64.., Xorshift::default().random(1u64..));
    let mut group = c.benchmark_group("barrett_reduction_u64");
    group.bench_function("barrett_reduction", |b| {
        let mut rng = Xorshift::default();
        let br = BarrettReduction::<u64>::new(spec.1);
        b.iter_batched(
            || rng.random(&spec),
            |(a, _b)| br.div_rem(a),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("naive_div_rem", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(
            || rng.random(&spec),
            |(a, b)| (a / b, a % b),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("naive_div", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(|| rng.random(&spec), |(a, b)| a / b, BatchSize::SmallInput)
    });
    group.bench_function("naive_rem", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(|| rng.random(&spec), |(a, b)| a % b, BatchSize::SmallInput)
    });
    group.finish();
}

pub fn bench_barrett_reduction_u128(c: &mut Criterion) {
    let mut rng = Xorshift::default();
    let spec = (0u64.., rng.random(1u64..) as u128 * rng.random(1u64..) as u128);
    let mut group = c.benchmark_group("barrett_reduction_u128");
    group.bench_function("barrett_reduction", |b| {
        let mut rng = Xorshift::default();
        let br = BarrettReduction::<u128>::new(spec.1);
        b.iter_batched(
            || (rng.random(&spec.0) as u128 * rng.random(&spec.0) as u128, spec.1),
            |(a, _b)| br.div_rem(a),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("naive_div_rem", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(
            || (rng.random(&spec.0) as u128 * rng.random(&spec.0) as u128, spec.1),
            |(a, b)| (a / b, a % b),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("naive_div", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(
            || (rng.random(&spec.0) as u128 * rng.random(&spec.0) as u128, spec.1),
            |(a, b)| a / b,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("naive_rem", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(
            || (rng.random(&spec.0) as u128 * rng.random(&spec.0) as u128, spec.1),
            |(a, b)| a % b,
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

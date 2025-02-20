use competitive::{
    algebra::{Gf2_63, Mersenne61, SemiRing},
    tools::Xorshift,
};
use criterion::{BatchSize, Criterion};

pub fn bench_special_ring(c: &mut Criterion) {
    let mut group = c.benchmark_group("special_ring");
    group.bench_function("gf2_63", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(
            || rng.random((..Gf2_63::MOD, ..Gf2_63::MOD)),
            |(a, b)| Gf2_63::mul(&a, &b),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("mersenne_61", |b| {
        let mut rng = Xorshift::default();
        b.iter_batched(
            || rng.random((..Mersenne61::MOD, ..Mersenne61::MOD)),
            |(a, b)| Mersenne61::mul(&a, &b),
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

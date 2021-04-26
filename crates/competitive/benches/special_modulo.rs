use competitive::{math::*, num::MInt, tools::Xorshift};
use criterion::{BatchSize, Criterion};

pub fn bench_special_modulo(c: &mut Criterion) {
    let mut group = c.benchmark_group("special_modulo");
    group.bench_function("mersenne_61", |b| {
        let mut rng = Xorshift::default();
        type M = MInt<Mersenne61>;
        b.iter_batched(
            || {
                let (a, b) = rng.gen(&(..M::get_mod(), ..M::get_mod()));
                (M::new_unchecked(a), M::new_unchecked(b))
            },
            |(a, b)| a * b,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("mersenne_127", |b| {
        let mut rng = Xorshift::default();
        type M = MInt<Mersenne127>;
        b.iter_batched(
            || {
                let (a, b) = (
                    ((rng.rand64() as u128) << 64 | rng.rand64() as u128) % M::get_mod(),
                    ((rng.rand64() as u128) << 64 | rng.rand64() as u128) % M::get_mod(),
                );
                (M::new_unchecked(a), M::new_unchecked(b))
            },
            |(a, b)| a * b,
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

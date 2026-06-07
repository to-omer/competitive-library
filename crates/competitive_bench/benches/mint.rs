use competitive::{math::FastPrimeMod, num::*, tools::Xorshift};
use criterion::{BatchSize, Criterion};

pub fn bench_mod_mul(c: &mut Criterion) {
    const A: u32 = 998244353;
    let spec = (..A, ..A);
    let mut group = c.benchmark_group("mod_mul");
    group.bench_function("const_mod_mul", |b| {
        type M = mint_basic::MInt998244353;
        let mut rng = Xorshift::default();
        b.iter_batched(
            || {
                let (a, b) = rng.random(spec);
                (M::new_unchecked(a), M::new_unchecked(b))
            },
            |(a, b)| a * b,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("montgomery_mul", |b| {
        type M = montgomery::MInt998244353;
        let mut rng = Xorshift::default();
        b.iter_batched(
            || {
                let (a, b) = rng.random(spec);
                (M::new_unchecked(a), M::new_unchecked(b))
            },
            |(a, b)| a * b,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("dynmint_mul", |b| {
        type M = mint_basic::DynMIntU32;
        M::set_mod(A);
        let mut rng = Xorshift::default();
        b.iter_batched(
            || {
                let (a, b) = rng.random(spec);
                (M::new_unchecked(a), M::new_unchecked(b))
            },
            |(a, b)| a * b,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("barrett_reduction_rem", |b| {
        let mut rng = Xorshift::default();
        let br = BarrettReduction::<u64>::new(A as _);
        b.iter_batched(
            || {
                let (a, b) = rng.random(spec);
                (a as u64, b as u64)
            },
            |(a, b)| br.rem(a * b) as u32,
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn mod_pow_barrett(a: u32, mut exp: usize, br: &BarrettReduction<u64>) -> u32 {
    let mut x = a;
    let mut y = 1;
    while exp > 0 {
        if exp & 1 == 1 {
            y = br.rem(y as u64 * x as u64) as u32;
        }
        x = br.rem(x as u64 * x as u64) as u32;
        exp >>= 1;
    }
    y
}

pub fn bench_mod_pow(c: &mut Criterion) {
    const A: u32 = 998244353;
    let spec = (0..A, 0usize..);
    let mut group = c.benchmark_group("mod_pow");
    group.bench_function("const_mod_pow", |b| {
        type M = mint_basic::MInt998244353;
        let mut rng = Xorshift::default();
        b.iter_batched(
            || {
                let (a, exp) = rng.random(&spec);
                (M::new_unchecked(a), exp)
            },
            |(a, exp)| a.pow(exp),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("montgomery_pow", |b| {
        type M = montgomery::MInt998244353;
        let mut rng = Xorshift::default();
        b.iter_batched(
            || {
                let (a, exp) = rng.random(&spec);
                (M::new_unchecked(a), exp)
            },
            |(a, exp)| a.pow(exp),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("dynmint_pow", |b| {
        type M = mint_basic::DynMIntU32;
        M::set_mod(A);
        let mut rng = Xorshift::default();
        b.iter_batched(
            || {
                let (a, exp) = rng.random(&spec);
                (M::new_unchecked(a), exp)
            },
            |(a, exp)| a.pow(exp),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("barrett_reduction_pow", |b| {
        let br = BarrettReduction::<u64>::new(A as u64);
        let mut rng = Xorshift::default();
        b.iter_batched(
            || rng.random(&spec),
            |(a, exp)| mod_pow_barrett(a, exp, &br),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("fast_prime_mod_pow", |b| {
        let fast = FastPrimeMod::<998_244_353, false, true>::new();
        let mut rng = Xorshift::default();
        b.iter_batched(
            || rng.random(&spec),
            |(a, exp)| fast.pow(a, exp as u64),
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

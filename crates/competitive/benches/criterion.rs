use criterion::{criterion_group, criterion_main};

mod gcd;
mod mint;

criterion_group!(
    benches,
    gcd::bench_gcd,
    gcd::bench_extgcd,
    gcd::bench_modinv,
    mint::bench_mod_mul
);

criterion_main!(benches);
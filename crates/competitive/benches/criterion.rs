use criterion::{criterion_group, criterion_main, Criterion};
use std::time::Duration;

mod gcd;
mod mint;
mod special_modulo;

criterion_group!(
    name = small_benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(1));
    targets = gcd::bench_gcd,
        gcd::bench_extgcd,
        gcd::bench_modinv,
        mint::bench_mod_mul,
        special_modulo::bench_special_modulo,
);

criterion_main!(small_benches);

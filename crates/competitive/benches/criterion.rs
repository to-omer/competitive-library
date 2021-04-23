use criterion::{criterion_group, criterion_main};

mod fft;
mod gcd;
mod mint;
mod ntt;

criterion_group!(
    benches,
    gcd::bench_gcd,
    gcd::bench_extgcd,
    gcd::bench_modinv,
    mint::bench_mod_mul,
    fft::bench_convolve,
    ntt::bench_convolve,
);

criterion_main!(benches);

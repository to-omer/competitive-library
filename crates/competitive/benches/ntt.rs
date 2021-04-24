use competitive::{math::*, num::*, tools::Xorshift};
use criterion::{AxisScale, BenchmarkId, Criterion, PlotConfiguration, Throughput};

pub fn bench_convolve(c: &mut Criterion) {
    const SIZE: [usize; 4] = [1 << 6, 1 << 10, 1 << 14, 1 << 18];
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let mut group = c.benchmark_group("convolve_ntt");
    group.plot_config(plot_config);
    for size in SIZE.iter().cloned() {
        let mut rng = Xorshift::default();
        group.throughput(Throughput::Elements(size as _));
        let x: Vec<_> = rng
            .gen_iter(..mint_basic::MInt998244353::get_mod())
            .map(mint_basic::MInt998244353::new_unchecked)
            .take(size)
            .collect();
        let y: Vec<_> = rng
            .gen_iter(..mint_basic::MInt998244353::get_mod())
            .map(mint_basic::MInt998244353::new_unchecked)
            .take(size)
            .collect();
        group.bench_with_input(BenchmarkId::from_parameter(size), &(x, y), |b, (x, y)| {
            b.iter_with_large_drop(|| Ntt998244353::convolve(x.to_owned(), y.to_owned()))
        });
    }
    group.finish();
}

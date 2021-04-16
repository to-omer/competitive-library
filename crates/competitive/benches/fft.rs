use competitive::{math::*, rand, tools::Xorshift};
use criterion::{AxisScale, BenchmarkId, Criterion, PlotConfiguration, Throughput};

pub fn bench_convolve(c: &mut Criterion) {
    const SIZE: [usize; 4] = [1 << 6, 1 << 10, 1 << 14, 1 << 18];
    const A: i32 = 100;

    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let mut group = c.benchmark_group("convolve_fft");
    group.plot_config(plot_config);
    for size in SIZE.iter().cloned() {
        let mut rng = Xorshift::default();
        group.throughput(Throughput::Elements(size as _));
        rand!(rng, x: [0..=A; size], y: [0..=A; size]);
        group.bench_with_input(BenchmarkId::from_parameter(size), &(&x, &y), |b, (x, y)| {
            b.iter_with_large_drop(|| convolve_fft(x.iter().cloned(), y.iter().cloned()))
        });
    }
    group.finish();
}

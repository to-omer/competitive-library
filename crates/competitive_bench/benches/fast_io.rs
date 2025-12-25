use std::io::{BufWriter, Write};

use competitive::tools::{FastInput, FastOutput};
use criterion::{BatchSize, Criterion};

pub fn bench_fast_input_u32(c: &mut Criterion) {
    let mut group = c.benchmark_group("fast_input_u32");
    for i in 0..=9 {
        let x = 10u32.pow(i);
        let s = format!("{}\n", x).repeat(100);
        group.bench_function(criterion::BenchmarkId::new("fast_input", i), |b| {
            b.iter_batched(
                || unsafe { FastInput::from_slice(s.as_bytes()) },
                |mut fi| unsafe { fi.u32() },
                BatchSize::SmallInput,
            )
        });
        group.bench_function(criterion::BenchmarkId::new("from_str", i), |b| {
            b.iter_batched(
                || unsafe { FastInput::from_slice(s.as_bytes()) },
                |mut fi| unsafe { fi.parse::<u32>() },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

pub fn bench_fast_input_u64(c: &mut Criterion) {
    let mut group = c.benchmark_group("fast_input_u64");
    for i in 0..=19 {
        let x = 10u64.pow(i);
        let s = format!("{}\n", x).repeat(100);
        group.bench_function(criterion::BenchmarkId::new("fast_input", i), |b| {
            b.iter_batched(
                || unsafe { FastInput::from_slice(s.as_bytes()) },
                |mut fi| unsafe { fi.u64() },
                BatchSize::SmallInput,
            )
        });
        group.bench_function(criterion::BenchmarkId::new("from_str", i), |b| {
            b.iter_batched(
                || unsafe { FastInput::from_slice(s.as_bytes()) },
                |mut fi| unsafe { fi.parse::<u64>() },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

pub fn bench_fast_output_u32(c: &mut Criterion) {
    let mut group = c.benchmark_group("fast_output_u32");
    for i in 0..=9 {
        let x = 10u32.pow(i);
        group.bench_function(criterion::BenchmarkId::new("fast_output", i), |b| {
            b.iter_batched(
                || FastOutput::with_capacity(1024, std::io::empty()),
                |mut fo| fo.u32(x),
                BatchSize::SmallInput,
            )
        });
        group.bench_function(criterion::BenchmarkId::new("to_string", i), |b| {
            b.iter_batched(
                || BufWriter::new(std::io::empty()),
                |mut bw| write!(bw, "{}", x).unwrap(),
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

pub fn bench_fast_output_u64(c: &mut Criterion) {
    let mut group = c.benchmark_group("fast_output_u64");
    for i in 0..=19 {
        let x = 10u64.pow(i);
        group.bench_function(criterion::BenchmarkId::new("fast_output", i), |b| {
            b.iter_batched(
                || FastOutput::with_capacity(1024, std::io::empty()),
                |mut fo| fo.u64(x),
                BatchSize::SmallInput,
            )
        });
        group.bench_function(criterion::BenchmarkId::new("to_string", i), |b| {
            b.iter_batched(
                || BufWriter::new(std::io::empty()),
                |mut bw| write!(bw, "{}", x).unwrap(),
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

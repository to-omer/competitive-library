use std::{
    hint::black_box,
    io::{BufWriter, Write},
};

use competitive::tools::{FastInput, FastOutput, Xorshift};
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput};

pub fn bench_fast_input_u32(c: &mut Criterion) {
    let mut group = c.benchmark_group("fast_input_u32");
    group.throughput(Throughput::Elements(4096));
    let mut rng = Xorshift::default();
    for i in 0..=10 {
        let values: Vec<u32> = if i == 10 {
            (0..4096)
                .map(|_| {
                    let shift = rng.random(0..u32::BITS);
                    (rng.rand64() as u32) >> shift
                })
                .collect()
        } else {
            vec![10u32.pow(i); 4096]
        };
        let name = if i == 10 {
            "mixed".to_owned()
        } else {
            i.to_string()
        };
        let mut input = Vec::new();
        for x in &values {
            writeln!(input, "{x}").unwrap();
        }
        input.extend_from_slice(&[b' '; 32]);
        group.bench_function(BenchmarkId::new("fast_input", &name), |b| {
            b.iter(|| {
                let mut fi = unsafe { FastInput::from_slice(black_box(&input)) };
                let mut sum = 0u32;
                for _ in 0..values.len() {
                    sum = sum.wrapping_add(unsafe { fi.u32() });
                }
                black_box(sum)
            })
        });
        group.bench_function(BenchmarkId::new("from_str", &name), |b| {
            b.iter(|| {
                let mut fi = unsafe { FastInput::from_slice(black_box(&input)) };
                let mut sum = 0u32;
                for _ in 0..values.len() {
                    let x: u32 = unsafe { fi.parse() };
                    sum = sum.wrapping_add(x);
                }
                black_box(sum)
            })
        });
    }
    group.finish();
}

pub fn bench_fast_output_u32(c: &mut Criterion) {
    let mut group = c.benchmark_group("fast_output_u32");
    group.throughput(Throughput::Elements(4096));
    let mut rng = Xorshift::default();
    for i in 0..=10 {
        let values: Vec<u32> = if i == 10 {
            (0..4096)
                .map(|_| {
                    let shift = rng.random(0..u32::BITS);
                    (rng.rand64() as u32) >> shift
                })
                .collect()
        } else {
            vec![10u32.pow(i); 4096]
        };
        let name = if i == 10 {
            "mixed".to_owned()
        } else {
            i.to_string()
        };
        group.bench_function(BenchmarkId::new("fast_output", &name), |b| {
            b.iter_batched(
                || FastOutput::with_capacity(1024, Vec::with_capacity(values.len() * 11)),
                |mut fo| {
                    for &x in black_box(&values) {
                        fo.u32(x);
                        fo.byte(b'\n');
                    }
                    fo.flush();
                    black_box(fo)
                },
                BatchSize::LargeInput,
            )
        });
        group.bench_function(BenchmarkId::new("to_string", &name), |b| {
            b.iter_batched(
                || BufWriter::with_capacity(1024, Vec::with_capacity(values.len() * 11)),
                |mut bw| {
                    for &x in black_box(&values) {
                        writeln!(bw, "{x}").unwrap();
                    }
                    bw.flush().unwrap();
                    black_box(bw)
                },
                BatchSize::LargeInput,
            )
        });
    }
    group.finish();
}

pub fn bench_fast_input_u64(c: &mut Criterion) {
    let mut group = c.benchmark_group("fast_input_u64");
    group.throughput(Throughput::Elements(4096));
    let mut rng = Xorshift::default();
    for i in 0..=20 {
        let values: Vec<u64> = if i == 20 {
            (0..4096)
                .map(|_| {
                    let shift = rng.random(0..u64::BITS);
                    rng.rand64() >> shift
                })
                .collect()
        } else {
            vec![10u64.pow(i); 4096]
        };
        let name = if i == 20 {
            "mixed".to_owned()
        } else {
            i.to_string()
        };
        let mut input = Vec::new();
        for x in &values {
            writeln!(input, "{x}").unwrap();
        }
        input.extend_from_slice(&[b' '; 32]);
        group.bench_function(BenchmarkId::new("fast_input", &name), |b| {
            b.iter(|| {
                let mut fi = unsafe { FastInput::from_slice(black_box(&input)) };
                let mut sum = 0u64;
                for _ in 0..values.len() {
                    sum = sum.wrapping_add(unsafe { fi.u64() });
                }
                black_box(sum)
            })
        });
        group.bench_function(BenchmarkId::new("from_str", &name), |b| {
            b.iter(|| {
                let mut fi = unsafe { FastInput::from_slice(black_box(&input)) };
                let mut sum = 0u64;
                for _ in 0..values.len() {
                    let x: u64 = unsafe { fi.parse() };
                    sum = sum.wrapping_add(x);
                }
                black_box(sum)
            })
        });
    }
    group.finish();
}

pub fn bench_fast_output_u64(c: &mut Criterion) {
    let mut group = c.benchmark_group("fast_output_u64");
    group.throughput(Throughput::Elements(4096));
    let mut rng = Xorshift::default();
    for i in 0..=20 {
        let values: Vec<u64> = if i == 20 {
            (0..4096)
                .map(|_| {
                    let shift = rng.random(0..u64::BITS);
                    rng.rand64() >> shift
                })
                .collect()
        } else {
            vec![10u64.pow(i); 4096]
        };
        let name = if i == 20 {
            "mixed".to_owned()
        } else {
            i.to_string()
        };
        group.bench_function(BenchmarkId::new("fast_output", &name), |b| {
            b.iter_batched(
                || FastOutput::with_capacity(1024, Vec::with_capacity(values.len() * 21)),
                |mut fo| {
                    for &x in black_box(&values) {
                        fo.u64(x);
                        fo.byte(b'\n');
                    }
                    fo.flush();
                    black_box(fo)
                },
                BatchSize::LargeInput,
            )
        });
        group.bench_function(BenchmarkId::new("to_string", &name), |b| {
            b.iter_batched(
                || BufWriter::with_capacity(1024, Vec::with_capacity(values.len() * 21)),
                |mut bw| {
                    for &x in black_box(&values) {
                        writeln!(bw, "{x}").unwrap();
                    }
                    bw.flush().unwrap();
                    black_box(bw)
                },
                BatchSize::LargeInput,
            )
        });
    }
    group.finish();
}

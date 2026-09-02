# competitive SIMD benchmark v5: AtCoder run instructions

## Immutable inputs

- Timing source: `util/simd_benchmark_v2/methods.rs`
  - SHA-256: `7961b05c8156ded44e54886408a4c4400aae4739ece6693a8a56be1a242358e8`
- Memory source: `util/simd_benchmark_v2/memory.rs`
  - SHA-256: `a4f1546fd827f874f576b25ab350d09fbb95d4ccccbca9c6cda64dbbbac5dde3`

Do not edit either source. Use Rust 1.89.0 with the default target CPU:

```sh
rustc --edition=2024 -C opt-level=3 -C lto=off methods.rs -o timing
rustc --edition=2024 -C opt-level=3 -C lto=off memory.rs -o memory
```

The source fixes the seed, five measured rounds, alternating implementation order, workload, and
checksum. On Linux it measures thread CPU time. Before the measured rounds, every case calibrates a
single power-of-two repetition count shared by all implementations until the fastest implementation
reaches two milliseconds. `units` includes this repetition count and `repetitions` records it.
Each suite must be a separate execution so the 10-second limit does not combine unrelated cases.

## Timing executions

Run `timing` once for each of these stdin values and preserve stdout verbatim:

```text
0
10
11
12
13
14
15
20
21
22
23
24
25
30
31
32
33
34
35
36
37
38
39
40
41
50
51
60
61
62
63
64
65
66
67
68
69
```

Coverage:

- 0: CPU/cache and isolated scalar/AVX2/AVX-512 search, suffix-add, range-minimum,
  and child-maximum kernels
- 10-11: BitVector methods, build paths, select scalar/BMI2
- 12-15: WaveletMatrix single/batch/Fold/PointAdd and existing 2D BIT baseline
- 20-23: StaticSearch U8/I8/U16/I16, one type per suite
- 24-25: remaining StaticSearch widths, signed keys, custom SimdKey, direct/slice baseline
- 30: BucketQueue U8/I8/U16/I16 and BinaryHeap
- 31-39: DaryHeap every supported width/backend, size, distribution, and public operation
- 40-41: WidePrefix U32/U64 every public operation and the repository's
  `BinaryIndexedTree<AdditiveOperation<Wrapping<_>>>` baseline
- 50-51: WideSegmentTree min/max I32/I64 every public operation, SegmentTree and RMQ baselines
- 60-67: production/candidate RadixHeap widths, delta distributions, Binary/Dary/Bucket, Dijkstra
- 68: existing PairingHeap U128 versus BinaryHeap and DaryHeap
- 69: signed RadixHeap I8/I16/I32/I64/I128 small-delta pop+push

Heap replacement baselines use `BinaryHeap::peek_mut`. Radix steady-state cases retain separately
labeled `pop+push` and replacement-capable BinaryHeap/DaryHeap/BucketQueue paths. Do not validate
against a fixed row count: x86_64 environments emit extra AVX2/AVX-512 rows according to runtime
feature detection. Validate the suite set, five samples per emitted row, calibrated units, and
checksums instead.

Every execution must contain its `benchmark=... suite=...` header. Suite 0 must contain
`self_test=ok`. Stop and report the exact output if any assertion, panic, or checksum mismatch occurs.

## Memory executions

Run `memory` once for each stdin value and preserve stdout verbatim:

```text
70
71
72
```

- 70: broad current/legacy/existing live and peak allocation comparison
- 71: WaveletMatrix PointAdd versus existing CompressedBinaryIndexedTree2d
- 72: RadixHeap U8/U16/U32/U64/U128, BinaryHeap, BucketQueue, object sizes

Each execution must end with `self_test=ok`.

## Environment data to preserve

Record the exact AtCoder compiler version, CPU model/vendor/flags when available, kernel/OS,
logical CPU count, and cache sizes. Do not add `target-cpu=native`; the common source is deliberately
compiled with target CPU defaults and performs runtime dispatch.

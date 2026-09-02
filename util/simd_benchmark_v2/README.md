# SIMD benchmark v3

These standalone Rust 1.89 sources are the immutable cross-environment benchmark inputs.

- `methods.rs` SHA-256: `abc1a320228c9b53034b24eda854270879a00dfa68bc509d0c43fda0e9c6be61`
- `memory.rs` SHA-256: `a4f1546fd827f874f576b25ab350d09fbb95d4ccccbca9c6cda64dbbbac5dde3`

Compile without changing either source:

```sh
rustc --edition=2024 -C opt-level=3 -C lto=off methods.rs -o methods
rustc --edition=2024 -C opt-level=3 -C lto=off memory.rs -o memory
```

Run each timing suite in a separate process:

```text
0 10 11 12 13 14 15 20 21 22 30 31 32 33 34 35 36 37 38 39 40 41 50 51 60 61 62 63 64 65 66 67 68 69
```

Run memory suites `70`, `71`, and `72` separately. Preserve raw stdout and report the compiler, CPU, kernel, cache hierarchy, and CPU flags. Any failed self-test or mismatched checksum invalidates the run.

Heap replacement workloads use `BinaryHeap::peek_mut` as the standard-library baseline. Radix-heap steady-state comparisons retain individually labeled `pop+push` and replacement-capable baseline paths.

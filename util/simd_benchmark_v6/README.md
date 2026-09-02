# SIMD benchmark v6

This benchmark compares the candidate data structures with existing repository and standard-library
implementations one public method at a time. Mixed workloads are not used for adoption decisions.

Immutable submission sources:

- `timing/src/main.rs`: `14528e311f16ceeab2d94308c90f427720324de20cf1378951410a5517bf4af4`
- `memory/src/main.rs`: `126e99589be3ab6d432a4474246f8a818c6d2fcfe9cf92ef3fc3c6a6b01144e8`

The timing and memory programs are separate Cargo projects so both environments use the same build
path as AtCoder's Rust 1.89.0 language image:

```text
cargo build --release --quiet --offline
[profile.release]
lto = true
rustflags = ["--cfg", "atcoder"]
```

Do not add `target-cpu=native`, extra `RUSTFLAGS`, affinity, or priority settings. They would make the
two environments incomparable.

Build locally with Rust 1.89.0:

```sh
(cd timing && rustup run 1.89.0 cargo build --release --quiet --offline)
(cd memory && rustup run 1.89.0 cargo build --release --quiet --offline)
```

Run every suite in a separate process:

```sh
./run_timing.sh timing/target/release/competitive-simd-methods-v6 methods.raw
./run_memory.sh memory/target/release/competitive-simd-memory-v6 memory.raw
```

Timing suites:

```text
0 10 11 12 13 14 15 20 21 22 23 24 25 30 31 32 33 34 35 36 37 38 39 40 41 50 51 60 61 62 63 64 65 66 67 68 69
```

Memory suites are `70`, `71`, and `72`. Preserve raw stdout. A panic, failed self-test, checksum
mismatch, build-profile mismatch, or sample shorter than the parser's minimum invalidates the run.

The timing harness uses thread CPU time on Linux and macOS. It calibrates one shared power-of-two
repetition count per case until the fastest implementation reaches two milliseconds, then records
five rounds in alternating implementation order. Compare implementation ratios within an
environment; absolute nanoseconds across environments are diagnostic data, not adoption evidence.

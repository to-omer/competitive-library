#!/usr/bin/env bash
set -eu

for features in \
    "" \
    "+avx2" \
    "+avx512f,+avx512dq,+avx512cd,+avx512bw,+avx512vl"
do
    printf 'Verifying snippets: %s\n' "${features:-baseline}"
    cargo codesnip \
        --source-config=.github/workflows/codesnip.toml \
        verify \
        --verbose \
        --toolchain=1.89.0 \
        --edition=2024 \
        --target=x86_64-unknown-linux-gnu \
        --rustc-arg="-Ctarget-feature=${features}"
done

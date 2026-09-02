#!/bin/sh
set -eu

benchmark_binary=$1
raw_output=$2

: > "$raw_output"
for suite in 0 10 11 12 13 14 15 20 21 22 30 31 32 33 34 35 36 37 38 39 40 41 50 51 60 61 62 63 64 65 66 67 68 69; do
    echo "running_suite=$suite"
    printf '%s\n' "$suite" | "$benchmark_binary" >> "$raw_output"
done

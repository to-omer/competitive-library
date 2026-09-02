#!/bin/sh
set -eu

benchmark_binary=$1
raw_output=$2

: > "$raw_output"
for suite in 70 71 72; do
    echo "running_suite=$suite"
    printf '%s\n' "$suite" | "$benchmark_binary" >> "$raw_output"
done

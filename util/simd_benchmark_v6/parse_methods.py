#!/usr/bin/env python3
import argparse
import csv
import re


HEADER = re.compile(r"^benchmark=competitive_simd_methods_v6 suite=(\d+)\b")
BUILD_PROFILE = (
    "build_profile=atcoder_rust_1.89.0 cargo_release edition_2024 "
    "lto_true cfg_atcoder target_cpu_default"
)


def fields(line):
    return dict(part.split("=", 1) for part in line.split() if "=" in part)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--environment", required=True)
    parser.add_argument("--input", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--expected-suites")
    args = parser.parse_args()

    rows = []
    suites = []
    current_suite = None
    self_test_suites = set()
    build_flag_suites = set()
    header_environment = None
    with open(args.input) as file:
        for source_line in file:
            line = source_line.strip()
            match = HEADER.match(line)
            if match:
                current_suite = int(match.group(1))
                suites.append(current_suite)
                item = fields(line)
                expected_fields = {
                    "benchmark",
                    "suite",
                    "seed",
                    "rounds",
                    "calibration_target_ns",
                    "timer",
                    "order",
                    "arch",
                    "os",
                    "pointer_bits",
                    "bmi2",
                    "avx2",
                    "avx512f",
                    "avx512vl",
                }
                if set(item) != expected_fields:
                    raise ValueError(
                        f"unexpected header fields: {set(item) ^ expected_fields}: {line}"
                    )
                expected_constants = {
                    "benchmark": "competitive_simd_methods_v6",
                    "suite": str(current_suite),
                    "seed": "2611923443488327891",
                    "rounds": "5",
                    "calibration_target_ns": "2000000",
                    "timer": "thread_cpu_time",
                    "order": "alternating",
                    "pointer_bits": "64",
                }
                for key, value in expected_constants.items():
                    if item[key] != value:
                        raise ValueError(f"unexpected header {key}: {line}")
                for key in ("bmi2", "avx2", "avx512f", "avx512vl"):
                    if item[key] not in ("true", "false"):
                        raise ValueError(f"unexpected header {key}: {line}")
                environment = tuple(
                    item[key]
                    for key in (
                        "arch",
                        "os",
                        "pointer_bits",
                        "bmi2",
                        "avx2",
                        "avx512f",
                        "avx512vl",
                    )
                )
                if header_environment is None:
                    header_environment = environment
                elif header_environment != environment:
                    raise ValueError(
                        f"header environment changed: {header_environment} != {environment}"
                    )
                continue
            if line == BUILD_PROFILE:
                if current_suite is None:
                    raise ValueError("build flags before suite header")
                if current_suite in build_flag_suites:
                    raise ValueError(f"duplicate build flags for suite {current_suite}")
                build_flag_suites.add(current_suite)
                continue
            if line == "self_test=ok":
                if current_suite is None:
                    raise ValueError("self test before suite header")
                self_test_suites.add(current_suite)
                continue
            lowered = line.lower()
            if "panicked at" in lowered or "assertion failed" in lowered or "checksum mismatch" in lowered:
                raise ValueError(f"failed run: {line}")
            if not line.startswith("case="):
                continue
            if current_suite is None:
                raise ValueError(f"case before suite header: {line}")
            item = fields(line)
            required = {
                "case",
                "impl",
                "units",
                "repetitions",
                "raw_ns",
                "median_ns",
                "min_ns",
                "max_ns",
                "ns/unit",
                "checksum",
            }
            if set(item) != required:
                raise ValueError(f"unexpected fields: {set(item) ^ required}: {line}")
            raw = tuple(int(value) for value in item["raw_ns"].split(","))
            if len(raw) != 5:
                raise ValueError(f"expected five rounds: {line}")
            if int(item["median_ns"]) != sorted(raw)[2]:
                raise ValueError(f"median mismatch: {line}")
            if int(item["min_ns"]) != min(raw) or int(item["max_ns"]) != max(raw):
                raise ValueError(f"range mismatch: {line}")
            units = int(item["units"])
            repetitions = int(item["repetitions"])
            if repetitions <= 0 or units <= 0 or units % repetitions != 0:
                raise ValueError(f"invalid calibrated units: {line}")
            if int(item["median_ns"]) < 1_000_000:
                raise ValueError(f"sample shorter than one millisecond: {line}")
            expected_per_unit = int(item["median_ns"]) / units
            if abs(float(item["ns/unit"]) - expected_per_unit) > 0.000_501:
                raise ValueError(f"ns/unit mismatch: {line}")
            rows.append(
                {
                    "environment": args.environment,
                    "suite": current_suite,
                    "case": item["case"],
                    "implementation": item["impl"],
                    "units": item["units"],
                    "base_units": units // repetitions,
                    "repetitions": item["repetitions"],
                    "raw_ns": item["raw_ns"],
                    "median_ns": item["median_ns"],
                    "min_ns": item["min_ns"],
                    "max_ns": item["max_ns"],
                    "ns_per_unit": item["ns/unit"],
                    "checksum": item["checksum"],
                }
            )

    if len(suites) != len(set(suites)):
        raise ValueError(f"duplicate suite headers: {suites}")
    if args.expected_suites:
        expected = [int(value) for value in args.expected_suites.split(",")]
        if suites != expected:
            raise ValueError(f"suite order mismatch: got {suites}, expected {expected}")
    if build_flag_suites != set(suites):
        raise ValueError(
            f"build flag suites mismatch: got {sorted(build_flag_suites)}, "
            f"expected {sorted(suites)}"
        )
    if 0 in suites and 0 not in self_test_suites:
        raise ValueError("suite 0 did not report self_test=ok")
    if self_test_suites - {0}:
        raise ValueError(f"unexpected self test suites: {sorted(self_test_suites)}")
    row_suites = {row["suite"] for row in rows}
    if row_suites != set(suites):
        raise ValueError(
            f"row suites mismatch: got {sorted(row_suites)}, expected {sorted(suites)}"
        )
    keys = [(row["suite"], row["case"], row["implementation"]) for row in rows]
    if len(keys) != len(set(keys)):
        raise ValueError("duplicate benchmark rows")
    by_case = {}
    for row in rows:
        by_case.setdefault((row["suite"], row["case"]), []).append(row)
    for key, items in by_case.items():
        if len({item["units"] for item in items}) != 1:
            raise ValueError(f"implementation units differ: {key}")
        if len({item["repetitions"] for item in items}) != 1:
            raise ValueError(f"implementation repetitions differ: {key}")
        if len({item["checksum"] for item in items}) != 1:
            raise ValueError(f"implementation checksums differ: {key}")
    with open(args.output, "w", newline="") as file:
        writer = csv.DictWriter(file, fieldnames=list(rows[0]), lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)
    print(f"suites={len(suites)} rows={len(rows)}")


if __name__ == "__main__":
    main()

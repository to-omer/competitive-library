#!/usr/bin/env python3
import argparse
import csv
import re


HEADER = re.compile(r"^benchmark=competitive_simd_methods_v[234] suite=(\d+)\b")


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
    self_test_ok = False
    with open(args.input) as file:
        for source_line in file:
            line = source_line.strip()
            match = HEADER.match(line)
            if match:
                current_suite = int(match.group(1))
                suites.append(current_suite)
                continue
            if line == "self_test=ok":
                self_test_ok = True
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
            rows.append(
                {
                    "environment": args.environment,
                    "suite": current_suite,
                    "case": item["case"],
                    "implementation": item["impl"],
                    "units": item["units"],
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
    if 0 in suites and not self_test_ok:
        raise ValueError("suite 0 did not report self_test=ok")
    keys = [(row["suite"], row["case"], row["implementation"]) for row in rows]
    if len(keys) != len(set(keys)):
        raise ValueError("duplicate benchmark rows")
    with open(args.output, "w", newline="") as file:
        writer = csv.DictWriter(file, fieldnames=list(rows[0]), lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)
    print(f"suites={len(suites)} rows={len(rows)}")


if __name__ == "__main__":
    main()

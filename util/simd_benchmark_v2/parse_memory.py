#!/usr/bin/env python3
import argparse
import csv


SUITES = {
    "benchmark=competitive_simd_memory_v2": 70,
    "benchmark=competitive_simd_memory_wavelet_existing_v2": 71,
    "benchmark=competitive_simd_memory_radix_widths_v2": 72,
}


def fields(line):
    return dict(part.split("=", 1) for part in line.split() if "=" in part)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--environment", required=True)
    parser.add_argument("--input", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    rows = []
    suites = []
    current_suite = None
    self_tests = set()
    with open(args.input) as file:
        for source_line in file:
            line = source_line.strip()
            if line.startswith("benchmark="):
                prefix = line.split(maxsplit=1)[0]
                if prefix not in SUITES:
                    raise ValueError(f"unexpected header: {line}")
                current_suite = SUITES[prefix]
                suites.append(current_suite)
                continue
            if line == "self_test=ok":
                if current_suite is None:
                    raise ValueError("self-test before header")
                self_tests.add(current_suite)
                continue
            lowered = line.lower()
            if "panicked at" in lowered or "assertion failed" in lowered:
                raise ValueError(f"failed run: {line}")
            if line.startswith("memory "):
                item = fields(line)
                rows.append(
                    {
                        "environment": args.environment,
                        "suite": current_suite,
                        "metric": "allocation",
                        "name": item["name"],
                        "live_bytes": item["live_bytes"],
                        "peak_bytes": item["peak_bytes"],
                        "object_bytes": "",
                    }
                )
            elif line.startswith("object_size "):
                item = fields(line)
                for suffix in ["", "2", "3"]:
                    name = item.get(f"name{suffix}")
                    size = item.get(f"bytes{suffix}")
                    if name is not None:
                        if size is None:
                            raise ValueError(f"missing size: {line}")
                        rows.append(
                            {
                                "environment": args.environment,
                                "suite": current_suite,
                                "metric": "object_size",
                                "name": name,
                                "live_bytes": "",
                                "peak_bytes": "",
                                "object_bytes": size,
                            }
                        )

    if suites != [70, 71, 72]:
        raise ValueError(f"suite order mismatch: {suites}")
    if self_tests != {70, 71, 72}:
        raise ValueError(f"missing self-test: {self_tests}")
    keys = [(row["suite"], row["metric"], row["name"]) for row in rows]
    if len(keys) != len(set(keys)):
        raise ValueError("duplicate memory rows")
    with open(args.output, "w", newline="") as file:
        writer = csv.DictWriter(file, fieldnames=list(rows[0]), lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)
    print(f"suites={len(suites)} rows={len(rows)}")


if __name__ == "__main__":
    main()

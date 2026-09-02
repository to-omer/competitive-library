#!/usr/bin/env python3
import argparse
import csv


SUITES = {
    "benchmark=competitive_simd_memory_v6": 70,
    "benchmark=competitive_simd_memory_wavelet_existing_v6": 71,
    "benchmark=competitive_simd_memory_radix_widths_v6": 72,
}
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
    args = parser.parse_args()

    rows = []
    suites = []
    current_suite = None
    self_tests = set()
    build_profile_suites = set()
    header_arch = None
    with open(args.input) as file:
        for source_line in file:
            line = source_line.strip()
            if line.startswith("benchmark="):
                prefix = line.split(maxsplit=1)[0]
                if prefix not in SUITES:
                    raise ValueError(f"unexpected header: {line}")
                current_suite = SUITES[prefix]
                suites.append(current_suite)
                item = fields(line)
                expected = {"benchmark", "n", "arch", "pointer_bits"}
                if current_suite == 72:
                    expected.add("signed_layout")
                if set(item) != expected:
                    raise ValueError(
                        f"unexpected header fields: {set(item) ^ expected}: {line}"
                    )
                expected_n = "32891" if current_suite == 71 else "1048699"
                if item["n"] != expected_n or item["pointer_bits"] != "64":
                    raise ValueError(f"unexpected header constants: {line}")
                if current_suite == 72 and item["signed_layout"] != (
                    "identical_after_sign_bit_encoding"
                ):
                    raise ValueError(f"unexpected signed layout: {line}")
                if header_arch is None:
                    header_arch = item["arch"]
                elif header_arch != item["arch"]:
                    raise ValueError(
                        f"header architecture changed: {header_arch} != {item['arch']}"
                    )
                continue
            if line == BUILD_PROFILE:
                if current_suite is None:
                    raise ValueError("build profile before header")
                if current_suite in build_profile_suites:
                    raise ValueError(f"duplicate build profile for suite {current_suite}")
                build_profile_suites.add(current_suite)
                continue
            if line == "self_test=ok":
                if current_suite is None:
                    raise ValueError("self-test before header")
                if current_suite in self_tests:
                    raise ValueError(f"duplicate self-test for suite {current_suite}")
                self_tests.add(current_suite)
                continue
            lowered = line.lower()
            if "panicked at" in lowered or "assertion failed" in lowered:
                raise ValueError(f"failed run: {line}")
            if line.startswith("memory "):
                if current_suite is None:
                    raise ValueError("memory row before header")
                item = fields(line)
                if set(item) != {"name", "live_bytes", "peak_bytes"}:
                    raise ValueError(f"unexpected memory fields: {line}")
                live = int(item["live_bytes"])
                peak = int(item["peak_bytes"])
                if live < 0 or peak < live:
                    raise ValueError(f"invalid allocation values: {line}")
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
                if current_suite is None:
                    raise ValueError("object-size row before header")
                item = fields(line)
                allowed = {
                    "name",
                    "bytes",
                    "name2",
                    "bytes2",
                    "name3",
                    "bytes3",
                }
                if not item or set(item) - allowed:
                    raise ValueError(f"unexpected object-size fields: {line}")
                for suffix in ["", "2", "3"]:
                    name = item.get(f"name{suffix}")
                    size = item.get(f"bytes{suffix}")
                    if (name is None) != (size is None):
                        raise ValueError(f"unpaired object-size fields: {line}")
                    if name is not None:
                        if int(size) < 0:
                            raise ValueError(f"negative object size: {line}")
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
    if build_profile_suites != {70, 71, 72}:
        raise ValueError(f"missing build profile: {build_profile_suites}")
    keys = [(row["suite"], row["metric"], row["name"]) for row in rows]
    if len(keys) != len(set(keys)):
        raise ValueError("duplicate memory rows")
    row_suites = {row["suite"] for row in rows}
    if row_suites != {70, 71, 72}:
        raise ValueError(f"row suites mismatch: {sorted(row_suites)}")
    with open(args.output, "w", newline="") as file:
        writer = csv.DictWriter(file, fieldnames=list(rows[0]), lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)
    print(f"suites={len(suites)} rows={len(rows)}")


if __name__ == "__main__":
    main()

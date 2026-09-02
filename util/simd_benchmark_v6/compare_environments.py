#!/usr/bin/env python3
import argparse
import csv
import itertools
import math
import statistics


def load(spec):
    name, path = spec.split("=", 1)
    with open(path, newline="") as file:
        rows = list(csv.DictReader(file))
    result = {}
    for row in rows:
        key = (int(row["suite"]), row["case"], row["implementation"])
        if key in result:
            raise ValueError(f"duplicate row in {name}: {key}")
        raw = tuple(int(value) for value in row["raw_ns"].split(","))
        if len(raw) != 5:
            raise ValueError(f"expected five rounds in {name}: {key}: {raw}")
        median = int(row["median_ns"])
        if median != int(statistics.median(raw)):
            raise ValueError(f"median mismatch in {name}: {key}: {median}: {raw}")
        if median <= 0:
            raise ValueError(f"non-positive sample in {name}: {key}: {raw}")
        units = int(row["units"])
        repetitions = int(row["repetitions"])
        base_units = int(row["base_units"])
        if repetitions <= 0 or base_units <= 0 or units != repetitions * base_units:
            raise ValueError(
                f"invalid calibrated units in {name}: {key}: "
                f"units={units} base={base_units} repetitions={repetitions}"
            )
        result[key] = {
            "units": units,
            "base_units": base_units,
            "repetitions": repetitions,
            "median": median,
            "per_unit": float(row["ns_per_unit"]),
            "raw": raw,
            "spread": (max(raw) - min(raw)) / statistics.median(raw),
            "checksum": row["checksum"],
        }
    return name, result


def geometric_mean(values):
    return math.exp(statistics.fmean(math.log(value) for value in values))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--env", action="append", required=True, metavar="NAME=CSV")
    parser.add_argument("--absolute-out", required=True)
    parser.add_argument("--pair-out", required=True)
    parser.add_argument("--coverage-out", required=True)
    parser.add_argument("--summary-out", required=True)
    parser.add_argument("--alerts-out", required=True)
    args = parser.parse_args()

    environments = dict(load(spec) for spec in args.env)
    names = list(environments)
    if len(names) != len(args.env):
        raise ValueError("environment names must be unique")
    key_sets = [set(rows) for rows in environments.values()]
    common = set.intersection(*key_sets)
    union = set.union(*key_sets)
    for key in union:
        base_units = {
            environments[name][key]["base_units"]
            for name in names
            if key in environments[name]
        }
        if len(base_units) != 1:
            raise ValueError(f"base unit mismatch: {key}: {base_units}")
        checksums = {
            environments[name][key]["checksum"]
            for name in names
            if key in environments[name]
        }
        if len(checksums) != 1:
            raise ValueError(f"checksum mismatch: {key}: {checksums}")

    for name, rows in environments.items():
        by_case = {}
        for (suite, case, _), item in rows.items():
            by_case.setdefault((suite, case), []).append(item)
        for key, items in by_case.items():
            units = {item["units"] for item in items}
            if len(units) != 1:
                raise ValueError(f"implementation unit mismatch in {name}: {key}: {units}")
            repetitions = {item["repetitions"] for item in items}
            if len(repetitions) != 1:
                raise ValueError(
                    f"implementation repetition mismatch in {name}: {key}: {repetitions}"
                )
            checksums = {item["checksum"] for item in items}
            if len(checksums) != 1:
                raise ValueError(
                    f"implementation checksum mismatch in {name}: {key}: {checksums}"
                )

    with open(args.coverage_out, "w", newline="") as file:
        fields = ["suite", "case", "implementation", *names]
        writer = csv.DictWriter(file, fieldnames=fields)
        writer.writeheader()
        for key in sorted(union):
            row = dict(zip(("suite", "case", "implementation"), key))
            for name in names:
                row[name] = key in environments[name]
            writer.writerow(row)

    with open(args.absolute_out, "w", newline="") as file:
        fields = ["suite", "case", "implementation"]
        for name in names:
            fields += [
                f"{name}_ns_per_unit",
                f"{name}_spread",
                f"{name}_repetitions",
                f"{name}_effective_units",
            ]
        for left, right in itertools.combinations(names, 2):
            fields += [f"{right}_over_{left}"]
        writer = csv.DictWriter(file, fieldnames=fields)
        writer.writeheader()
        for key in sorted(union):
            row = dict(zip(("suite", "case", "implementation"), key))
            for name in names:
                if key in environments[name]:
                    item = environments[name][key]
                    row[f"{name}_ns_per_unit"] = f'{item["per_unit"]:.9f}'
                    row[f"{name}_spread"] = f'{item["spread"]:.9f}'
                    row[f"{name}_repetitions"] = item["repetitions"]
                    row[f"{name}_effective_units"] = item["units"]
            for left, right in itertools.combinations(names, 2):
                if key in environments[left] and key in environments[right]:
                    ratio = (
                        environments[right][key]["per_unit"]
                        / environments[left][key]["per_unit"]
                    )
                    row[f"{right}_over_{left}"] = f"{ratio:.9f}"
            writer.writerow(row)

    cases = {}
    for suite, case, implementation in union:
        cases.setdefault((suite, case), []).append(implementation)

    pair_rows = []
    for (suite, case), implementations in sorted(cases.items()):
        for left, right in itertools.combinations(sorted(implementations), 2):
            row = {
                "suite": suite,
                "case": case,
                "left": left,
                "right": right,
            }
            winners_by_environment = {}
            ratios_by_environment = {}
            for name in names:
                left_key = (suite, case, left)
                right_key = (suite, case, right)
                if left_key not in environments[name] or right_key not in environments[name]:
                    row[f"{name}_right_over_left"] = ""
                    continue
                left_value = environments[name][left_key]["per_unit"]
                right_value = environments[name][right_key]["per_unit"]
                paired_ratios = tuple(
                    right / left
                    for left, right in zip(
                        environments[name][left_key]["raw"],
                        environments[name][right_key]["raw"],
                    )
                )
                ratio = statistics.median(paired_ratios)
                ratios_by_environment[name] = (ratio, paired_ratios)
                row[f"{name}_right_over_left"] = f"{ratio:.9f}"
                row[f"{name}_paired_spread"] = (
                    f"{(max(paired_ratios) - min(paired_ratios)) / ratio:.9f}"
                )
                winners_by_environment[name] = (
                    right if ratio < 1 else left if ratio > 1 else "tie"
                )
                row[f"{name}_round_winner_flip"] = (
                    min(paired_ratios) < 1 < max(paired_ratios)
                )
            row["winner_flip"] = (
                len(winners_by_environment) >= 2
                and len(set(winners_by_environment.values())) != 1
            )
            for first, second in itertools.combinations(names, 2):
                first_ratio = row[f"{first}_right_over_left"]
                second_ratio = row[f"{second}_right_over_left"]
                if first_ratio and second_ratio:
                    row[f"ratio_shift_{second}_over_{first}"] = (
                        f"{float(second_ratio) / float(first_ratio):.9f}"
                    )
                    first_pair = ratios_by_environment[first]
                    second_pair = ratios_by_environment[second]
                    shift = second_pair[0] / first_pair[0]
                    stable = all(
                        (max(pair) - min(pair)) / median <= 0.10
                        and not (min(pair) < 1 < max(pair))
                        for median, pair in (first_pair, second_pair)
                    )
                    margins = (
                        max(first_pair[0], 1 / first_pair[0]),
                        max(second_pair[0], 1 / second_pair[0]),
                    )
                    winner_flip = (
                        winners_by_environment[first]
                        != winners_by_environment[second]
                    )
                    row[f"winner_flip_{second}_vs_{first}"] = winner_flip
                    row[f"stable_{second}_vs_{first}"] = stable
                    row[f"meaningful_shift_{second}_vs_{first}"] = (
                        stable and max(shift, 1 / shift) >= 1.10
                    )
                    row[f"robust_winner_flip_{second}_vs_{first}"] = (
                        stable
                        and winner_flip
                        and min(margins) >= 1.03
                    )
            pair_rows.append(row)

    pair_fields = ["suite", "case", "left", "right"]
    pair_fields += [f"{name}_right_over_left" for name in names]
    pair_fields += [f"{name}_paired_spread" for name in names]
    pair_fields += [f"{name}_round_winner_flip" for name in names]
    pair_fields += ["winner_flip"]
    pair_fields += [
        f"ratio_shift_{second}_over_{first}"
        for first, second in itertools.combinations(names, 2)
    ]
    pair_fields += [
        field
        for first, second in itertools.combinations(names, 2)
        for field in (
            f"winner_flip_{second}_vs_{first}",
            f"stable_{second}_vs_{first}",
            f"meaningful_shift_{second}_vs_{first}",
            f"robust_winner_flip_{second}_vs_{first}",
        )
    ]
    with open(args.pair_out, "w", newline="") as file:
        writer = csv.DictWriter(file, fieldnames=pair_fields)
        writer.writeheader()
        writer.writerows(pair_rows)

    alert_fields = ["suite", "case", "left", "right"]
    for first, second in itertools.combinations(names, 2):
        alert_fields += [
            f"{first}_right_over_left",
            f"{second}_right_over_left",
            f"ratio_shift_{second}_over_{first}",
            f"winner_flip_{second}_vs_{first}",
            f"stable_{second}_vs_{first}",
            f"meaningful_shift_{second}_vs_{first}",
            f"robust_winner_flip_{second}_vs_{first}",
        ]
    with open(args.alerts_out, "w", newline="") as file:
        writer = csv.DictWriter(file, fieldnames=alert_fields, extrasaction="ignore")
        writer.writeheader()
        writer.writerows(
            row
            for row in pair_rows
            if any(
                row.get(f"meaningful_shift_{second}_vs_{first}")
                or row.get(f"robust_winner_flip_{second}_vs_{first}")
                for first, second in itertools.combinations(names, 2)
            )
        )

    with open(args.summary_out, "w") as file:
        print(f"environments={','.join(names)}", file=file)
        print(f"union_implementation_rows={len(union)}", file=file)
        print(f"common_implementation_rows={len(common)}", file=file)
        print(f"common_pair_rows={len(pair_rows)}", file=file)
        print(f"winner_flips={sum(row['winner_flip'] for row in pair_rows)}", file=file)
        print(
            "round_winner_flips="
            + str(
                sum(
                    row.get(f"{name}_round_winner_flip", False)
                    for row in pair_rows
                    for name in names
                )
            ),
            file=file,
        )
        for name in names:
            print(f"rows_{name}={len(environments[name])}", file=file)
            print(f"missing_{name}={len(union - set(environments[name]))}", file=file)
        for left, right in itertools.combinations(names, 2):
            print(
                f"meaningful_shifts_{right}_vs_{left}="
                + str(
                    sum(
                        row.get(f"meaningful_shift_{right}_vs_{left}", False)
                        for row in pair_rows
                    )
                ),
                file=file,
            )
            print(
                f"robust_winner_flips_{right}_vs_{left}="
                + str(
                    sum(
                        row.get(f"robust_winner_flip_{right}_vs_{left}", False)
                        for row in pair_rows
                    )
                ),
                file=file,
            )
            absolute = [
                environments[right][key]["per_unit"] / environments[left][key]["per_unit"]
                for key in common
            ]
            relative = [
                float(row[f"ratio_shift_{right}_over_{left}"])
                for row in pair_rows
                if row.get(f"ratio_shift_{right}_over_{left}")
            ]
            print(
                f"absolute_{right}_over_{left}_geomean={geometric_mean(absolute):.9f}",
                file=file,
            )
            if relative:
                print(
                    f"relative_ratio_shift_{right}_over_{left}_geomean="
                    f"{geometric_mean(relative):.9f}",
                    file=file,
                )
                print(
                    f"relative_ratio_shift_{right}_over_{left}_median_abs_log2="
                    f"{statistics.median(abs(math.log2(value)) for value in relative):.9f}",
                    file=file,
                )


if __name__ == "__main__":
    main()

from argparse import ArgumentParser
from collections import defaultdict
from html import escape
from json import JSONDecoder, loads
from pathlib import Path

from kani_matrix import harness_names


def cell(text):
    return (
        escape(" ".join(text.split()), quote=False)
        .replace("|", "&#124;")
        .replace("[", "&#91;")
        .replace("]", "&#93;")
    )


def module_heading(module, links):
    parts = module.split("::") if module else []
    # Proof-only modules are absent from ordinary rustdoc; link their owning module.
    end = parts.index("verification") if "verification" in parts else len(parts)
    target = "::".join(parts[:end])
    path = "".join(f"{part}/" for part in parts[:end])
    suffix = "::".join(parts[end:])
    reference = f"crate::{target}" if target else "crate"
    links[reference] = f"{path}index.html"
    return f"### [{target or '(crate root)'}][{reference}]" + (
        f"::{suffix}" if suffix else ""
    )


def verus_summary(data, links):
    result = data["verification-results"]
    if not result["is-verifying-entire-crate"]:
        raise ValueError("Verus did not verify the entire selected crate")
    if result["verified"] + result["errors"] == 0:
        raise ValueError("Verus reported no verification checks")
    lines = [
        "## Verus",
        "",
        f"**{'PASS' if result['success'] else 'FAIL'}** — "
        f"{result['verified']} checks verified, {result['errors']} errors.",
    ]
    modules = data["times-ms"]["smt"]["smt-run-module-times"]
    for module in sorted(modules, key=lambda m: m["module"]):
        lines += [
            "",
            module_heading(module["module"], links),
            "",
            "| Function | Properties checked | Result |",
            "| --- | --- | --- |",
        ]
        for function in sorted(
            module["function-breakdown"], key=lambda f: f["function"]
        ):
            name = function["function"].split(f"::{module['module']}::", 1)[-1]
            details = data["func-details"][function["function"]]
            failed = set(details["failed_proof_notes"])
            notes = set(details["obligation_proof_notes"]) | failed
            properties = (
                "<br>".join(
                    cell(note + (" [FAIL]" if note in failed else ""))
                    for note in sorted(notes)
                )
                or "No labeled property; see source"
            )
            lines.append(
                f"| `{name}` | {properties} | "
                f"{'PASS' if function['success'] else 'FAIL'}"
                f"<br>SMT: {function['time-micros'] / 1000:.3f} ms |"
            )
    return lines


def kani_summary(data, links, expected):
    results = []
    successful = failed = 0
    for part in data:
        summary = part["verification_results"]["summary"]
        items = part["verification_results"]["results"]
        if (
            summary["status"] != "completed"
            or not summary["total_harnesses"]
            or summary["executed"] != summary["total_harnesses"]
            or len(items) != summary["executed"]
        ):
            raise ValueError("Kani verification is incomplete or contains no harnesses")
        results.extend(items)
        successful += summary["successful"]
        failed += summary["failed"]
    actual = {harness["harness_id"] for harness in results}
    if len(actual) != len(results):
        raise ValueError("Kani results contain duplicate harnesses")
    if expected is not None and actual != expected:
        raise ValueError(
            f"Kani harness mismatch: missing {sorted(expected - actual)}, "
            f"unexpected {sorted(actual - expected)}"
        )
    lines = [
        "## Kani",
        "",
        f"{successful} passed, {failed} failed.",
    ]
    groups = defaultdict(list)
    for harness in results:
        module, _, name = harness["harness_id"].rpartition("::")
        groups[module].append((name, harness))
    for module, harnesses in sorted(groups.items()):
        lines += [
            "",
            module_heading(module, links),
            "",
            "| Harness | Harness assertions | Result |",
            "| --- | --- | --- |",
        ]
        for name, harness in sorted(harnesses):
            covers = [
                check for check in harness["checks"] if check["category"] == "cover"
            ]
            reached = sum(check["status"] == "Satisfied" for check in covers)
            cover = f"<br>Cover: {reached}/{len(covers)} reached" if covers else ""
            assertions = {
                (check["description"], check["status"])
                for check in harness["checks"]
                if check["category"] == "assertion"
                and check["function"] == harness["harness_id"]
            }
            properties = (
                "<br>".join(
                    cell(f"[{status}] {text}") for text, status in sorted(assertions)
                )
                or "No harness assertions"
            )
            lines.append(
                f"| `{name}` | {properties} | {harness['status']}"
                f"<br>{harness['duration_ms'] / 1000:.3f} s{cover} |"
            )
    return lines


def main():
    parser = ArgumentParser()
    parser.add_argument("tool", choices=["verus", "kani"])
    parser.add_argument("result", type=Path, nargs="+")
    parser.add_argument("--harnesses", type=Path, nargs="+", default=[])
    parser.add_argument(
        "--doc", type=Path, help="write Markdown with local rustdoc links"
    )
    args = parser.parse_args()
    links = {}
    try:
        if args.tool == "verus":
            if len(args.result) != 1 or args.harnesses:
                raise ValueError("Verus requires one result and no harness list")
            text = args.result[0].read_text()
            # cargo-make and dependency verification surround the root crate's JSON with logs.
            data, _ = JSONDecoder().raw_decode(text[text.index("{") :])
            lines = verus_summary(data, links)
        else:
            expected = None
            for path in args.harnesses:
                names = harness_names(loads(path.read_text()))
                if expected is not None and names != expected:
                    raise ValueError("Kani harness lists differ between jobs")
                expected = names
            lines = kani_summary(
                [loads(path.read_text()) for path in args.result], links, expected
            )
    except (OSError, ValueError, KeyError) as error:
        print(
            f"## Formal verification ({args.tool})\n\n"
            "Report unavailable: no complete verification results. See the Verify step log."
        )
        raise SystemExit(str(error)) from error
    markdown = "\n".join(lines) + "\n\n"
    if args.doc:
        args.doc.write_text(
            markdown
            + "".join(f"[{ref}]: ../{path}\n" for ref, path in links.items())
            + "\n"
        )
    print(markdown, end="")
    for reference, path in links.items():
        print(
            f"[{reference}]: "
            f"https://to-omer.github.io/competitive-library/competitive/{path}"
        )
    print()


if __name__ == "__main__":
    main()

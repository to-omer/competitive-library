from argparse import ArgumentParser
from hashlib import sha256
from json import loads
from pathlib import Path
from subprocess import run


def harness_names(data):
    return {
        name
        for kind in ("standard-harnesses", "contract-harnesses")
        for names in data[kind].values()
        for name in names
    }


def main():
    parser = ArgumentParser()
    parser.add_argument("nth", type=int)
    parser.add_argument("size", type=int)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    if not 0 <= args.nth < args.size:
        parser.error("expected 0 <= nth < size")
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    run(
        [
            "cargo",
            "kani",
            "-p",
            "competitive",
            "--manifest-path",
            str(Path("Cargo.toml").resolve()),
            "list",
            "--format",
            "json",
        ],
        cwd=output,
        check=True,
    )
    names = harness_names(loads((output / "kani-list.json").read_text()))
    selected = [
        name
        for name in sorted(names)
        if int(sha256(name.encode()).hexdigest(), 16) % args.size == args.nth
    ]
    if not selected:
        parser.error("no harnesses assigned to this job")
    print(f"Verifying {len(selected)}/{len(names)} Kani harnesses", flush=True)
    run(
        [
            "makers",
            "formal-kani",
            "--exact",
            "-Z",
            "unstable-options",
            "--export-json",
            str(output / "formal.json"),
            *[arg for name in selected for arg in ("--harness", name)],
        ],
        check=True,
    )


if __name__ == "__main__":
    main()

from argparse import ArgumentParser
from pathlib import Path
from shutil import move
from subprocess import PIPE, run

verify_packages = ["aizu_online_judge", "library_checker"]


def cargo_verify(package: str, name: str):
    run(
        [
            "cargo",
            "test",
            "--package",
            package,
            "--release",
            name,
            "--",
            "--ignored",
            "--exact",
            "--nocapture",
        ]
    )


def verify_list():
    for package in verify_packages:
        command = [
            "cargo",
            "test",
            "--package",
            package,
            "--quiet",
            "--release",
            "--",
            "--list",
            "--ignored",
        ]
        res = run(command, stdout=PIPE)
        for s in res.stdout.splitlines():
            yield (package, s.split()[0][:-1].decode("utf-8"))


def arrange_artifacts():
    res = run(["git", "ls-files", "-o", "--exclude-standard", "crates"], stdout=PIPE)
    artifact = Path("artifact")
    artifact.mkdir(exist_ok=True)
    for s in res.stdout.split():
        target = artifact / Path(s.decode("utf-8"))
        target.parent.mkdir(parents=True, exist_ok=True)
        move(s.decode("utf-8"), target)


def main():
    parser = ArgumentParser()
    parser.add_argument("-n", "--name", type=str)
    parser.add_argument("--arrange", action="store_true")
    args = parser.parse_args()
    for package, name in verify_list():
        if args.name and name.find(args.name) < 0:
            continue
        cargo_verify(package, name)

    if args.arrange:
        arrange_artifacts()


if __name__ == "__main__":
    main()

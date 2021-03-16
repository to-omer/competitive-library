from argparse import ArgumentParser
from itertools import islice
from pathlib import Path
from shutil import move
from subprocess import run

SIZE = 16

verify_packages = ['aizu_online_judge', 'library_checker']

def cargo_verify(package: str, name: str):
    res = run(['cargo', 'test', '--package', package, '--release', name, '--', '--ignored', '--exact', '--nocapture'])
    if res.returncode:
        print(f'::warning::verify failed `{name}`')

def verify_list():
    for package in verify_packages:
        command = ['cargo', 'test', '--package', package, '--quiet', '--release', '--', '--list', '--ignored']
        res = run(command, capture_output=True)
        for s in res.stdout.splitlines():
            yield (package, s.split()[0][:-1].decode('utf-8'))

def arrange_artifacts():
    res = run(['git', 'ls-files', '-o', '--exclude-standard', 'crates'], capture_output=True)
    artifact = Path('artifact')
    artifact.mkdir(exist_ok=True)
    for s in res.stdout.split():
        target = artifact / Path(s.decode('utf-8'))
        target.parent.mkdir(parents=True, exist_ok=True)
        move(s.decode('utf-8'), target)


def main():
    parser = ArgumentParser()
    parser.add_argument('nth', type=int, nargs='?')
    args = parser.parse_args()
    for package, name in islice(verify_list(), args.nth, None, SIZE):
        cargo_verify(package, name)

    arrange_artifacts()

if __name__ == '__main__':
    main()

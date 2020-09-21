from argparse import ArgumentParser
from itertools import islice
from pathlib import Path
from shutil import move
from subprocess import Popen, PIPE, call

SIZE = 16

verify_packages = ['aizu_online_judge', 'library_checker']

def cargo_verify(package: str, name: str):
    return call(['cargo', 'test', '--package', package, '--release', name, '--', '--ignored', '--exact', '--nocapture'])

def verify_list():
    for package in verify_packages:
        command = ['cargo', 'test', '--package', package, '--quiet', '--release', '--', '--list', '--ignored']
        with Popen(command, stdout=PIPE) as p:
            for s in p.stdout.readlines():
                yield (package, s.split()[0][:-1].decode('utf-8'))

def arrange_artifacts():
    with Popen(['git', 'ls-files', '-o', '--exclude-standard', 'crates'], stdout=PIPE) as p:
        artifact = Path('artifact')
        artifact.mkdir(exist_ok=True)
        for s in p.stdout.read().split():
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

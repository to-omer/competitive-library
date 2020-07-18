from argparse import ArgumentParser
from subprocess import Popen, PIPE, call
from pathlib import Path
from shutil import move

SIZE = 16

def cargo_verify(name: str):
    return ['cargo', 'test', '--lib', '--release', name, '--', '--ignored']

def main():
    parser = ArgumentParser()
    parser.add_argument('nth', type=int, nargs='?')
    args = parser.parse_args()
    with Popen(['cargo', 'test', '--lib', '--quiet', '--release', '--', '--list', '--ignored'], stdout=PIPE) as p:
        for s in p.stdout.readlines()[args.nth::SIZE]:
            call(cargo_verify(s.split()[0][:-1].decode('utf-8')))

    with Popen(['git', 'ls-files', '-o', '--exclude-standard', 'src'], stdout=PIPE) as p:
        artifact = Path('artifact')
        artifact.mkdir(exist_ok=True)
        for s in p.stdout.read().split():
            target = artifact / Path(s.decode('utf-8'))
            target.parent.mkdir(parents=True, exist_ok=True)
            move(s.decode('utf-8'), target)

if __name__ == '__main__':
    main()

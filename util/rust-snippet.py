from json import dump, loads
from subprocess import run

def main():
    out = run(['cargo', 'snippet', '-t', 'vscode', 'crates/competitive'], capture_output=True, check=True).stdout
    snippets = loads(out)
    for name in snippets:
        snippets[name]['scope'] = 'rust'
    with open('.vscode/rust.code-snippets', 'w') as f:
        dump(snippets, f)

if __name__ == '__main__':
    main()

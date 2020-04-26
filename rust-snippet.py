import json
from subprocess import Popen, PIPE

def main():
    with Popen(["cargo", "snippet", "-t", "vscode"], stdout=PIPE) as p:
        snippets = json.loads(p.stdout.read())
    for name in snippets:
        snippets[name]["scope"] = "rust"
    with open(".vscode/rust.code-snippets", "w") as f:
        json.dump(snippets, f)

if __name__ == "__main__":
    main()

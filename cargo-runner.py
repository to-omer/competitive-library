import sys
import toml
import subprocess

if __name__ == "__main__":
    bindatas = toml.load("Cargo.toml").get("bin")
    names = dict()
    if bindatas is not None:
        for bindata in bindatas:
            name = bindata.get("name")
            path = bindata.get("path")
            if path is not None:
                names[path] = name

        subprocess.check_call(["cargo", "--release", "--quiet", "--bin", names[sys.argv[1]]])

    assert False

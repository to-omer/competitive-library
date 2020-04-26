import toml
import onlinejudge_verify

if __name__ == "__main__":
    bindatas = toml.load("Cargo.toml").get("bin")
    paths = list()
    if bindatas is not None:
        for bindata in bindatas:
            path = bindata.get("path")
            if path is not None:
                paths.append(path)

    onlinejudge_verify.main(paths)

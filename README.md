# competitive-library

[![CI](https://github.com/to-omer/competitive-library/actions/workflows/ci.yml/badge.svg)](https://github.com/to-omer/competitive-library/actions/workflows/ci.yml)
[![GitHub Pages](https://img.shields.io/static/v1?label=GitHub+Pages&message=+&color=brightgreen&logo=github)](https://to-omer.github.io/competitive-library/)

competitive programming library

Install development tools with [mise](https://mise.jdx.dev/) (Rust via rustup is required):
```sh
$ mise install
```
Run the commands below in a mise-activated shell, or prefix them with `mise exec --`.

## Lint
```sh
$ makers all
```

## Verify
[here](crates/tools/verify/README.md)

## Formal verification
```sh
$ makers formal
```

## Doc
generate documentation
```sh
$ RUSTDOCFLAGS="--html-in-header util/header.html" cargo doc --no-deps --document-private-items
$ cp -r util/gh-pages/* target/doc
$ rustup toolchain install 1.89.0 --profile minimal --target x86_64-unknown-linux-gnu
$ cargo run --release -p verify_source
```

## Snippet
See [codesnip](https://github.com/to-omer/codesnip) and [generated snippets](https://to-omer.github.io/competitive-library/snippets/index.html).

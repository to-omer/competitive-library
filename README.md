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
generate documentation with verify results
```sh
$ cargo doc --no-deps --all-features --document-private-items
$ cp -r util/gh-pages/* target/doc
```

## Snippet
See [codesnip](https://github.com/to-omer/codesnip) and [generated snippets](https://to-omer.github.io/competitive-library/snippets/index.html).

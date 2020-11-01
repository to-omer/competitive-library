# competitive-library

[![Actions Status](https://github.com/to-omer/competitive-library/workflows/verify/badge.svg)](https://github.com/to-omer/competitive-library/actions)
[![GitHub Pages](https://img.shields.io/static/v1?label=GitHub+Pages&message=+&color=brightgreen&logo=github)](https://to-omer.github.io/competitive-library/)

competitive programming library

## Verify
[here](crates/tools/verify/README.md)

## Doc
generate documentation with verify results
```sh
cargo doc --no-deps --all-features
cp -r util/gh-pages/* target/doc
```

## Snippet
generate snippet for vscode
```sh
cargo install --path crates/tools/codesnip
cargo codesnip --target=crates/competitive/src/lib.rs --filter-item=test --cfg=nightly snippet --output=.vscode/rust.code-snippets
```

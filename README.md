# competitive-library

[![Actions Status](https://github.com/to-omer/competitive-library/workflows/verify/badge.svg)](https://github.com/to-omer/competitive-library/actions)
[![GitHub Pages](https://img.shields.io/static/v1?label=GitHub+Pages&message=+&color=brightgreen&logo=github)](https://to-omer.github.io/competitive-library/)

competitive programming library

## Verify
[here](crates/verify/README.md)

## Doc
generate documentation with verify results
```sh
cargo doc --no-deps --all-features
cp -r util/gh-pages/* target/doc
```

## Snippet
generate snippet for vscode
```sh
cargo run --release --bin snippet-extract -- crates/competitive/src/lib.rs crates/competitive/src/main.rs --filter-item=test --cfg=nightly
```

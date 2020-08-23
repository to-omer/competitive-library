# competitive-library

[![Actions Status](https://github.com/to-omer/competitive-library/workflows/verify/badge.svg)](https://github.com/to-omer/competitive-library/actions)
[![GitHub Pages](https://img.shields.io/static/v1?label=GitHub+Pages&message=+&color=brightgreen&logo=github)](https://to-omer.github.io/competitive-library/)

competitive programming library

## Verify
use `verify_attr::verify` attribute like `test` attribute
```rust
#[verify_attr::verify("problem-url")]
fn problem_name(reader: &mut impl Read, writer: &mut impl Write) {
    writeln!(writer, "solved!").ok();
}
```

set `eps` option and judge as floating point number
```rust
#[verify_attr::verify("problem-url", eps = "1e-6")]
fn problem_name(reader: &mut impl Read, writer: &mut impl Write) {
    writeln!(writer, "12.34").ok();
}
```

set `judge` option and judge with specified function
```rust
#[verify_attr::verify("problem-url", judge = "judge_problem_name")]
fn problem_name(reader: &mut impl Read, writer: &mut impl Write) {
    writeln!(writer, "12.34").ok();
}

fn judge_problem_name(input: &mut impl Read, output: &mut impl Read, result: &mut impl Read) -> bool {
    let mut s = String::new();
    result.read_to_string(&mut s).ok();
    s.parse::<u64>().map(|ans| ans == 0).unwrap_or_default()
}
```

verify problem and generate verify result markdown
```sh
cargo test --release verify_problem_name -- --ignored --nocapture
```

set `RUST_LOG` environmental variable as `competitive::verify=info` to output progress information

test problem on stdin and stdout
```sh
cargo test --features=verify_test test_problem_name
```

generate documentation with verify results
```sh
cargo doc --no-deps --features verify_doc
```

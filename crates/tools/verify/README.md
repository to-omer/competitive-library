# Verify

use `verify::verify` attribute like `test` attribute
```rust
#[verify::verify("problem-url")]
fn problem_name(reader: impl Read, mut writer: impl Write) {
    writeln!(writer, "solved!").ok();
}
```

set `eps` option and judge as floating point number
```rust
#[verify::verify("problem-url", eps = "1e-6")]
fn problem_name(reader: impl Read, mut writer: impl Write) {
    writeln!(writer, "12.34").ok();
}
```

set `judge` option and judge with specified function
```rust
#[verify::verify("problem-url", judge = "judge_problem_name")]
fn problem_name(reader: impl Read, mut writer: impl Write) {
    writeln!(writer, "0").ok();
}

fn judge_problem_name(input: impl Read, output: impl Read, result: impl Read) -> bool {
    let mut s = String::new();
    result.read_to_string(&mut s).ok();
    s.parse::<u64>().map(|ans| ans == 0).unwrap_or_default()
}
```

verify problem and generate verify result markdown
```sh
cargo test --release verify_problem_name -- --ignored --nocapture
```

set `RUST_LOG` environmental variable as `verify=info` to output progress information

test problem on stdin and stdout
```sh
cargo test --features=verify_test test_problem_name
```

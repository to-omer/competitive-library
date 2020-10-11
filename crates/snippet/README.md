# snippet

## cargo-snippet-extract

### Example
```rust
#[snippet::entry(inline)]
/// doc of `abc`
pub mod abc {
    /// doc of `a`
    pub fn a() {}
    /// doc of `b`
    pub fn b() {}
    #[snippet::skip]
    /// doc of `c`
    pub fn c() {}
    #[cfg(test)]
    mod tests {
        fn test_a() {
            super::a();
        }
    }
}
```

This code extracted with NAME `abc`  as below.

```rust
/// doc of `a`
pub fn a() {}
/// doc of `b`
pub fn b() {}
```

### Format
```
#[snippet::entry (AttrList,*)?]

AttrList:
    NAME | INCLUDE | INLINE

NAME:
    Lit
  | name = Lit

INCLUDE:                  specify NAME
  include (Lit,*)

INLINE:
    inline                inline `mod ... { ... }`
  | no_inline             default

Lit:
    "..."
  | "_..."                hidden
```

### Usage
```
USAGE:
    cargo snippet-extract [OPTIONS] [--] [FILE]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --cfg <SPEC>...            Configure the environment: e.g. --cfg feature="nightly"
        --filter-attr <PATH>...    Filter attributes by attributes path: e.g. --filter-attr path
        --filter-item <PATH>...    Filter items by attributes path: e.g. --filter-item test
    -o, --output <FILE>            Output file, default stdout
        --save-cache <FILE>        Save analyzed data in to file
        --use-cache <FILE>         Use cached data

ARGS:
    <FILE>...    Target file paths
```
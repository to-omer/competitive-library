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
#[snippet::entry (AttrList)?]

AttrList:
    NAME | INCLUDE | INLINE

NAME:
    Lit
  | name = Lit
  | name (Lit)

INCLUDE:          specify NAME
    include = Lit
  | include (Lit*)

INLINE:
    inline        inline `mod ... { ... }`
  | no_inline

Lit:
    "..."
  | "_..."        hidden
```

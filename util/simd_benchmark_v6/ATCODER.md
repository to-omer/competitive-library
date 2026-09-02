# AtCoder execution protocol

Select Rust 1.89.0 and submit the unchanged contents of `timing/src/main.rs` or
`memory/src/main.rs`. AtCoder supplies the matching Cargo release profile (`lto = true`) and
`--cfg atcoder`; do not add compiler flags.

- Timing source SHA-256: `14528e311f16ceeab2d94308c90f427720324de20cf1378951410a5517bf4af4`
- Memory source SHA-256: `126e99589be3ab6d432a4474246f8a818c6d2fcfe9cf92ef3fc3c6a6b01144e8`

Run each timing suite as a separate custom test using one of these stdin values:

```text
0
10
11
12
13
14
15
20
21
22
23
24
25
30
31
32
33
34
35
36
37
38
39
40
41
50
51
60
61
62
63
64
65
66
67
68
69
```

Run memory suites `70`, `71`, and `72` separately with `memory/src/main.rs`. Preserve every output
line verbatim. Report a timeout, panic, assertion failure, or checksum mismatch instead of retrying
with changed source or flags.

The source fixes the seed, five measured rounds, alternating order, calibration rule, workload, and
checksums. The parser accepts environment-specific repetition counts but requires the same base
units, case names, implementations, and checksums.

use super::{FastInput, ScanSource};
use std::str::from_utf8_unchecked;

macro_rules! impl_fast_scan_integer {
    ($($ty:ty, $read:ident, $method:ident);* $(;)?) => {$(
        #[inline]
        fn $read(&mut self) -> Option<$ty> {
            Some(unsafe { self.$method() })
        }
    )*};
}

impl ScanSource for FastInput {
    #[inline]
    fn skip_whitespace(&mut self) {
        self.skip_whitespace();
    }

    #[inline]
    fn next_token(&mut self) -> Option<&str> {
        Some(unsafe { from_utf8_unchecked(self.bytes()) })
    }

    impl_fast_scan_integer!(
        u8, read_u8, u8;
        u16, read_u16, u16;
        u32, read_u32, u32;
        u64, read_u64, u64;
        u128, read_u128, u128;
        usize, read_usize, usize;
        i8, read_i8, i8;
        i16, read_i16, i16;
        i32, read_i32, i32;
        i64, read_i64, i64;
        i128, read_i128, i128;
        isize, read_isize, isize;
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{Bytes, Chars, Scan, Scanner, SizedCollect, Usize1, Xorshift};

    #[test]
    fn test_integer_tokens() {
        let mut rng = Xorshift::default();
        macro_rules! check {
            ($($ty:ty),* $(,)?) => {$(
                let mut values = vec![0, <$ty>::MIN, <$ty>::MAX];
                values.extend((0..=255).map(|x| x as $ty));
                values.extend((0..512).map(|_| ((rng.rand64() as u128) << 64 | rng.rand64() as u128) as $ty));
                let width = <$ty>::MAX.to_string().len();
                let tokens: Vec<_> = values.iter().flat_map(|value| {
                    let token = value.to_string();
                    let digits = token.trim_start_matches('-');
                    let padded = format!("{}{}{}", if token.starts_with('-') { "-" } else { "" }, "0".repeat(width - digits.len()), digits);
                    [token, padded]
                }).collect();
                for sep in [" ", "\n", "\t", "\x0c"] {
                    let input = format!("{}                 ", tokens.join(sep));
                    let mut scanner = unsafe { FastInput::from_slice(input.as_bytes()) };
                    for &value in &values {
                        assert_eq!(<$ty as Scan>::scan(&mut scanner), Some(value));
                        assert_eq!(<$ty as Scan>::scan(&mut scanner), Some(value));
                    }
                }
            )*};
        }
        check!(
            u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
        );
    }

    #[test]
    fn test_composite_scan() {
        crate::define_enum_scan! {
            enum Query: raw {
                "ADD" => Add { v: Usize1, n: usize, xs: [(i64, u32); n] },
                "END" => End,
            }
        }
        fn check(scanner: &mut impl ScanSource) {
            crate::scan!(scanner, q: Query, bytes: Bytes, chars: Chars, pair: [i32; const 2], values: SizedCollect<u64>);
            match q {
                Query::Add { v, n, xs } => {
                    assert_eq!((v, n, xs), (1, 2, vec![(-3, 4), (5, 6)]));
                }
                Query::End => panic!("unexpected query"),
            }
            assert_eq!(bytes, "é".as_bytes());
            assert_eq!(chars, vec!['あ', 'a']);
            assert_eq!(pair, [7, -8]);
            assert_eq!(values, [9, 10]);
            assert!(matches!(scanner.scan::<Query>(), Query::End));
        }
        let input = "ADD 2 2 -3 4 5 6 é あa 7 -8 2 9 10 END                 ";
        check(&mut unsafe { FastInput::from_slice(input.as_bytes()) });
        check(&mut Scanner::new(input));
    }

    #[test]
    fn test_prepare_io() {
        use std::io::Write as _;
        let mut rng = Xorshift::default();
        for n in 0..=32 {
            let values: Vec<_> = (0..n)
                .map(|_| (rng.rand64(), rng.rand64() as i64))
                .collect();
            let mut input = n.to_string();
            if n == 0 {
                input.push('\n');
            }
            let mut expected = Vec::new();
            for &(a, b) in &values {
                input.push_str(&format!("\n{a} {b}"));
                writeln!(expected, "{a} {b}").unwrap();
            }
            input.push_str("\nEND");
            let mut output = Vec::new();
            {
                crate::prepare_io!(input.as_bytes(), &mut output);
                sc!(len: usize);
                for (a, b) in sv!([(u64, i64); iter len]) {
                    pp!(@tup (a, b));
                }
                assert_eq!(sv!(String), "END");
            }
            assert_eq!(output, expected);
        }
    }

    #[test]
    fn test_collection_scan() {
        use crate::tools::Collect;
        let mut rng = Xorshift::default();
        for mask in 0..256 {
            let rows: Vec<Vec<_>> = (0..4)
                .map(|i| (0..(mask >> (i * 2)) & 3).map(|_| rng.rand64()).collect())
                .collect();
            let mut input = String::new();
            for row in &rows {
                input.push_str(
                    &row.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(" "),
                );
                input.push('\n');
            }
            input.push_str("END\n                 ");
            for lazy in [false, true] {
                let mut scanner = unsafe { FastInput::from_slice(input.as_bytes()) };
                for row in &rows {
                    let values = if lazy {
                        crate::scan_value!(scanner, [u64; iter row.len()]).collect::<Vec<_>>()
                    } else {
                        crate::scan_value!(scanner, [u64; row.len()])
                    };
                    assert_eq!(&values, row);
                }
                assert_eq!(scanner.scan::<String>(), "END");
                for _ in 0..32 {
                    assert!(crate::scan_value!(scanner, [u8; 0]).is_empty());
                    assert!(crate::scan_value!(scanner, [u8; const 0]).is_empty());
                    assert!(scanner.scan_vec::<u8>(0).is_empty());
                    assert_eq!(crate::scan_value!(scanner, [u8; iter 0]).count(), 0);
                    assert!(scanner.mscan(Collect::<u8>::new(0)).is_empty());
                }
            }
        }
    }
}

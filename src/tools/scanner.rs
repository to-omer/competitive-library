pub fn read_stdin_all() -> String {
    use std::io::Read as _;
    let mut s = String::new();
    std::io::stdin().read_to_string(&mut s).unwrap();
    s
}
pub fn read_all(reader: &mut impl std::io::Read) -> String {
    let mut s = String::new();
    reader.read_to_string(&mut s).unwrap();
    s
}

#[cargo_snippet::snippet("scanner")]
pub trait IterScan: Sized {
    type Output;
    fn scan<'a, I: Iterator<Item = &'a str>>(iter: &mut I) -> Option<Self::Output>;
}
#[cargo_snippet::snippet("scanner")]
pub trait MarkedIterScan: Sized {
    type Output;
    fn mscan<'a, I: Iterator<Item = &'a str>>(self, iter: &mut I) -> Option<Self::Output>;
}
#[cargo_snippet::snippet("scanner")]
#[derive(Debug)]
pub struct Scanner<'a> {
    iter: std::str::SplitAsciiWhitespace<'a>,
}
#[cargo_snippet::snippet("scanner")]
impl<'a> Scanner<'a> {
    #[inline]
    pub fn new(s: &'a str) -> Self {
        let iter = s.split_ascii_whitespace();
        Self { iter }
    }
    #[inline]
    pub fn scan<T: IterScan>(&mut self) -> <T as IterScan>::Output {
        T::scan(&mut self.iter).unwrap()
    }
    #[inline]
    pub fn mscan<T: MarkedIterScan>(&mut self, marker: T) -> <T as MarkedIterScan>::Output {
        marker.mscan(&mut self.iter).unwrap()
    }
    #[inline]
    pub fn scan_vec<T: IterScan>(&mut self, size: usize) -> Vec<<T as IterScan>::Output> {
        (0..size)
            .map(|_| T::scan(&mut self.iter).unwrap())
            .collect()
    }
    #[inline]
    pub fn scan_chars(&mut self) -> Vec<char> {
        self.iter.next().unwrap().chars().collect::<Vec<char>>()
    }
}

#[cargo_snippet::snippet("scanner")]
mod scanner_impls {
    use super::*;
    macro_rules! iter_scan_impls {
        ($($t:ty)*) => {$(
            impl IterScan for $t {
                type Output = Self;
                #[inline]
                fn scan<'a, I: Iterator<Item = &'a str>>(iter: &mut I) -> Option<Self> {
                    iter.next()?.parse::<$t>().ok()
                }
            })*
        };
    }
    iter_scan_impls!(char u8 u16 u32 u64 usize i8 i16 i32 i64 isize f32 f64 u128 i128 String);

    macro_rules! iter_scan_tuple_impl {
        ($($T:ident)+) => {
            impl<$($T: IterScan),+> IterScan for ($($T,)+) {
                type Output = ($(<$T as IterScan>::Output,)+);
                #[inline]
                fn scan<'a, It: Iterator<Item = &'a str>>(iter: &mut It) -> Option<Self::Output> {
                    Some(($($T::scan(iter)?,)+))
                }
            }
        };
    }
    iter_scan_tuple_impl!(A);
    iter_scan_tuple_impl!(A B);
    iter_scan_tuple_impl!(A B C);
    iter_scan_tuple_impl!(A B C D);
    iter_scan_tuple_impl!(A B C D E);
    iter_scan_tuple_impl!(A B C D E F);
    iter_scan_tuple_impl!(A B C D E F G);
    iter_scan_tuple_impl!(A B C D E F G H);
    iter_scan_tuple_impl!(A B C D E F G H I);
    iter_scan_tuple_impl!(A B C D E F G H I J);
    iter_scan_tuple_impl!(A B C D E F G H I J K);
}

#[cargo_snippet::snippet("scanner")]
mod marker {
    use super::*;
    struct Usize1;
    impl IterScan for Usize1 {
        type Output = usize;
        #[inline]
        fn scan<'a, I: Iterator<Item = &'a str>>(iter: &mut I) -> Option<Self::Output> {
            usize::scan(iter).map(|x| x.wrapping_sub(1))
        }
    }
    struct Isize1;
    impl IterScan for Isize1 {
        type Output = isize;
        #[inline]
        fn scan<'a, I: Iterator<Item = &'a str>>(iter: &mut I) -> Option<Self::Output> {
            isize::scan(iter).map(|x| x.wrapping_sub(1))
        }
    }
}

#[macro_export]
macro_rules! scan_value {
    ($scanner:expr, ($($t:tt),*)) => {
        ($($crate::scan_value!($scanner, $t)),*)
    };
    ($scanner:expr, [$t:tt; $len:expr]) => {
        (0..$len).map(|_| $crate::scan_value!($scanner, $t)).collect::<Vec<_>>()
    };
    ($scanner:expr, { $t:tt => $f:expr }) => {
        $f($crate::scan_value!($scanner, $t))
    };
    ($scanner:expr, chars) => {
        $scanner.scan_chars()
    };
    ($scanner:expr, $t:ty) => {
        $scanner.scan::<$t>()
    };
}

#[macro_export]
macro_rules! scan {
    ($scanner:expr) => {};
    ($scanner:expr,) => {};
    ($scanner:expr, mut $var:ident: $t:tt) => {
        let mut $var = $crate::scan_value!($scanner, $t);
    };
    ($scanner:expr, $var:ident: $t:tt) => {
        let $var = $crate::scan_value!($scanner, $t);
    };
    ($scanner:expr, mut $var:ident: $t:tt, $($rest:tt)*) => {
        let mut $var = $crate::scan_value!($scanner, $t);
        scan!($scanner, $($rest)*)
    };
    ($scanner:expr, $var:ident: $t:tt, $($rest:tt)*) => {
        let $var = $crate::scan_value!($scanner, $t);
        scan!($scanner, $($rest)*)
    };

    ($scanner:expr, mut $var:ident) => {
        let mut $var = $crate::scan_value!($scanner, usize);
    };
    ($scanner:expr, $var:ident) => {
        let $var = $crate::scan_value!($scanner, usize);
    };
    ($scanner:expr, mut $var:ident, $($rest:tt)*) => {
        let mut $var = $crate::scan_value!($scanner, usize);
        scan!($scanner, $($rest)*)
    };
    ($scanner:expr, $var:ident, $($rest:tt)*) => {
        let $var = $crate::scan_value!($scanner, usize);
        scan!($scanner, $($rest)*)
    };
}

#[test]
fn test_scan() {
    let mut s = Scanner::new("1 2 3");
    scan!(s, x, y: char, z: {usize => |z| z - 1});
    assert_eq!(x, 1);
    assert_eq!(y, '2');
    assert_eq!(z, 2);

    let mut s = Scanner::new(
        r#"1 2
2 3
4 5"#,
    );
    scan!(s, edges: [({usize => |x| x - 1}, {usize => |x| x - 1}); 3]);
    assert_eq!(edges, vec![(0, 1), (1, 2), (3, 4)]);
}

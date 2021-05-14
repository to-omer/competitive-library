#[codesnip::skip]
pub fn read_stdin_all() -> String {
    use std::io::Read as _;
    let mut s = String::new();
    std::io::stdin().read_to_string(&mut s).expect("io error");
    s
}
pub fn read_stdin_all_unchecked() -> String {
    use std::io::Read as _;
    let mut buf = Vec::new();
    std::io::stdin().read_to_end(&mut buf).expect("io error");
    unsafe { String::from_utf8_unchecked(buf) }
}
pub fn read_stdin_line() -> String {
    let mut s = String::new();
    std::io::stdin().read_line(&mut s).expect("io error");
    s
}
#[codesnip::skip]
pub fn read_all(mut reader: impl std::io::Read) -> String {
    let mut s = String::new();
    reader.read_to_string(&mut s).expect("io error");
    s
}
#[codesnip::skip]
pub fn read_all_unchecked(mut reader: impl std::io::Read) -> String {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).expect("io error");
    unsafe { String::from_utf8_unchecked(buf) }
}

pub use scanner_impls::{IterScan, MarkedIterScan, Scanner};
mod scanner_impls {
    pub trait IterScan: Sized {
        type Output;
        fn scan<'a, I: Iterator<Item = &'a str>>(iter: &mut I) -> Option<Self::Output>;
    }
    pub trait MarkedIterScan: Sized {
        type Output;
        fn mscan<'a, I: Iterator<Item = &'a str>>(self, iter: &mut I) -> Option<Self::Output>;
    }
    #[derive(Clone, Debug)]
    pub struct Scanner<'a> {
        iter: std::str::SplitAsciiWhitespace<'a>,
    }
    impl<'a> Scanner<'a> {
        #[inline]
        pub fn new(s: &'a str) -> Self {
            let iter = s.split_ascii_whitespace();
            Self { iter }
        }
        #[inline]
        pub fn scan<T>(&mut self) -> <T as IterScan>::Output
        where
            T: IterScan,
        {
            <T as IterScan>::scan(&mut self.iter).expect("scan error")
        }
        #[inline]
        pub fn mscan<T>(&mut self, marker: T) -> <T as MarkedIterScan>::Output
        where
            T: MarkedIterScan,
        {
            marker.mscan(&mut self.iter).expect("scan error")
        }
        #[inline]
        pub fn scan_vec<T>(&mut self, size: usize) -> Vec<<T as IterScan>::Output>
        where
            T: IterScan,
        {
            (0..size)
                .map(|_| <T as IterScan>::scan(&mut self.iter).expect("scan error"))
                .collect()
        }
        #[inline]
        pub fn iter<'b, T>(&'b mut self) -> ScannerIter<'a, 'b, T>
        where
            T: IterScan,
        {
            ScannerIter {
                inner: self,
                _marker: std::marker::PhantomData,
            }
        }
    }

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
        ($($T:ident)*) => {
            impl<$($T: IterScan),*> IterScan for ($($T,)*) {
                type Output = ($(<$T as IterScan>::Output,)*);
                #[inline]
                fn scan<'a, It: Iterator<Item = &'a str>>(_iter: &mut It) -> Option<Self::Output> {
                    Some(($(<$T as IterScan>::scan(_iter)?,)*))
                }
            }
        };
    }
    iter_scan_tuple_impl!();
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

    pub struct ScannerIter<'a, 'b, T> {
        inner: &'b mut Scanner<'a>,
        _marker: std::marker::PhantomData<fn() -> T>,
    }
    impl<'a, 'b, T> Iterator for ScannerIter<'a, 'b, T>
    where
        T: IterScan,
    {
        type Item = <T as IterScan>::Output;
        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            <T as IterScan>::scan(&mut self.inner.iter)
        }
    }
}

pub use marker_impls::{CharWithBase, Chars, CharsWithBase, Collect, SizedCollect, Usize1};
mod marker_impls {
    use super::*;
    use std::{
        iter::{repeat_with, FromIterator},
        marker::PhantomData,
    };

    #[derive(Debug, Copy, Clone)]
    pub struct Usize1;
    impl IterScan for Usize1 {
        type Output = usize;
        #[inline]
        fn scan<'a, I: Iterator<Item = &'a str>>(iter: &mut I) -> Option<Self::Output> {
            <usize as IterScan>::scan(iter)?.checked_sub(1)
        }
    }
    #[derive(Debug, Copy, Clone)]
    pub struct CharWithBase(pub char);
    impl MarkedIterScan for CharWithBase {
        type Output = usize;
        #[inline]
        fn mscan<'a, I: Iterator<Item = &'a str>>(self, iter: &mut I) -> Option<Self::Output> {
            Some((<char as IterScan>::scan(iter)? as u8 - self.0 as u8) as usize)
        }
    }
    #[derive(Debug, Copy, Clone)]
    pub struct Chars;
    impl IterScan for Chars {
        type Output = Vec<char>;
        #[inline]
        fn scan<'a, I: Iterator<Item = &'a str>>(iter: &mut I) -> Option<Self::Output> {
            Some(iter.next()?.chars().collect())
        }
    }
    #[derive(Debug, Copy, Clone)]
    pub struct CharsWithBase(pub char);
    impl MarkedIterScan for CharsWithBase {
        type Output = Vec<usize>;
        #[inline]
        fn mscan<'a, I: Iterator<Item = &'a str>>(self, iter: &mut I) -> Option<Self::Output> {
            Some(
                iter.next()?
                    .chars()
                    .map(|c| (c as u8 - self.0 as u8) as usize)
                    .collect(),
            )
        }
    }
    #[derive(Debug, Copy, Clone)]
    pub struct Collect<T, B = Vec<<T as IterScan>::Output>>
    where
        T: IterScan,
        B: FromIterator<<T as IterScan>::Output>,
    {
        size: usize,
        _marker: PhantomData<fn() -> (T, B)>,
    }
    impl<T, B> Collect<T, B>
    where
        T: IterScan,
        B: FromIterator<<T as IterScan>::Output>,
    {
        pub fn new(size: usize) -> Self {
            Self {
                size,
                _marker: PhantomData,
            }
        }
    }
    impl<T, B> MarkedIterScan for Collect<T, B>
    where
        T: IterScan,
        B: FromIterator<<T as IterScan>::Output>,
    {
        type Output = B;
        #[inline]
        fn mscan<'a, I: Iterator<Item = &'a str>>(self, iter: &mut I) -> Option<Self::Output> {
            repeat_with(|| <T as IterScan>::scan(iter))
                .take(self.size)
                .collect()
        }
    }
    #[derive(Debug, Copy, Clone)]
    pub struct SizedCollect<T, B = Vec<<T as IterScan>::Output>>
    where
        T: IterScan,
        B: FromIterator<<T as IterScan>::Output>,
    {
        _marker: PhantomData<fn() -> (T, B)>,
    }
    impl<T, B> IterScan for SizedCollect<T, B>
    where
        T: IterScan,
        B: FromIterator<<T as IterScan>::Output>,
    {
        type Output = B;
        #[inline]
        fn scan<'a, I: Iterator<Item = &'a str>>(iter: &mut I) -> Option<Self::Output> {
            let size = usize::scan(iter)?;
            repeat_with(|| <T as IterScan>::scan(iter))
                .take(size)
                .collect()
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
    ($scanner:expr, [$t:ty; $len:expr]) => {
        $scanner.scan_vec::<$t>($len)
    };
    ($scanner:expr, [$t:ty]) => {
        $scanner.iter::<$t>()
    };
    ($scanner:expr, {$e:expr}) => {
        $scanner.mscan($e)
    };
    ($scanner:expr, $t:ty) => {
        $scanner.scan::<$t>()
    };
}

#[macro_export]
macro_rules! scan {
    ($scanner:expr) => {};
    ($scanner:expr,) => {};
    ($scanner:expr, mut $var:tt: $t:tt) => {
        let mut $var = $crate::scan_value!($scanner, $t);
    };
    ($scanner:expr, $var:tt: $t:tt) => {
        let $var = $crate::scan_value!($scanner, $t);
    };
    ($scanner:expr, mut $var:tt: $t:tt, $($rest:tt)*) => {
        let mut $var = $crate::scan_value!($scanner, $t);
        scan!($scanner, $($rest)*)
    };
    ($scanner:expr, $var:tt: $t:tt, $($rest:tt)*) => {
        let $var = $crate::scan_value!($scanner, $t);
        scan!($scanner, $($rest)*)
    };

    ($scanner:expr, mut $var:tt) => {
        let mut $var = $crate::scan_value!($scanner, usize);
    };
    ($scanner:expr, $var:tt) => {
        let $var = $crate::scan_value!($scanner, usize);
    };
    ($scanner:expr, mut $var:tt, $($rest:tt)*) => {
        let mut $var = $crate::scan_value!($scanner, usize);
        scan!($scanner, $($rest)*)
    };
    ($scanner:expr, $var:tt, $($rest:tt)*) => {
        let $var = $crate::scan_value!($scanner, usize);
        scan!($scanner, $($rest)*)
    };
}

#[test]
fn test_scan() {
    let mut s = Scanner::new("1 2 3");
    scan!(s, x, y: char, z: Usize1);
    assert_eq!(x, 1);
    assert_eq!(y, '2');
    assert_eq!(z, 2);
}

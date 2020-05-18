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

pub trait IterScan: Sized {
    fn scan<'a, I: Iterator<Item = &'a str>>(iter: &mut I) -> Option<Self>;
}
#[derive(Debug)]
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
    pub fn scan_from_str<T: std::str::FromStr>(&mut self) -> T {
        self.iter.next().unwrap().parse::<T>().ok().unwrap()
    }
    #[inline]
    pub fn scan<T: IterScan>(&mut self) -> T {
        T::scan(&mut self.iter).unwrap()
    }
    #[inline]
    pub fn scan_vec<T: IterScan>(&mut self, size: usize) -> Vec<T> {
        (0..size)
            .map(|_| T::scan(&mut self.iter).unwrap())
            .collect()
    }
    #[inline]
    pub fn scan_chars(&mut self) -> Vec<char> {
        self.iter.next().unwrap().chars().collect::<Vec<char>>()
    }
}

macro_rules! iter_scan_impls {
    ($($t:ty)*) => {$(
        impl IterScan for $t {
            #[inline]
            fn scan<'a, I: Iterator<Item = &'a str>>(iter: &mut I) -> Option<Self> {
                iter.next()?.parse::<$t>().ok()
            }
        })*
    };
}
iter_scan_impls!(char u8 u16 u32 u64 usize i8 i16 i32 i64 isize f32 f64);

macro_rules! iter_scan_tuple_impl {
    ($($T:ident)+) => {
        impl<$($T: IterScan),+> IterScan for ($($T,)+) {
            #[inline]
            fn scan<'a, It: Iterator<Item = &'a str>>(iter: &mut It) -> Option<Self> {
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

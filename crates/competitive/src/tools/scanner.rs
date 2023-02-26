use std::{
    iter::{from_fn, repeat_with, FromIterator},
    marker::PhantomData,
};

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
pub fn read_all(mut reader: impl std::io::Read) -> String {
    let mut s = String::new();
    reader.read_to_string(&mut s).expect("io error");
    s
}
pub fn read_all_unchecked(mut reader: impl std::io::Read) -> String {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).expect("io error");
    unsafe { String::from_utf8_unchecked(buf) }
}
pub fn read_stdin_line() -> String {
    let mut s = String::new();
    std::io::stdin().read_line(&mut s).expect("io error");
    s
}
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
    (@impl $($T:ident)*) => {
        impl<$($T: IterScan),*> IterScan for ($($T,)*) {
            type Output = ($(<$T as IterScan>::Output,)*);
            #[inline]
            fn scan<'a, It: Iterator<Item = &'a str>>(_iter: &mut It) -> Option<Self::Output> {
                Some(($(<$T as IterScan>::scan(_iter)?,)*))
            }
        }
    };
    (@inner $($T:ident)*,) => {
        iter_scan_tuple_impl!(@impl $($T)*);
    };
    (@inner $($T:ident)*, $U:ident $($Rest:ident)*) => {
        iter_scan_tuple_impl!(@impl $($T)*);
        iter_scan_tuple_impl!(@inner $($T)* $U, $($Rest)*);
    };
    ($($T:ident)*) => {
        iter_scan_tuple_impl!(@inner , $($T)*);
    };
}
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

/// scan a value with Scanner
///
/// - `scan_value!(scanner, ELEMENT)`
///
/// ELEMENT :=
/// - `$ty`: IterScan
/// - `@$expr`: MarkedIterScan
/// - `[ELEMENT; $expr]`: vector
/// - `[ELEMENT; const $expr]`: array
/// - `[ELEMENT]`: iterator
/// - `($(ELEMENT)*,)`: tuple
#[macro_export]
macro_rules! scan_value {
    (@repeat $scanner:expr, [$($t:tt)*] $($len:expr)?)                                    => { ::std::iter::repeat_with(|| $crate::scan_value!(@inner $scanner, [] $($t)*)) $(.take($len).collect::<Vec<_>>())? };
    (@array $scanner:expr, [$($t:tt)*] $len:expr)                                         => { $crate::array![|| $crate::scan_value!(@inner $scanner, [] $($t)*); $len] };
    (@tuple $scanner:expr, [$([$($args:tt)*])*])                                          => { ($($($args)*,)*) };
    (@$tag:ident $scanner:expr, [[$($args:tt)*]])                                         => { $($args)* };
    (@$tag:ident $scanner:expr, [$($args:tt)*] @$e:expr)                                  => { $crate::scan_value!(@$tag $scanner, [$($args)* [$scanner.mscan($e)]]) };
    (@$tag:ident $scanner:expr, [$($args:tt)*] @$e:expr, $($t:tt)*)                       => { $crate::scan_value!(@$tag $scanner, [$($args)* [$scanner.mscan($e)]] $($t)*) };
    (@$tag:ident $scanner:expr, [$($args:tt)*] ($($tuple:tt)*) $($t:tt)*)                 => { $crate::scan_value!(@$tag $scanner, [$($args)* [$crate::scan_value!(@tuple $scanner, [] $($tuple)*)]] $($t)*) };
    (@$tag:ident $scanner:expr, [$($args:tt)*] [@$e:expr; const $len:expr] $($t:tt)*)     => { $crate::scan_value!(@$tag $scanner, [$($args)* [$crate::scan_value!(@array $scanner, [@$e] $len)]] $($t)*) };
    (@$tag:ident $scanner:expr, [$($args:tt)*] [@$e:expr; $len:expr] $($t:tt)*)           => { $crate::scan_value!(@$tag $scanner, [$($args)* [$crate::scan_value!(@repeat $scanner, [@$e] $len)]] $($t)*) };
    (@$tag:ident $scanner:expr, [$($args:tt)*] [[$($tt:tt)*]; const $len:expr] $($t:tt)*) => { $crate::scan_value!(@$tag $scanner, [$($args)* [$crate::scan_value!(@array $scanner, [[$($tt)*]] $len)]] $($t)*) };
    (@$tag:ident $scanner:expr, [$($args:tt)*] [[$($tt:tt)*]; $len:expr] $($t:tt)*)       => { $crate::scan_value!(@$tag $scanner, [$($args)* [$crate::scan_value!(@repeat $scanner, [[$($tt)*]] $len)]] $($t)*) };
    (@$tag:ident $scanner:expr, [$($args:tt)*] [($($tt:tt)*); const $len:expr] $($t:tt)*) => { $crate::scan_value!(@$tag $scanner, [$($args)* [$crate::scan_value!(@array $scanner, [($($tt)*)] $len)]] $($t)*) };
    (@$tag:ident $scanner:expr, [$($args:tt)*] [($($tt:tt)*); $len:expr] $($t:tt)*)       => { $crate::scan_value!(@$tag $scanner, [$($args)* [$crate::scan_value!(@repeat $scanner, [($($tt)*)] $len)]] $($t)*) };
    (@$tag:ident $scanner:expr, [$($args:tt)*] [$ty:ty; const $len:expr] $($t:tt)*)       => { $crate::scan_value!(@$tag $scanner, [$($args)* [$crate::scan_value!(@array $scanner, [$ty] $len)]] $($t)*) };
    (@$tag:ident $scanner:expr, [$($args:tt)*] [$ty:ty; $len:expr] $($t:tt)*)             => { $crate::scan_value!(@$tag $scanner, [$($args)* [$crate::scan_value!(@repeat $scanner, [$ty] $len)]] $($t)*) };
    (@$tag:ident $scanner:expr, [$($args:tt)*] [$($tt:tt)*] $($t:tt)*)                    => { $crate::scan_value!(@$tag $scanner, [$($args)* [$crate::scan_value!(@repeat $scanner, [$($tt)*])]] $($t)*) };
    (@$tag:ident $scanner:expr, [$($args:tt)*] $ty:ty)                                    => { $crate::scan_value!(@$tag $scanner, [$($args)* [$scanner.scan::<$ty>()]]) };
    (@$tag:ident $scanner:expr, [$($args:tt)*] $ty:ty, $($t:tt)*)                         => { $crate::scan_value!(@$tag $scanner, [$($args)* [$scanner.scan::<$ty>()]] $($t)*) };
    (@$tag:ident $scanner:expr, [$($args:tt)*] , $($t:tt)*)                               => { $crate::scan_value!(@$tag $scanner, [$($args)*] $($t)*) };
    (@$tag:ident $scanner:expr, [$($args:tt)*])                                           => { ::std::compile_error!(::std::stringify!($($args)*)) };
    ($scanner:expr, $($t:tt)*)                                                            => { $crate::scan_value!(@inner $scanner, [] $($t)*) }
}

/// scan and bind values with Scanner
///
/// - `scan!(scanner, $($pat $(: ELEMENT)?),*)`
#[macro_export]
macro_rules! scan {
    (@assert $p:pat) => {};
    (@assert $($p:tt)*) => { ::std::compile_error!(::std::concat!("expected pattern, found `", ::std::stringify!($($p)*), "`")); };
    (@pat $scanner:expr, [] [])                                          => {};
    (@pat $scanner:expr, [] [] , $($t:tt)*)                              => { $crate::scan!(@pat $scanner, [] [] $($t)*) };
    (@pat $scanner:expr, [$($p:tt)*] [] $x:ident $($t:tt)*)              => { $crate::scan!(@pat $scanner, [$($p)* $x] [] $($t)*) };
    (@pat $scanner:expr, [$($p:tt)*] [] :: $($t:tt)*)                    => { $crate::scan!(@pat $scanner, [$($p)* ::] [] $($t)*) };
    (@pat $scanner:expr, [$($p:tt)*] [] & $($t:tt)*)                     => { $crate::scan!(@pat $scanner, [$($p)* &] [] $($t)*) };
    (@pat $scanner:expr, [$($p:tt)*] [] ($($x:tt)*) $($t:tt)*)           => { $crate::scan!(@pat $scanner, [$($p)* ($($x)*)] [] $($t)*) };
    (@pat $scanner:expr, [$($p:tt)*] [] [$($x:tt)*] $($t:tt)*)           => { $crate::scan!(@pat $scanner, [$($p)* [$($x)*]] [] $($t)*) };
    (@pat $scanner:expr, [$($p:tt)*] [] {$($x:tt)*} $($t:tt)*)           => { $crate::scan!(@pat $scanner, [$($p)* {$($x)*}] [] $($t)*) };
    (@pat $scanner:expr, [$($p:tt)*] [] : $($t:tt)*)                     => { $crate::scan!(@ty  $scanner, [$($p)*] [] $($t)*) };
    (@pat $scanner:expr, [$($p:tt)*] [] $($t:tt)*)                       => { $crate::scan!(@let $scanner, [$($p)*] [usize] $($t)*) };
    (@ty  $scanner:expr, [$($p:tt)*] [$($tt:tt)*] @$e:expr)              => { $crate::scan!(@let $scanner, [$($p)*] [$($tt)* @$e]) };
    (@ty  $scanner:expr, [$($p:tt)*] [$($tt:tt)*] @$e:expr, $($t:tt)*)   => { $crate::scan!(@let $scanner, [$($p)*] [$($tt)* @$e], $($t)*) };
    (@ty  $scanner:expr, [$($p:tt)*] [$($tt:tt)*] ($($x:tt)*) $($t:tt)*) => { $crate::scan!(@let $scanner, [$($p)*] [$($tt)* ($($x)*)] $($t)*) };
    (@ty  $scanner:expr, [$($p:tt)*] [$($tt:tt)*] [$($x:tt)*] $($t:tt)*) => { $crate::scan!(@let $scanner, [$($p)*] [$($tt)* [$($x)*]] $($t)*) };
    (@ty  $scanner:expr, [$($p:tt)*] [$($tt:tt)*] $ty:ty)                => { $crate::scan!(@let $scanner, [$($p)*] [$($tt)* $ty]) };
    (@ty  $scanner:expr, [$($p:tt)*] [$($tt:tt)*] $ty:ty, $($t:tt)*)     => { $crate::scan!(@let $scanner, [$($p)*] [$($tt)* $ty], $($t)*) };
    (@let $scanner:expr, [$($p:tt)*] [$($tt:tt)*] $($t:tt)*) => {
        $crate::scan!{@assert $($p)*}
        let $($p)* = $crate::scan_value!($scanner, $($tt)*);
        $crate::scan!(@pat $scanner, [] [] $($t)*)
    };
    ($scanner:expr, $($t:tt)*) => { $crate::scan!(@pat $scanner, [] [] $($t)*) }
}

#[derive(Debug, Copy, Clone)]
pub enum Usize1 {}
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
pub enum Chars {}
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
pub enum Byte1 {}
impl IterScan for Byte1 {
    type Output = u8;
    #[inline]
    fn scan<'a, I: Iterator<Item = &'a str>>(iter: &mut I) -> Option<Self::Output> {
        let bytes = iter.next()?.as_bytes();
        assert_eq!(bytes.len(), 1);
        Some(bytes[0])
    }
}
#[derive(Debug, Copy, Clone)]
pub struct ByteWithBase(pub u8);
impl MarkedIterScan for ByteWithBase {
    type Output = usize;
    #[inline]
    fn mscan<'a, I: Iterator<Item = &'a str>>(self, iter: &mut I) -> Option<Self::Output> {
        Some((<char as IterScan>::scan(iter)? as u8 - self.0) as usize)
    }
}
#[derive(Debug, Copy, Clone)]
pub enum Bytes {}
impl IterScan for Bytes {
    type Output = Vec<u8>;
    #[inline]
    fn scan<'a, I: Iterator<Item = &'a str>>(iter: &mut I) -> Option<Self::Output> {
        Some(iter.next()?.bytes().collect())
    }
}
#[derive(Debug, Copy, Clone)]
pub struct BytesWithBase(pub u8);
impl MarkedIterScan for BytesWithBase {
    type Output = Vec<usize>;
    #[inline]
    fn mscan<'a, I: Iterator<Item = &'a str>>(self, iter: &mut I) -> Option<Self::Output> {
        Some(
            iter.next()?
                .bytes()
                .map(|c| (c - self.0) as usize)
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
#[derive(Debug, Copy, Clone)]
pub struct Splitted<T, P>
where
    T: IterScan,
{
    pat: P,
    _marker: PhantomData<fn() -> T>,
}
impl<T, P> Splitted<T, P>
where
    T: IterScan,
{
    pub fn new(pat: P) -> Self {
        Self {
            pat,
            _marker: PhantomData,
        }
    }
}
impl<T> MarkedIterScan for Splitted<T, char>
where
    T: IterScan,
{
    type Output = Vec<<T as IterScan>::Output>;
    fn mscan<'a, I: Iterator<Item = &'a str>>(self, iter: &mut I) -> Option<Self::Output> {
        let mut iter = iter.next()?.split(self.pat);
        Some(from_fn(|| <T as IterScan>::scan(&mut iter)).collect())
    }
}
impl<T> MarkedIterScan for Splitted<T, &str>
where
    T: IterScan,
{
    type Output = Vec<<T as IterScan>::Output>;
    fn mscan<'a, I: Iterator<Item = &'a str>>(self, iter: &mut I) -> Option<Self::Output> {
        let mut iter = iter.next()?.split(self.pat);
        Some(from_fn(|| <T as IterScan>::scan(&mut iter)).collect())
    }
}
impl<T, F> MarkedIterScan for F
where
    F: Fn(&str) -> Option<T>,
{
    type Output = T;
    fn mscan<'a, I: Iterator<Item = &'a str>>(self, iter: &mut I) -> Option<Self::Output> {
        self(iter.next()?)
    }
}

#[test]
fn test_scan() {
    use crate::scan;
    let mut s = Scanner::new("1 2 3 a 1 2 1 1 1.1 2 3");
    scan!(s, x, y: char, z: Usize1, a: @CharWithBase('a'), b: [usize; 2], c: (usize, @CharWithBase('0')), d: @Splitted::<usize, _>::new('.'), e: [usize; const 2]);
    assert_eq!(x, 1);
    assert_eq!(y, '2');
    assert_eq!(z, 2);
    assert_eq!(a, 0);
    assert_eq!(b, vec![1, 2]);
    assert_eq!(c, (1, 1));
    assert_eq!(d, vec![1, 1]);
    assert_eq!(e, [2, 3]);
}

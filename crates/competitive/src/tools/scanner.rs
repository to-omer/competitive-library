use std::{
    iter::{FromIterator, from_fn, repeat_with},
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
pub trait Scan: Sized {
    type Output;
    fn scan<S: ScanSource>(source: &mut S) -> Option<Self::Output>;
}
pub trait MarkedScan: Sized {
    type Output;
    fn mscan<S: ScanSource>(self, source: &mut S) -> Option<Self::Output>;
}

pub trait ScanSource: Sized {
    /// Reads a token under the source's input requirements.
    /// Checked sources return `None` at EOF; unchecked sources require available input.
    fn next_token(&mut self) -> Option<&str>;
    /// Skips separators without consuming the next token.
    #[inline]
    fn skip_whitespace(&mut self) {}
    #[inline]
    fn read_u8(&mut self) -> Option<u8> {
        self.next_token()?.parse().ok()
    }
    #[inline]
    fn read_u16(&mut self) -> Option<u16> {
        self.next_token()?.parse().ok()
    }
    #[inline]
    fn read_u32(&mut self) -> Option<u32> {
        self.next_token()?.parse().ok()
    }
    #[inline]
    fn read_u64(&mut self) -> Option<u64> {
        self.next_token()?.parse().ok()
    }
    #[inline]
    fn read_u128(&mut self) -> Option<u128> {
        self.next_token()?.parse().ok()
    }
    #[inline]
    fn read_usize(&mut self) -> Option<usize> {
        self.next_token()?.parse().ok()
    }
    #[inline]
    fn read_i8(&mut self) -> Option<i8> {
        self.next_token()?.parse().ok()
    }
    #[inline]
    fn read_i16(&mut self) -> Option<i16> {
        self.next_token()?.parse().ok()
    }
    #[inline]
    fn read_i32(&mut self) -> Option<i32> {
        self.next_token()?.parse().ok()
    }
    #[inline]
    fn read_i64(&mut self) -> Option<i64> {
        self.next_token()?.parse().ok()
    }
    #[inline]
    fn read_i128(&mut self) -> Option<i128> {
        self.next_token()?.parse().ok()
    }
    #[inline]
    fn read_isize(&mut self) -> Option<isize> {
        self.next_token()?.parse().ok()
    }
    /// Panics if reading fails.
    #[inline]
    fn scan<T: Scan>(&mut self) -> T::Output {
        T::scan(self).expect("scan error")
    }
    /// Panics if reading fails.
    #[inline]
    fn mscan<T: MarkedScan>(&mut self, marker: T) -> T::Output {
        marker.mscan(self).expect("scan error")
    }
    fn scan_vec<T: Scan>(&mut self, size: usize) -> Vec<T::Output> {
        if size == 0 {
            self.skip_whitespace();
        }
        (0..size).map(|_| self.scan::<T>()).collect()
    }
    #[inline]
    fn iter<T: Scan>(&mut self) -> ScannerIter<'_, Self, T> {
        ScannerIter {
            inner: self,
            _marker: PhantomData,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Scanner<'a, I: Iterator<Item = &'a str> = std::str::SplitAsciiWhitespace<'a>> {
    iter: I,
}
impl<'a> Scanner<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            iter: s.split_ascii_whitespace(),
        }
    }
}
impl<'a, I: Iterator<Item = &'a str>> Scanner<'a, I> {
    pub fn new_from_iter(iter: I) -> Self {
        Self { iter }
    }
}
impl<'a, I: Iterator<Item = &'a str>> ScanSource for Scanner<'a, I> {
    fn next_token(&mut self) -> Option<&str> {
        self.iter.next()
    }
}

macro_rules! impl_scan {
    ($($t:ty)*) => {$(
        impl Scan for $t {
            type Output = Self;
            fn scan<I: ScanSource>(iter: &mut I) -> Option<Self> {
                iter.next_token()?.parse::<$t>().ok()
            }
        })*
    };
}
impl_scan!(char f32 f64 String);

macro_rules! impl_integer_scan {
    ($($t:ty => $read:ident),* $(,)?) => {$(
        impl Scan for $t {
            type Output = Self;
            #[inline]
            fn scan<S: ScanSource>(source: &mut S) -> Option<Self> {
                source.$read()
            }
        }
    )*};
}
impl_integer_scan!(
    u8 => read_u8, u16 => read_u16, u32 => read_u32, u64 => read_u64,
    u128 => read_u128, usize => read_usize, i8 => read_i8, i16 => read_i16,
    i32 => read_i32, i64 => read_i64, i128 => read_i128, isize => read_isize,
);

macro_rules! impl_scan_tuple {
    (@impl $($T:ident)*) => {
        impl<$($T: Scan),*> Scan for ($($T,)*) {
            type Output = ($(<$T as Scan>::Output,)*);
            fn scan<It: ScanSource>(_iter: &mut It) -> Option<Self::Output> {
                Some(($(<$T as Scan>::scan(_iter)?,)*))
            }
        }
    };
    (@inner $($T:ident)*,) => {
        impl_scan_tuple!(@impl $($T)*);
    };
    (@inner $($T:ident)*, $U:ident $($Rest:ident)*) => {
        impl_scan_tuple!(@impl $($T)*);
        impl_scan_tuple!(@inner $($T)* $U, $($Rest)*);
    };
    ($($T:ident)*) => {
        impl_scan_tuple!(@inner , $($T)*);
    };
}
impl_scan_tuple!(A B C D E F G H I J K);

pub struct ScannerIter<'a, S, T> {
    inner: &'a mut S,
    _marker: PhantomData<fn() -> T>,
}
impl<S: ScanSource, T: Scan> Iterator for ScannerIter<'_, S, T> {
    type Item = T::Output;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        T::scan(self.inner)
    }
}

/// scan a value with Scanner
///
/// - `scan_value!(scanner, ELEMENT)`
///
/// ELEMENT :=
/// - `$ty`: Scan
/// - `&str`: borrowed token; the source stays borrowed until its last use
/// - `@$expr`: MarkedScan
/// - `$ty = $expr`: MarkedScan
/// - `[ELEMENT; $expr]`: vector
/// - `[ELEMENT; const $expr]`: array
/// - `[ELEMENT]`: iterator
/// - `[ELEMENT; iter $expr]`: iterator of the specified length
/// - `($(ELEMENT)*,)`: tuple
#[macro_export]
macro_rules! scan_value {
    (@repeat $scanner:expr, [$($t:tt)*] $len:expr)                             => { { $crate::scan_value!(@iter $scanner, [$($t)*] $len).collect::<Vec<_>>() } };
    (@repeat $scanner:expr, [$($t:tt)*])                                       => { { ::std::iter::repeat_with(|| $crate::scan_value!(@inner $scanner, [] $($t)*)) } };
    (@iter $scanner:expr, [$($t:tt)*] $len:expr)                               => {{ let size = $len; if size == 0 { $scanner.skip_whitespace(); } $crate::scan_value!(@repeat $scanner, [$($t)*]).take(size) }};
    (@array $scanner:expr, [$($t:tt)*] $len:expr)                              => { { if $len == 0 { $scanner.skip_whitespace(); } $crate::array![|| $crate::scan_value!(@inner $scanner, [] $($t)*); $len] } };
    (@tuple $scanner:expr, [$([$($args:tt)*])*])                               => { ($($($args)*,)*) };
    (@sparen $scanner:expr, [] @$e:expr; $($t:tt)*)                            => { $crate::scan_value!(@sparen $scanner, [@$e] $($t)*) };
    (@sparen $scanner:expr, [] ($($tt:tt)*); $($t:tt)*)                        => { $crate::scan_value!(@sparen $scanner, [($($tt)*)] $($t)*) };
    (@sparen $scanner:expr, [] [$($tt:tt)*]; $($t:tt)*)                        => { $crate::scan_value!(@sparen $scanner, [[$($tt)*]] $($t)*) };
    (@sparen $scanner:expr, [] $ty:ty = $e:expr; $($t:tt)*)                    => { $crate::scan_value!(@sparen $scanner, [$ty = $e] $($t)*) };
    (@sparen $scanner:expr, [] $ty:ty; $($t:tt)*)                              => { $crate::scan_value!(@sparen $scanner, [$ty] $($t)*) };
    (@sparen $scanner:expr, [] $($args:tt)*)                                   => { $crate::scan_value!(@repeat $scanner, [$($args)*]) };
    (@sparen $scanner:expr, [$($args:tt)+] const $len:expr)                    => { $crate::scan_value!(@array $scanner, [$($args)+] $len) };
    (@sparen $scanner:expr, [$($args:tt)+] iter $len:expr)                     => { $crate::scan_value!(@iter $scanner, [$($args)+] $len) };
    (@sparen $scanner:expr, [$($args:tt)+] $len:expr)                          => { $crate::scan_value!(@repeat $scanner, [$($args)+] $len) };
    (@$tag:ident $scanner:expr, [[$($args:tt)*]])                              => { $($args)* };
    (@$tag:ident $scanner:expr, [$($args:tt)*] @$e:expr $(, $($t:tt)*)?)       => { $crate::scan_value!(@$tag $scanner, [$($args)* [$scanner.mscan($e)]] $(, $($t)*)?) };
    (@$tag:ident $scanner:expr, [$($args:tt)*] ($($tuple:tt)*) $($t:tt)*)      => { $crate::scan_value!(@$tag $scanner, [$($args)* [$crate::scan_value!(@tuple $scanner, [] $($tuple)*)]] $($t)*) };
    (@$tag:ident $scanner:expr, [$($args:tt)*] [$($tt:tt)*] $($t:tt)*)         => { $crate::scan_value!(@$tag $scanner, [$($args)* [$crate::scan_value!(@sparen $scanner, [] $($tt)*)]] $($t)*) };
    (@$tag:ident $scanner:expr, [$($args:tt)*] &str $(, $($t:tt)*)?)           => { $crate::scan_value!(@$tag $scanner, [$($args)* [$scanner.next_token().expect("scan error")]] $(, $($t)*)?) };
    (@$tag:ident $scanner:expr, [$($args:tt)*] $ty:ty = $e:expr $(, $($t:tt)*)?) => { $crate::scan_value!(@$tag $scanner, [$($args)* [{ let _tmp: $ty = $scanner.mscan($e); _tmp }]] $(, $($t)*)?) };
    (@$tag:ident $scanner:expr, [$($args:tt)*] $ty:ty $(, $($t:tt)*)?)         => { $crate::scan_value!(@$tag $scanner, [$($args)* [$scanner.scan::<$ty>()]] $(, $($t)*)?) };
    (@$tag:ident $scanner:expr, [$($args:tt)*] , $($t:tt)*)                    => { $crate::scan_value!(@$tag $scanner, [$($args)*] $($t)*) };
    (@$tag:ident $scanner:expr, [$($args:tt)*])                                => { ::std::compile_error!(::std::stringify!($($args)*)) };
    (src = $src:expr, $($t:tt)*)                                               => { { let mut __scanner = Scanner::new($src); $crate::scan_value!(@inner __scanner, [] $($t)*) } };
    (iter = $iter:expr, $($t:tt)*)                                             => { { let mut __scanner = Scanner::new_from_iter($iter); $crate::scan_value!(@inner __scanner, [] $($t)*) } };
    ($scanner:expr, $($t:tt)*)                                                 => { $crate::scan_value!(@inner $scanner, [] $($t)*) }
}

/// scan and bind values with Scanner
///
/// - `scan!(scanner, $($pat $(: ELEMENT)?),*)`
#[macro_export]
macro_rules! scan {
    (@assert $p:pat) => {};
    (@assert $($p:tt)*) => { ::std::compile_error!(::std::concat!("expected pattern, found `", ::std::stringify!($($p)*), "`")); };
    (@pat $scanner:expr, [] [])                                                     => {};
    (@pat $scanner:expr, [] [] , $($t:tt)*)                                         => { $crate::scan!(@pat $scanner, [] [] $($t)*) };
    (@pat $scanner:expr, [$($p:tt)*] [] $x:ident $($t:tt)*)                         => { $crate::scan!(@pat $scanner, [$($p)* $x] [] $($t)*) };
    (@pat $scanner:expr, [$($p:tt)*] [] :: $($t:tt)*)                               => { $crate::scan!(@pat $scanner, [$($p)* ::] [] $($t)*) };
    (@pat $scanner:expr, [$($p:tt)*] [] & $($t:tt)*)                                => { $crate::scan!(@pat $scanner, [$($p)* &] [] $($t)*) };
    (@pat $scanner:expr, [$($p:tt)*] [] ($($x:tt)*) $($t:tt)*)                      => { $crate::scan!(@pat $scanner, [$($p)* ($($x)*)] [] $($t)*) };
    (@pat $scanner:expr, [$($p:tt)*] [] [$($x:tt)*] $($t:tt)*)                      => { $crate::scan!(@pat $scanner, [$($p)* [$($x)*]] [] $($t)*) };
    (@pat $scanner:expr, [$($p:tt)*] [] {$($x:tt)*} $($t:tt)*)                      => { $crate::scan!(@pat $scanner, [$($p)* {$($x)*}] [] $($t)*) };
    (@pat $scanner:expr, [$($p:tt)*] [] : $($t:tt)*)                                => { $crate::scan!(@ty  $scanner, [$($p)*] [] $($t)*) };
    (@pat $scanner:expr, [$($p:tt)*] [] $($t:tt)*)                                  => { $crate::scan!(@let $scanner, [$($p)*] [usize] $($t)*) };
    (@ty  $scanner:expr, [$($p:tt)*] [$($tt:tt)*] @$e:expr $(, $($t:tt)*)?)         => { $crate::scan!(@let $scanner, [$($p)*] [$($tt)* @$e] $(, $($t)*)?) };
    (@ty  $scanner:expr, [$($p:tt)*] [$($tt:tt)*] ($($x:tt)*) $($t:tt)*)            => { $crate::scan!(@let $scanner, [$($p)*] [$($tt)* ($($x)*)] $($t)*) };
    (@ty  $scanner:expr, [$($p:tt)*] [$($tt:tt)*] [$($x:tt)*] $($t:tt)*)            => { $crate::scan!(@let $scanner, [$($p)*] [$($tt)* [$($x)*]] $($t)*) };
    (@ty  $scanner:expr, [$($p:tt)*] [$($tt:tt)*] &str $(, $($t:tt)*)?)             => { $crate::scan!(@let $scanner, [$($p)*] [$($tt)* &str] $(, $($t)*)?) };
    (@ty  $scanner:expr, [$($p:tt)*] [$($tt:tt)*] $ty:ty = $e:expr $(, $($t:tt)*)?) => { $crate::scan!(@let $scanner, [$($p)*] [$($tt)* $ty = $e] $(, $($t)*)?) };
    (@ty  $scanner:expr, [$($p:tt)*] [$($tt:tt)*] $ty:ty $(, $($t:tt)*)?)           => { $crate::scan!(@let $scanner, [$($p)*] [$($tt)* $ty] $(, $($t)*)?) };
    (@let $scanner:expr, [$($p:tt)*] [$($tt:tt)*] $($t:tt)*) => {
        $crate::scan!{@assert $($p)*}
        let $($p)* = $crate::scan_value!($scanner, $($tt)*);
        $crate::scan!(@pat $scanner, [] [] $($t)*)
    };
    (src = $src:expr, $($t:tt)*)   => { let mut __scanner = Scanner::new($src); $crate::scan!(@pat __scanner, [] [] $($t)*) };
    (iter = $iter:expr, $($t:tt)*) => { let mut __scanner = Scanner::new_from_iter($iter); $crate::scan!(@pat __scanner, [] [] $($t)*) };
    ($scanner:expr, $($t:tt)*) => { $crate::scan!(@pat $scanner, [] [] $($t)*) }
}

/// define enum scan rules
///
/// # Example
/// ```rust
/// # use competitive::{define_enum_scan, tools::{CharsWithBase, Scan, ScanSource, Scanner, Usize1}};
/// define_enum_scan! {
///   enum Query: u8 {
///     0 => Noop,
///     1 => Args { i: Usize1, s: char },
///     9 => Complex { n: usize, c: [(usize, Vec<usize> = CharsWithBase('a')); n] },
///   }
/// }
/// ```
#[macro_export]
macro_rules! define_enum_scan {
    (@field_ty @repeat [$($t:tt)*] $($len:expr)?)                           => { Vec<$crate::define_enum_scan!(@field_ty $($t)*)> };
    (@field_ty @array [$($t:tt)*] $len:expr)                                => { [$crate::define_enum_scan!(@field_ty $($t)*); $len] };
    (@field_ty @tuple [$([$($args:tt)*])*])                                 => { ($( $($args)* ,)*) };
    (@field_ty @sparen [] ($($tt:tt)*); $($t:tt)*)                          => { $crate::define_enum_scan!(@field_ty @sparen [($($tt)*)] $($t)*) };
    (@field_ty @sparen [] [$($tt:tt)*]; $($t:tt)*)                          => { $crate::define_enum_scan!(@field_ty @sparen [[$($tt)*]] $($t)*) };
    (@field_ty @sparen [] $ty:ty = $e:expr; $($t:tt)*)                      => { $crate::define_enum_scan!(@field_ty @sparen [$ty = $e] $($t)*) };
    (@field_ty @sparen [] $ty:ty; $($t:tt)*)                                => { $crate::define_enum_scan!(@field_ty @sparen [$ty] $($t)*) };
    (@field_ty @sparen [] $($args:tt)*)                                     => { $crate::define_enum_scan!(@field_ty @repeat [$($args)*]) };
    (@field_ty @sparen [$($args:tt)+] const $len:expr)                      => { $crate::define_enum_scan!(@field_ty @array [$($args)+] $len) };
    (@field_ty @sparen [$($args:tt)+] $len:expr)                            => { $crate::define_enum_scan!(@field_ty @repeat [$($args)+] $len) };
    (@field_ty @$tag:ident [$($args:tt)*] ($($tuple:tt)*) $($t:tt)*)        => { $crate::define_enum_scan!(@field_ty @$tag [$($args)* [$crate::define_enum_scan!(@field_ty @tuple [] $($tuple)*)]] $($t)*) };
    (@field_ty @$tag:ident [$($args:tt)*] [$($tt:tt)*] $($t:tt)*)           => { $crate::define_enum_scan!(@field_ty @$tag [$($args)* [$crate::define_enum_scan!(@field_ty @sparen [] $($tt)*)]] $($t)*) };
    (@field_ty @$tag:ident [$($args:tt)*] $ty:ty = $e:expr $(, $($t:tt)*)?) => { $crate::define_enum_scan!(@field_ty @$tag [$($args)* [$ty]] $(, $($t)*)?) };
    (@field_ty @$tag:ident [$($args:tt)*] $ty:ty $(, $($t:tt)*)?)           => { $crate::define_enum_scan!(@field_ty @$tag [$($args)* [<$ty as Scan>::Output]] $(, $($t)*)?) };
    (@field_ty @$tag:ident [$($args:tt)*] , $($t:tt)*)                      => { $crate::define_enum_scan!(@field_ty @$tag [$($args)*] $($t)*) };
    (@field_ty @$tag:ident [[$($args:tt)*]])                                => { $($args)* };
    (@field_ty @$tag:ident [$($args:tt)*])                                  => { ::std::compile_error!(::std::stringify!($($args)*)) };
    (@field_ty $($t:tt)*) => { $crate::define_enum_scan!(@field_ty @inner [] $($t)*) };

    (@tag_expr raw, $iter:ident) => { ScanSource::next_token($iter)? };
    (@tag_expr $d:ty, $iter:ident) => { <$d as Scan>::scan($iter)? };
    (@variant ([$($attr:tt)*] $vis:vis $T:ident $d:tt) [$($vars:tt)*]) => { $crate::define_enum_scan! { @def $($attr)* $vis enum $T : $d { $($vars)* } } };
    (@variant $ctx:tt [$($vars:tt)*] $p:pat => $v:ident { $($fs:tt)* } $($rest:tt)*) => { $crate::define_enum_scan! { @field   $ctx [$($vars)*] $p => $v [] $($fs)* ; $($rest)* } };
    (@variant $ctx:tt [$($vars:tt)*] $p:pat => $v:ident $($rest:tt)*)                    => { $crate::define_enum_scan! { @variant $ctx [$($vars)* $p => $v ,] $($rest)* } };
    (@variant $ctx:tt [$($vars:tt)*] , $($rest:tt)*)                                     => { $crate::define_enum_scan! { @variant $ctx [$($vars)*] $($rest)* } };
    (@endfield $ctx:tt [$($vars:tt)*] $p:pat => $v:ident [$($fs:tt)*] [$f:ident : $($spec:tt)*] , $($rest:tt)*) => { $crate::define_enum_scan! { @field $ctx [$($vars)*] $p => $v [$($fs)* [$f : $($spec)*]] $($rest)* } };
    (@endfield $ctx:tt [$($vars:tt)*] $p:pat => $v:ident [$($fs:tt)*] [$f:ident : $($spec:tt)*] ; $($rest:tt)*) => { $crate::define_enum_scan! { @variant $ctx [$($vars)* $p => $v { $($fs)* [$f : $($spec)*] } ,] $($rest)* } };
    (@field $ctx:tt [$($vars:tt)*] $p:pat => $v:ident [$($fs:tt)*] ; $($rest:tt)*)                                  => { $crate::define_enum_scan! { @variant $ctx [$($vars)* $p => $v { $($fs)* } ,] $($rest)* } };
    (@field $ctx:tt [$($vars:tt)*] $p:pat => $v:ident [$($fs:tt)*] $f:ident : ($($tuple:tt)*) $sep:tt $($rest:tt)*) => { $crate::define_enum_scan! { @endfield $ctx [$($vars)*] $p => $v [$($fs)*] [$f : ($($tuple)*)] $sep $($rest)* } };
    (@field $ctx:tt [$($vars:tt)*] $p:pat => $v:ident [$($fs:tt)*] $f:ident : [$($x:tt)*] $sep:tt $($rest:tt)*)     => { $crate::define_enum_scan! { @endfield $ctx [$($vars)*] $p => $v [$($fs)*] [$f : [$($x)*]] $sep $($rest)* } };
    (@field $ctx:tt [$($vars:tt)*] $p:pat => $v:ident [$($fs:tt)*] $f:ident : $ty:ty = $e:expr , $($rest:tt)*)      => { $crate::define_enum_scan! { @endfield $ctx [$($vars)*] $p => $v [$($fs)*] [$f : $ty = $e] , $($rest)* } };
    (@field $ctx:tt [$($vars:tt)*] $p:pat => $v:ident [$($fs:tt)*] $f:ident : $ty:ty ; $($rest:tt)*)                => { $crate::define_enum_scan! { @endfield $ctx [$($vars)*] $p => $v [$($fs)*] [$f : $ty] ; $($rest)* } };
    (@field $ctx:tt [$($vars:tt)*] $p:pat => $v:ident [$($fs:tt)*] $f:ident : $ty:ty = $e:expr ; $($rest:tt)*)      => { $crate::define_enum_scan! { @endfield $ctx [$($vars)*] $p => $v [$($fs)*] [$f : $ty = $e] ; $($rest)* } };
    (@field $ctx:tt [$($vars:tt)*] $p:pat => $v:ident [$($fs:tt)*] $f:ident : $ty:ty , $($rest:tt)*)                => { $crate::define_enum_scan! { @endfield $ctx [$($vars)*] $p => $v [$($fs)*] [$f : $ty] , $($rest)* } };
    (
        @def
        $(#[$attr:meta])*
        $vis:vis enum $T:ident : $d:tt {
            $( $p:pat => $v:ident $( { $( [$f:ident : $($spec:tt)*] )* } )?, )*
        }
    ) => {
        $(#[$attr])*
        $vis enum $T {
            $( $v $( { $( $f : $crate::define_enum_scan!(@field_ty $($spec)*) ),* } )? ),*
        }
        impl Scan for $T {
            type Output = Self;
            fn scan<I: ScanSource>(iter: &mut I) -> Option<Self> {
                let tag = $crate::define_enum_scan!(@tag_expr $d, iter);
                match tag {
                    $(
                        $p => {
                            $($(
                                let $f = $crate::scan_value!((*iter), $($spec)* );
                            )*)?
                            Some($T::$v $( { $( $f ),* } )?)
                        }
                    ),*
                    _ => None,
                }
            }
        }
    };
    (
        $(#[$attr:meta])*
        $vis:vis enum $T:ident : raw {
            $($body:tt)*
        }
    ) => {
        $crate::define_enum_scan! { @variant ([$(#[$attr])*] $vis $T raw) [] $($body)* }
    };
    (
        $(#[$attr:meta])*
        $vis:vis enum $T:ident : $d:ty {
            $($body:tt)*
        }
    ) => {
        $crate::define_enum_scan! { @variant ([$(#[$attr])*] $vis $T $d) [] $($body)* }
    };
}

#[derive(Debug, Copy, Clone)]
pub enum Usize1 {}
impl Scan for Usize1 {
    type Output = usize;
    fn scan<I: ScanSource>(iter: &mut I) -> Option<Self::Output> {
        <usize as Scan>::scan(iter)?.checked_sub(1)
    }
}
#[derive(Debug, Copy, Clone)]
pub struct CharWithBase(pub char);
impl MarkedScan for CharWithBase {
    type Output = usize;
    fn mscan<I: ScanSource>(self, iter: &mut I) -> Option<Self::Output> {
        Some((<char as Scan>::scan(iter)? as u8 - self.0 as u8) as usize)
    }
}
#[derive(Debug, Copy, Clone)]
pub enum Chars {}
impl Scan for Chars {
    type Output = Vec<char>;
    fn scan<I: ScanSource>(iter: &mut I) -> Option<Self::Output> {
        Some(iter.next_token()?.chars().collect())
    }
}
#[derive(Debug, Copy, Clone)]
pub struct CharsWithBase(pub char);
impl MarkedScan for CharsWithBase {
    type Output = Vec<usize>;
    fn mscan<I: ScanSource>(self, iter: &mut I) -> Option<Self::Output> {
        Some(
            iter.next_token()?
                .chars()
                .map(|c| (c as u8 - self.0 as u8) as usize)
                .collect(),
        )
    }
}
#[derive(Debug, Copy, Clone)]
pub enum Byte1 {}
impl Scan for Byte1 {
    type Output = u8;
    fn scan<I: ScanSource>(iter: &mut I) -> Option<Self::Output> {
        let bytes = iter.next_token()?.as_bytes();
        assert_eq!(bytes.len(), 1);
        Some(bytes[0])
    }
}
#[derive(Debug, Copy, Clone)]
pub struct ByteWithBase(pub u8);
impl MarkedScan for ByteWithBase {
    type Output = usize;
    fn mscan<I: ScanSource>(self, iter: &mut I) -> Option<Self::Output> {
        Some((<char as Scan>::scan(iter)? as u8 - self.0) as usize)
    }
}
#[derive(Debug, Copy, Clone)]
pub enum Bytes {}
impl Scan for Bytes {
    type Output = Vec<u8>;
    fn scan<I: ScanSource>(iter: &mut I) -> Option<Self::Output> {
        Some(iter.next_token()?.bytes().collect())
    }
}
#[derive(Debug, Copy, Clone)]
pub struct BytesWithBase(pub u8);
impl MarkedScan for BytesWithBase {
    type Output = Vec<usize>;
    fn mscan<I: ScanSource>(self, iter: &mut I) -> Option<Self::Output> {
        Some(
            iter.next_token()?
                .bytes()
                .map(|c| (c - self.0) as usize)
                .collect(),
        )
    }
}
#[derive(Debug, Copy, Clone)]
pub struct Collect<T, B = Vec<<T as Scan>::Output>>
where
    T: Scan,
    B: FromIterator<<T as Scan>::Output>,
{
    size: usize,
    _marker: PhantomData<fn() -> (T, B)>,
}
impl<T, B> Collect<T, B>
where
    T: Scan,
    B: FromIterator<<T as Scan>::Output>,
{
    pub fn new(size: usize) -> Self {
        Self {
            size,
            _marker: PhantomData,
        }
    }
}
impl<T, B> MarkedScan for Collect<T, B>
where
    T: Scan,
    B: FromIterator<<T as Scan>::Output>,
{
    type Output = B;
    fn mscan<I: ScanSource>(self, iter: &mut I) -> Option<Self::Output> {
        if self.size == 0 {
            iter.skip_whitespace();
        }
        repeat_with(|| <T as Scan>::scan(iter))
            .take(self.size)
            .collect()
    }
}
#[derive(Debug, Copy, Clone)]
pub struct SizedCollect<T, B = Vec<<T as Scan>::Output>>
where
    T: Scan,
    B: FromIterator<<T as Scan>::Output>,
{
    _marker: PhantomData<fn() -> (T, B)>,
}
impl<T, B> Scan for SizedCollect<T, B>
where
    T: Scan,
    B: FromIterator<<T as Scan>::Output>,
{
    type Output = B;
    fn scan<I: ScanSource>(iter: &mut I) -> Option<Self::Output> {
        let size = usize::scan(iter)?;
        if size == 0 {
            iter.skip_whitespace();
        }
        repeat_with(|| <T as Scan>::scan(iter)).take(size).collect()
    }
}
#[derive(Debug, Copy, Clone)]
pub struct Splitted<T, P>
where
    T: Scan,
{
    pat: P,
    _marker: PhantomData<fn() -> T>,
}
impl<T, P> Splitted<T, P>
where
    T: Scan,
{
    pub fn new(pat: P) -> Self {
        Self {
            pat,
            _marker: PhantomData,
        }
    }
}
impl<T> MarkedScan for Splitted<T, char>
where
    T: Scan,
{
    type Output = Vec<<T as Scan>::Output>;
    fn mscan<I: ScanSource>(self, iter: &mut I) -> Option<Self::Output> {
        let mut iter = Scanner::new_from_iter(iter.next_token()?.split(self.pat));
        Some(from_fn(|| <T as Scan>::scan(&mut iter)).collect())
    }
}
impl<T> MarkedScan for Splitted<T, &str>
where
    T: Scan,
{
    type Output = Vec<<T as Scan>::Output>;
    fn mscan<I: ScanSource>(self, iter: &mut I) -> Option<Self::Output> {
        let mut iter = Scanner::new_from_iter(iter.next_token()?.split(self.pat));
        Some(from_fn(|| <T as Scan>::scan(&mut iter)).collect())
    }
}
impl<T, F> MarkedScan for F
where
    F: Fn(&str) -> Option<T>,
{
    type Output = T;
    fn mscan<I: ScanSource>(self, iter: &mut I) -> Option<Self::Output> {
        self(iter.next_token()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan() {
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

        scan!(src = "12 34", c: Vec<usize> = CharsWithBase('0'), d: [Vec<usize> = CharsWithBase('0'); 1]);
        assert_eq!(c, vec![1, 2]);
        assert_eq!(d, vec![vec![3, 4]]);

        scan!(src = "1", x);
        assert_eq!(x, 1);
        assert_eq!(scan_value!(src = "1", usize), 1);

        scan!(iter = "1".split_ascii_whitespace(), x);
        assert_eq!(x, 1);
        assert_eq!(scan_value!(iter = "1".split_ascii_whitespace(), usize), 1);
    }

    #[test]
    fn test_define_enum_scan() {
        define_enum_scan! {
            enum Query: u8 {
                0 => Noop,
                1 => Args { i: Usize1, s: char },
                9 => Complex { n: usize, c: [(usize, Vec<usize> = CharsWithBase('a')); n] },
            }
        }

        let mut s = Scanner::new("0   1 2 a  9 2 3 ab 2 ab");
        scan!(s, q1: Query, q2: Query, q3: Query);
        match q1 {
            Query::Noop => {}
            _ => panic!("unexpected"),
        }
        match q2 {
            Query::Args { i, s } => {
                assert_eq!(i, 1);
                assert_eq!(s, 'a');
            }
            _ => panic!("unexpected"),
        }
        match q3 {
            Query::Complex { n, c } => {
                assert_eq!(n, 2);
                assert_eq!(c, vec![(3, vec![0, 1]), (2, vec![0, 1])]);
            }
            _ => panic!("unexpected"),
        }
    }
}

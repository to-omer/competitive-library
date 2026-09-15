use super::FastOutput;
use std::{fmt::Write as _, io::Write};

pub trait FastPrint {
    fn fast_print<W: Write>(&self, writer: &mut FastOutput<W>);
}

impl FastPrint for () {
    #[inline]
    fn fast_print<W: Write>(&self, _writer: &mut FastOutput<W>) {}
}

macro_rules! impl_fast_print_integer {
    ($($ty:ident)*) => {$(
        impl FastPrint for $ty {
            #[inline(always)]
            fn fast_print<W: Write>(&self, writer: &mut FastOutput<W>) {
                writer.$ty(*self);
            }
        }
    )*};
}
impl_fast_print_integer!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);

impl FastPrint for str {
    #[inline]
    fn fast_print<W: Write>(&self, writer: &mut FastOutput<W>) {
        writer.bytes(self.as_bytes());
    }
}
impl FastPrint for String {
    #[inline]
    fn fast_print<W: Write>(&self, writer: &mut FastOutput<W>) {
        self.as_str().fast_print(writer);
    }
}
impl FastPrint for char {
    #[inline(always)]
    fn fast_print<W: Write>(&self, writer: &mut FastOutput<W>) {
        if self.is_ascii() {
            writer.byte(*self as u8);
        } else {
            writer.bytes(self.encode_utf8(&mut [0; 4]).as_bytes());
        }
    }
}
impl<T: FastPrint + ?Sized> FastPrint for &T {
    #[inline]
    fn fast_print<W: Write>(&self, writer: &mut FastOutput<W>) {
        T::fast_print(self, writer);
    }
}
macro_rules! impl_fast_print_display {
    ($($ty:ty)*) => {$(
        impl FastPrint for $ty {
            fn fast_print<W: Write>(&self, writer: &mut FastOutput<W>) {
                write!(writer, "{}", self).expect("io error");
            }
        }
    )*};
}
impl_fast_print_display!(bool f32 f64);

pub trait FastIterPrint {
    fn fast_iter_print<W: Write, S: FastPrint>(
        self,
        writer: &mut FastOutput<W>,
        sep: S,
        is_head: bool,
    );
}
macro_rules! impl_fast_iter_print_tuple {
    (@impl) => {
        impl FastIterPrint for () {
            #[inline]
            fn fast_iter_print<W: Write, S: FastPrint>(self, _writer: &mut FastOutput<W>, _sep: S, _is_head: bool) {}
        }
    };
    (@impl $T:ident $v:ident $($U:ident $u:ident)*) => {
        impl<$T: FastPrint, $($U: FastPrint),*> FastIterPrint for ($T, $($U,)*) {
            #[inline]
            fn fast_iter_print<W: Write, S: FastPrint>(self, writer: &mut FastOutput<W>, sep: S, is_head: bool) {
                let ($v, $($u,)*) = self;
                if !is_head { sep.fast_print(writer); }
                $v.fast_print(writer);
                $(
                    sep.fast_print(writer);
                    $u.fast_print(writer);
                )*
            }
        }
    };
    (@inner [$($T:ident $v:ident)*] $U:ident $u:ident $($Rest:tt)*) => {
        impl_fast_iter_print_tuple!(@impl $($T $v)*);
        impl_fast_iter_print_tuple!(@inner [$($T $v)* $U $u] $($Rest)*);
    };
    (@inner [$($T:ident $v:ident)*]) => { impl_fast_iter_print_tuple!(@impl $($T $v)*); };
    ($($t:tt)*) => { impl_fast_iter_print_tuple!(@inner [] $($t)*); };
}
impl_fast_iter_print_tuple!(A a B b C c D d E e F f G g H h I i J j K k);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{iter_print, tools::Xorshift};

    #[test]
    fn test_fast_print_boundaries() {
        let mut rng = Xorshift::default();
        let mut values = vec![
            0,
            1,
            u64::MAX as u128,
            u64::MAX as u128 + 1,
            u128::MAX,
            i128::MAX as u128,
            1u128 << 127,
        ];
        for _ in 0..512 {
            values.push((rng.rand64() as u128) << 64 | rng.rand64() as u128);
        }
        let mut chars: Vec<_> = (0..=127).filter_map(char::from_u32).collect();
        chars.extend([
            '\u{80}',
            '\u{7ff}',
            '\u{800}',
            '\u{ffff}',
            '\u{10000}',
            '\u{10ffff}',
        ]);
        chars.extend((0..512).filter_map(|_| char::from_u32((rng.rand64() % 0x110000) as u32)));
        let mut expected = Vec::new();
        for &x in &values {
            writeln!(
                expected,
                "{} {} {} {} éあ",
                x, x as i128, x as usize, x as isize
            )
            .unwrap();
        }
        for &ch in &chars {
            writeln!(expected, "{ch}").unwrap();
        }
        for capacity in [0, 31, 32, 33, 63, 64, 127, 256] {
            let mut buf = Vec::new();
            {
                let mut writer = FastOutput::with_capacity(capacity, &mut buf);
                for &x in &values {
                    iter_print!(fast; writer, x, x as i128, x as usize, x as isize, "éあ");
                }
                for &ch in &chars {
                    iter_print!(fast; writer, ch);
                }
                writer.flush();
            }
            assert_eq!(buf, expected, "capacity={capacity}");
        }
    }
}

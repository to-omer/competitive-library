use std::{
    fmt::Display,
    io::{Error, Write},
};

pub trait IterPrint {
    fn iter_print<W, S>(self, writer: &mut W, sep: S, is_head: bool) -> Result<(), Error>
    where
        W: Write,
        S: Display;
}
macro_rules! impl_iter_print_tuple {
    (@impl ,) => {
        impl IterPrint for () {
            fn iter_print<W, S>(self, _writer: &mut W, _sep: S, _is_head: bool) -> Result<(), Error>
            where
                W: Write,
                S: Display
            {
                Ok(())
            }
        }
    };
    (@impl $($A:ident $a:ident)?, $($B:ident $b:ident)*) => {
        impl<$($A,)? $($B),*> IterPrint for ($($A,)? $($B),*)
        where
            $($A: Display,)? $($B: Display),*
        {
            fn iter_print<W, S>(self, writer: &mut W, sep: S, is_head: bool) -> Result<(), Error>
            where
                W: Write,
                S: Display
            {
                let ($($a,)? $($b,)*) = self;
                $(
                    if is_head {
                        ::std::write!(writer, "{}", $a)?;
                    } else {
                        ::std::write!(writer, "{}{}", sep, $a)?;
                    }
                )?
                $( ::std::write!(writer, "{}{}", sep, $b)?; )*
                Ok(())
            }
        }
    };
    (@inc , , $C:ident $c:ident $($D:ident $d:ident)*) => {
        impl_iter_print_tuple!(@impl ,);
        impl_iter_print_tuple!(@inc $C $c, , $($D $d)*);
    };
    (@inc $A:ident $a:ident, $($B:ident $b:ident)*, $C:ident $c:ident $($D:ident $d:ident)*) => {
        impl_iter_print_tuple!(@impl $A $a, $($B $b)*);
        impl_iter_print_tuple!(@inc $A $a, $($B $b)* $C $c, $($D $d)*);
    };
    (@inc $A:ident $a:ident, $($B:ident $b:ident)*,) => {
        impl_iter_print_tuple!(@impl $A $a, $($B $b)*);
    };
    ($($t:tt)*) => {
        impl_iter_print_tuple!(@inc , , $($t)*);
    };
}
impl_iter_print_tuple!(A a B b C c D d E e F f G g H h I i J j K k);

/// Print expressions with a separator.
/// - `iter_print!(writer, args...)`
/// - `iter_print!(fast; writer, args...)`: use `FastOutput` and `FastPrint`
/// - `@sep $expr`: set separator (default: `' '`)
/// - `@ns`: alias for `@sep ""`
/// - `@lf`: alias for `@sep '\n'`
/// - `@sp`: alias for `@sep ' '`
/// - `@fmt ($lit, $($expr),*)`: print `format!($lit, $($expr),*)`
/// - `@flush`: flush writer (auto insert `!`)
/// - `@it $expr`: print iterator
/// - `@it1 $expr`: print iterator as 1-indexed
/// - `@cw ($char $expr)`: print iterator as `(elem as u8 + $char as u8) as char`
/// - `@bw ($byte $expr)`: print iterator as `(elem as u8 + $byte) as char`
/// - `@it2d $expr`: print 2d-iterator
/// - `@tup $expr`: print tuple (need to import [`IterPrint`])
/// - `@ittup $expr`: print iterative tuple (need to import [`IterPrint`])
/// - `$expr`: print expr
/// - `{ args... }`: scoped
/// - `;`: print `'\n'`
/// - `!`: not print `'\n'` at the end
#[macro_export]
macro_rules! iter_print {
    (@@fmt normal $writer:expr, $sep:expr, $is_head:expr, ($lit:literal $(, $e:expr)* $(,)?)) => {
        if !$is_head {
            ::std::write!($writer, "{}", $sep).expect("io error");
        }
        ::std::write!($writer, $lit, $($e),*).expect("io error");
    };
    (@@fmt fast $writer:expr, $sep:expr, $is_head:expr, ($lit:literal $(, $e:expr)* $(,)?)) => {{
        use ::std::fmt::Write as _;
        if !$is_head {
            FastPrint::fast_print(&$sep, &mut $writer);
        }
        ::std::write!($writer, $lit, $($e),*).expect("io error");
    }};
    (@@item normal $writer:expr, $sep:expr, $is_head:expr, $e:expr) => {
        $crate::iter_print!(@@fmt normal $writer, $sep, $is_head, ("{}", $e));
    };
    (@@item fast $writer:expr, $sep:expr, $is_head:expr, $e:expr) => {
        if !$is_head {
            FastPrint::fast_print(&$sep, &mut $writer);
        }
        FastPrint::fast_print(&$e, &mut $writer);
    };
    (@@line_feed normal $writer:expr $(,)?) => {
        ::std::writeln!($writer).expect("io error");
    };
    (@@line_feed fast $writer:expr $(,)?) => {
        $writer.byte(b'\n');
    };
    (@@flush normal $writer:expr) => {{
        use ::std::io::Write as _;
        $writer.flush().expect("io error");
    }};
    (@@flush fast $writer:expr) => {
        $writer.flush();
    };
    (@@it fast $writer:expr, $sep:expr, $is_head:expr, $iter:expr) => {{
        // Keep borrowed temporary sources alive through the loop.
        match $iter.into_iter() {
            mut iter => {
                if let ::std::option::Option::Some(first) = iter.next() {
                    {
                        // Drop the first item before advancing the iterator.
                        let item = first;
                        $crate::iter_print!(@@item fast $writer, $sep, $is_head, item);
                    }
                    for item in iter {
                        $crate::iter_print!(@@item fast $writer, $sep, false, item);
                    }
                }
            }
        }
    }};
    (@@it $mode:ident $writer:expr, $sep:expr, $is_head:expr, $iter:expr) => {{
        let mut iter = $iter.into_iter();
        if let Some(item) = iter.next() {
            $crate::iter_print!(@@item $mode $writer, $sep, $is_head, item);
        }
        for item in iter {
            $crate::iter_print!(@@item $mode $writer, $sep, false, item);
        }
    }};
    (@@it1 $mode:ident $writer:expr, $sep:expr, $is_head:expr, $iter:expr) => {{
        let mut iter = $iter.into_iter();
        if let Some(item) = iter.next() {
            $crate::iter_print!(@@item $mode $writer, $sep, $is_head, item + 1);
        }
        for item in iter {
            $crate::iter_print!(@@item $mode $writer, $sep, false, item + 1);
        }
    }};
    (@@cw $mode:ident $writer:expr, $sep:expr, $is_head:expr, ($ch:literal $iter:expr)) => {{
        let mut iter = $iter.into_iter();
        let b = $ch as u8;
        if let Some(item) = iter.next() {
            $crate::iter_print!(@@item $mode $writer, $sep, $is_head, (item as u8 + b) as char);
        }
        for item in iter {
            $crate::iter_print!(@@item $mode $writer, $sep, false, (item as u8 + b) as char);
        }
    }};
    (@@bw $mode:ident $writer:expr, $sep:expr, $is_head:expr, ($b:literal $iter:expr)) => {{
        let mut iter = $iter.into_iter();
        let b: u8 = $b;
        if let Some(item) = iter.next() {
            $crate::iter_print!(@@item $mode $writer, $sep, $is_head, (item as u8 + b) as char);
        }
        for item in iter {
            $crate::iter_print!(@@item $mode $writer, $sep, false, (item as u8 + b) as char);
        }
    }};
    (@@it2d fast $writer:expr, $sep:expr, $is_head:expr, $iter:expr) => {
        for (i, item) in $iter.into_iter().enumerate() {
            if i > 0 {
                $crate::iter_print!(@@line_feed fast $writer);
            }
            $crate::iter_print!(@@it fast $writer, $sep, i > 0 || $is_head, item);
        }
    };
    (@@it2d $mode:ident $writer:expr, $sep:expr, $is_head:expr, $iter:expr) => {
        let mut iter = $iter.into_iter();
        if let Some(item) = iter.next() {
            $crate::iter_print!(@@it $mode $writer, $sep, $is_head, item);
        }
        for item in iter {
            $crate::iter_print!(@@line_feed $mode $writer);
            $crate::iter_print!(@@it $mode $writer, $sep, true, item);
        }
    };
    (@@tup normal $writer:expr, $sep:expr, $is_head:expr, $tuple:expr) => {
        IterPrint::iter_print($tuple, &mut $writer, $sep, $is_head).expect("io error");
    };
    (@@tup fast $writer:expr, $sep:expr, $is_head:expr, $tuple:expr) => {
        FastIterPrint::fast_iter_print($tuple, &mut $writer, $sep, $is_head);
    };
    (@@ittup $mode:ident $writer:expr, $sep:expr, $is_head:expr, $iter:expr) => {
        let mut iter = $iter.into_iter();
        if let Some(item) = iter.next() {
            $crate::iter_print!(@@tup $mode $writer, $sep, $is_head, item);
        }
        for item in iter {
            $crate::iter_print!(@@line_feed $mode $writer);
            $crate::iter_print!(@@tup $mode $writer, $sep, true, item);
        }
    };
    (@@assert_tag item) => {};
    (@@assert_tag it) => {};
    (@@assert_tag it1) => {};
    (@@assert_tag it2d) => {};
    (@@assert_tag tup) => {};
    (@@assert_tag ittup) => {};
    (@@assert_tag $tag:ident) => {
        ::std::compile_error!(::std::concat!("invalid tag in `iter_print!`: `", std::stringify!($tag), "`"));
    };
    (@@inner $mode:ident $writer:expr, $sep:expr, $is_head:expr, @sep $e:expr, $($t:tt)*) => {
        $crate::iter_print!(@@inner $mode $writer, $e, $is_head, $($t)*);
    };
    (@@inner fast $writer:expr, $sep:expr, $is_head:expr, @ns $($t:tt)*) => {
        $crate::iter_print!(@@inner fast $writer, (), $is_head, $($t)*);
    };
    (@@inner $mode:ident $writer:expr, $sep:expr, $is_head:expr, @ns $($t:tt)*) => {
        $crate::iter_print!(@@inner $mode $writer, "", $is_head, $($t)*);
    };
    (@@inner $mode:ident $writer:expr, $sep:expr, $is_head:expr, @lf $($t:tt)*) => {
        $crate::iter_print!(@@inner $mode $writer, '\n', $is_head, $($t)*);
    };
    (@@inner $mode:ident $writer:expr, $sep:expr, $is_head:expr, @sp $($t:tt)*) => {
        $crate::iter_print!(@@inner $mode $writer, ' ', $is_head, $($t)*);
    };
    (@@inner $mode:ident $writer:expr, $sep:expr, $is_head:expr, @flush $($t:tt)*) => {
        $crate::iter_print!(@@flush $mode $writer);
        $crate::iter_print!(@@inner $mode $writer, $sep, $is_head, ! $($t)*);
    };
    (@@inner $mode:ident $writer:expr, $sep:expr, $is_head:expr, @fmt $arg:tt $($t:tt)*) => {
        $crate::iter_print!(@@fmt $mode $writer, $sep, $is_head, $arg);
        $crate::iter_print!(@@inner $mode $writer, $sep, $is_head, $($t)*);
    };
    (@@inner $mode:ident $writer:expr, $sep:expr, $is_head:expr, @cw $arg:tt $($t:tt)*) => {
        $crate::iter_print!(@@cw $mode $writer, $sep, $is_head, $arg);
        $crate::iter_print!(@@inner $mode $writer, $sep, $is_head, $($t)*);
    };
    (@@inner $mode:ident $writer:expr, $sep:expr, $is_head:expr, @bw $arg:tt $($t:tt)*) => {
        $crate::iter_print!(@@bw $mode $writer, $sep, $is_head, $arg);
        $crate::iter_print!(@@inner $mode $writer, $sep, $is_head, $($t)*);
    };
    (@@inner $mode:ident $writer:expr, $sep:expr, $is_head:expr, @$tag:ident $e:expr, $($t:tt)*) => {
        $crate::iter_print!(@@assert_tag $tag);
        $crate::iter_print!(@@$tag $mode $writer, $sep, $is_head, $e);
        $crate::iter_print!(@@inner $mode $writer, $sep, false, $($t)*);
    };
    (@@inner $mode:ident $writer:expr, $sep:expr, $is_head:expr, @$tag:ident $e:expr; $($t:tt)*) => {
        $crate::iter_print!(@@assert_tag $tag);
        $crate::iter_print!(@@$tag $mode $writer, $sep, $is_head, $e);
        $crate::iter_print!(@@line_feed $mode $writer);
        $crate::iter_print!(@@inner $mode $writer, $sep, true, $($t)*);
    };
    (@@inner $mode:ident $writer:expr, $sep:expr, $is_head:expr, @$tag:ident $e:expr) => {
        $crate::iter_print!(@@assert_tag $tag);
        $crate::iter_print!(@@$tag $mode $writer, $sep, $is_head, $e);
        $crate::iter_print!(@@inner $mode $writer, $sep, false,);
    };
    (@@inner $mode:ident $writer:expr, $sep:expr, $is_head:expr, @$tag:ident $($t:tt)*) => {
        ::std::compile_error!(::std::concat!("invalid expr in `iter_print!`: `", std::stringify!($($t)*), "`"));
    };
    (@@inner $mode:ident $writer:expr, $sep:expr, $is_head:expr, , $($t:tt)*) => {
        $crate::iter_print!(@@inner $mode $writer, $sep, $is_head, $($t)*);
    };
    (@@inner $mode:ident $writer:expr, $sep:expr, $is_head:expr, ; $($t:tt)*) => {
        $crate::iter_print!(@@line_feed $mode $writer);
        $crate::iter_print!(@@inner $mode $writer, $sep, $is_head, $($t)*);
    };
    (@@inner $mode:ident $writer:expr, $sep:expr, $is_head:expr, ! $(,)?) => {};
    (@@inner $mode:ident $writer:expr, $sep:expr, $is_head:expr, ! $($t:tt)*) => {
        $crate::iter_print!(@@inner $mode $writer, $sep, $is_head, $($t)*);
    };
    (@@inner $mode:ident $writer:expr, $sep:expr, $is_head:expr,) => {
        $crate::iter_print!(@@line_feed $mode $writer);
    };
    (@@inner $mode:ident $writer:expr, $sep:expr, $is_head:expr, { $($t:tt)* } $($rest:tt)*) => {
        $crate::iter_print!(@@inner $mode $writer, $sep, $is_head, $($t)*, !);
        $crate::iter_print!(@@inner $mode $writer, $sep, $is_head, $($rest)*);
    };
    (@@inner $mode:ident $writer:expr, $sep:expr, $is_head:expr, $($t:tt)*) => {
        $crate::iter_print!(@@inner $mode $writer, $sep, $is_head, @item $($t)*);
    };
    (fast; $writer:expr, $($t:tt)*) => {{
        $crate::iter_print!(@@inner fast $writer, ' ', true, $($t)*);
    }};
    ($writer:expr, $($t:tt)*) => {{
        $crate::iter_print!(@@inner normal $writer, ' ', true, $($t)*);
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    use crate::tools::{FastIterPrint, FastOutput, FastPrint};
    use std::array;
    use std::iter;

    #[test]
    fn test_iter_print() {
        use std::fmt::Write as _;
        let mut rng = Xorshift::default();
        for _ in 0..1000 {
            let a: [i32; 23] = array::from_fn(|_| rng.random(-1000..=1000));
            let decimal = rng.random(-1000..=1000) as f64 / 10.0;
            let n = rng.random(0..=20);
            let letters: Vec<u8> = rng.random_iter(0..26).take(n).collect();
            let indices: Vec<usize> = rng.random_iter(0..100).take(n).collect();
            let mut buf = Vec::new();
            let mut fast_buf = Vec::new();
            macro_rules! check {
                ($writer:ident $(, $mode:ident)?) => {
                    iter_print!(
                        $($mode;)? $writer, a[0], a[1], @sep '.', a[2], a[3];
                        a[4], a[5], @sp @it &a[6..10];
                        @tup (a[0], a[1], a[2]); @flush a[3], @fmt ("{}?{}", a[4], decimal);
                        { @ns @it &a[7..10]; @lf @it &a[10..13] },
                        @it2d a[13..19].chunks(2);
                        @ns @ittup a[19..23].chunks(2).map(|row| (row[0], row[1]));
                        @flush,
                        @bw (b'a' letters.iter().copied());
                        @sp @it1 indices.iter().copied()
                    );
                };
            }
            check!(buf);
            {
                let mut out = FastOutput::new(&mut fast_buf);
                check!(out, fast);
            }
            let text: Vec<_> = a.iter().map(ToString::to_string).collect();
            let mut expected = String::new();
            writeln!(expected, "{} {}.{}.{}", a[0], a[1], a[2], a[3]).unwrap();
            writeln!(expected, "{}.{} {}", a[4], a[5], text[6..10].join(" ")).unwrap();
            writeln!(expected, "{}", text[..3].join(" ")).unwrap();
            writeln!(expected, "{} {}?{}", a[3], a[4], decimal).unwrap();
            writeln!(expected, "{}", text[7..10].concat()).unwrap();
            writeln!(
                expected,
                "{} {}",
                text[10..13].join("\n"),
                text[13..15].join(" ")
            )
            .unwrap();
            for row in text[15..19].chunks(2) {
                writeln!(expected, "{}", row.join(" ")).unwrap();
            }
            for row in text[19..23].chunks(2) {
                writeln!(expected, "{}", row.concat()).unwrap();
            }
            writeln!(
                expected,
                "{}",
                letters
                    .iter()
                    .map(|&x| char::from(b'a' + x))
                    .collect::<String>()
            )
            .unwrap();
            writeln!(
                expected,
                "{}",
                indices
                    .iter()
                    .map(|x| (x + 1).to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            )
            .unwrap();
            assert_eq!(buf, expected.as_bytes());
            assert_eq!(fast_buf, expected.as_bytes());
        }
    }

    #[test]
    fn test_iter_print_iterators() {
        for mask in 0..256 {
            for n in 0..=4 {
                let rows: Vec<Vec<_>> = (0..n)
                    .map(|i| (0..(mask >> (i * 2)) & 3).map(|j| i * 10 + j).collect())
                    .collect();
                let expected = rows
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(":")
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                let mut normal = Vec::new();
                iter_print!(normal, @sep ':', @it2d &rows, !);
                let mut fast = Vec::new();
                let mut calls = 0;
                let mut separators = 0;
                {
                    let mut writer = FastOutput::new(&mut fast);
                    iter_print!(fast; writer, @sep { separators += 1; ':' }, @it2d { calls += 1; &rows }, !);
                }
                assert_eq!(calls, 1);
                assert_eq!(
                    separators,
                    rows.iter()
                        .map(|row| row.len().saturating_sub(1))
                        .sum::<usize>()
                );
                assert_eq!(normal, expected.as_bytes());
                assert_eq!(fast, normal);
            }
        }
    }

    #[test]
    fn test_fast_iter_print_order() {
        use std::cell::Cell;

        struct Item<'a> {
            value: usize,
            dropped: &'a Cell<usize>,
        }
        impl FastPrint for Item<'_> {
            fn fast_print<W: std::io::Write>(&self, writer: &mut FastOutput<W>) {
                writer.usize(self.value);
            }
        }
        impl Drop for Item<'_> {
            fn drop(&mut self) {
                self.dropped.set(self.dropped.get() + 1);
            }
        }

        for n in 0..=32 {
            let dropped = Cell::new(0);
            let mut next = 0;
            let iter = iter::from_fn(|| {
                let value = next;
                assert_eq!(dropped.get(), value);
                next += 1;
                (value != n).then(|| Item {
                    value,
                    dropped: &dropped,
                })
            });
            let mut output = Vec::new();
            {
                let mut writer = FastOutput::new(&mut output);
                iter_print!(fast; writer, @it iter);
            }
            assert_eq!(next, n + 1);
            let expected = format!(
                "{}\n",
                (0..n).map(|i| i.to_string()).collect::<Vec<_>>().join(" ")
            );
            assert_eq!(output, expected.as_bytes());
            output.clear();
            {
                let mut writer = FastOutput::new(&mut output);
                iter_print!(fast; writer, @it (0..n).collect::<Vec<_>>().iter());
            }
            assert_eq!(output, expected.as_bytes());
        }
    }
}

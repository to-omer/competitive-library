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
    use crate::tools::{FastIterPrint, FastOutput, FastPrint};

    #[test]
    fn test_iter_print() {
        let mut buf = Vec::new();
        let mut fast_buf = Vec::new();
        macro_rules! check {
            ($writer:ident $(, $mode:ident)?) => {
        iter_print!(
            $($mode;)? $writer, 1, 2, @sep '.', 3, 4; 5, 6, @sp @it 7..=10;
            @tup (1, 2, 3); @flush 4, @fmt ("{}?{}", 5, 6.7);
            { @ns @it 8..=10; @lf @it 11..=13 },
            @it2d (0..3).map(|i| (14..=15).map(move |j| i * 2 + j));
            @ns @ittup (0..2).map(|i| (i * 2 + 20, i * 2 + 21));
            @flush,
            @bw (b'a' [0, 1, 2].iter().cloned());
            @sp @it1 (0..2)
        );
            };
        }
        check!(buf);
        {
            let mut out = FastOutput::new(&mut fast_buf);
            check!(out, fast);
        }
        let expected = r#"1 2.3.4
5.6 7 8 9 10
1 2 3
4 5?6.7
8910
11
12
13 14 15
16 17
18 19
2021
2223
abc
1 2
"#;
        assert_eq!(expected, String::from_utf8_lossy(&buf));
        assert_eq!(buf, fast_buf);
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
            let iter = std::iter::from_fn(|| {
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

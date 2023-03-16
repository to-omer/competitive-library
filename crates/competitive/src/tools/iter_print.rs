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
macro_rules! iter_print_tuple_impl {
    (@impl $($A:ident $a:ident)?, $($B:ident $b:ident)*) => {
        impl<$($A,)? $($B),*> IterPrint for ($($A,)? $($B),*)
        where
            $($A: Display,)? $($B: Display),*
        {
            #[allow(unused_variables)]
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
        iter_print_tuple_impl!(@impl ,);
        iter_print_tuple_impl!(@inc $C $c, , $($D $d)*);
    };
    (@inc $A:ident $a:ident, $($B:ident $b:ident)*, $C:ident $c:ident $($D:ident $d:ident)*) => {
        iter_print_tuple_impl!(@impl $A $a, $($B $b)*);
        iter_print_tuple_impl!(@inc $A $a, $($B $b)* $C $c, $($D $d)*);
    };
    (@inc $A:ident $a:ident, $($B:ident $b:ident)*,) => {
        iter_print_tuple_impl!(@impl $A $a, $($B $b)*);
    };
    ($($t:tt)*) => {
        iter_print_tuple_impl!(@inc , , $($t)*);
    };
}
iter_print_tuple_impl!(A a B b C c D d E e F f G g H h I i J j K k);

/// Print expressions with a separator.
/// - `iter_print!(writer, args...)`
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
    (@@fmt $writer:expr, $sep:expr, $is_head:expr, ($lit:literal $(, $e:expr)* $(,)?)) => {
        if !$is_head {
            ::std::write!($writer, "{}", $sep).expect("io error");
        }
        ::std::write!($writer, $lit, $($e),*).expect("io error");
    };
    (@@item $writer:expr, $sep:expr, $is_head:expr, $e:expr) => {
        $crate::iter_print!(@@fmt $writer, $sep, $is_head, ("{}", $e));
    };
    (@@line_feed $writer:expr $(,)?) => {
        ::std::writeln!($writer).expect("io error");
    };
    (@@it $writer:expr, $sep:expr, $is_head:expr, $iter:expr) => {{
        let mut iter = $iter.into_iter();
        if let Some(item) = iter.next() {
            $crate::iter_print!(@@item $writer, $sep, $is_head, item);
        }
        for item in iter {
            $crate::iter_print!(@@item $writer, $sep, false, item);
        }
    }};
    (@@it1 $writer:expr, $sep:expr, $is_head:expr, $iter:expr) => {{
        let mut iter = $iter.into_iter();
        if let Some(item) = iter.next() {
            $crate::iter_print!(@@item $writer, $sep, $is_head, item + 1);
        }
        for item in iter {
            $crate::iter_print!(@@item $writer, $sep, false, item + 1);
        }
    }};
    (@@cw $writer:expr, $sep:expr, $is_head:expr, ($ch:literal $iter:expr)) => {{
        let mut iter = $iter.into_iter();
        let b = $ch as u8;
        if let Some(item) = iter.next() {
            $crate::iter_print!(@@item $writer, $sep, $is_head, (item as u8 + b) as char);
        }
        for item in iter {
            $crate::iter_print!(@@item $writer, $sep, false, (item as u8 + b) as char);
        }
    }};
    (@@bw $writer:expr, $sep:expr, $is_head:expr, ($b:literal $iter:expr)) => {{
        let mut iter = $iter.into_iter();
        let b: u8 = $b;
        if let Some(item) = iter.next() {
            $crate::iter_print!(@@item $writer, $sep, $is_head, (item as u8 + b) as char);
        }
        for item in iter {
            $crate::iter_print!(@@item $writer, $sep, false, (item as u8 + b) as char);
        }
    }};
    (@@it2d $writer:expr, $sep:expr, $is_head:expr, $iter:expr) => {
        let mut iter = $iter.into_iter();
        if let Some(item) = iter.next() {
            $crate::iter_print!(@@it $writer, $sep, $is_head, item);
        }
        for item in iter {
            $crate::iter_print!(@@line_feed $writer);
            $crate::iter_print!(@@it $writer, $sep, true, item);
        }
    };
    (@@tup $writer:expr, $sep:expr, $is_head:expr, $tuple:expr) => {
        IterPrint::iter_print($tuple, &mut $writer, $sep, $is_head).expect("io error");
    };
    (@@ittup $writer:expr, $sep:expr, $is_head:expr, $iter:expr) => {
        let mut iter = $iter.into_iter();
        if let Some(item) = iter.next() {
            $crate::iter_print!(@@tup $writer, $sep, $is_head, item);
        }
        for item in iter {
            $crate::iter_print!(@@line_feed $writer);
            $crate::iter_print!(@@tup $writer, $sep, true, item);
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
    (@@inner $writer:expr, $sep:expr, $is_head:expr, @sep $e:expr, $($t:tt)*) => {
        $crate::iter_print!(@@inner $writer, $e, $is_head, $($t)*);
    };
    (@@inner $writer:expr, $sep:expr, $is_head:expr, @ns $($t:tt)*) => {
        $crate::iter_print!(@@inner $writer, "", $is_head, $($t)*);
    };
    (@@inner $writer:expr, $sep:expr, $is_head:expr, @lf $($t:tt)*) => {
        $crate::iter_print!(@@inner $writer, '\n', $is_head, $($t)*);
    };
    (@@inner $writer:expr, $sep:expr, $is_head:expr, @sp $($t:tt)*) => {
        $crate::iter_print!(@@inner $writer, ' ', $is_head, $($t)*);
    };
    (@@inner $writer:expr, $sep:expr, $is_head:expr, @flush $($t:tt)*) => {
        $writer.flush().expect("io error");
        $crate::iter_print!(@@inner $writer, $sep, $is_head, ! $($t)*);
    };
    (@@inner $writer:expr, $sep:expr, $is_head:expr, @fmt $arg:tt $($t:tt)*) => {
        $crate::iter_print!(@@fmt $writer, $sep, $is_head, $arg);
        $crate::iter_print!(@@inner $writer, $sep, $is_head, $($t)*);
    };
    (@@inner $writer:expr, $sep:expr, $is_head:expr, @cw $arg:tt $($t:tt)*) => {
        $crate::iter_print!(@@cw $writer, $sep, $is_head, $arg);
        $crate::iter_print!(@@inner $writer, $sep, $is_head, $($t)*);
    };
    (@@inner $writer:expr, $sep:expr, $is_head:expr, @bw $arg:tt $($t:tt)*) => {
        $crate::iter_print!(@@bw $writer, $sep, $is_head, $arg);
        $crate::iter_print!(@@inner $writer, $sep, $is_head, $($t)*);
    };
    (@@inner $writer:expr, $sep:expr, $is_head:expr, @$tag:ident $e:expr, $($t:tt)*) => {
        $crate::iter_print!(@@assert_tag $tag);
        $crate::iter_print!(@@$tag $writer, $sep, $is_head, $e);
        $crate::iter_print!(@@inner $writer, $sep, false, $($t)*);
    };
    (@@inner $writer:expr, $sep:expr, $is_head:expr, @$tag:ident $e:expr; $($t:tt)*) => {
        $crate::iter_print!(@@assert_tag $tag);
        $crate::iter_print!(@@$tag $writer, $sep, $is_head, $e);
        $crate::iter_print!(@@line_feed $writer);
        $crate::iter_print!(@@inner $writer, $sep, true, $($t)*);
    };
    (@@inner $writer:expr, $sep:expr, $is_head:expr, @$tag:ident $e:expr) => {
        $crate::iter_print!(@@assert_tag $tag);
        $crate::iter_print!(@@$tag $writer, $sep, $is_head, $e);
        $crate::iter_print!(@@inner $writer, $sep, false,);
    };
    (@@inner $writer:expr, $sep:expr, $is_head:expr, @$tag:ident $($t:tt)*) => {
        ::std::compile_error!(::std::concat!("invalid expr in `iter_print!`: `", std::stringify!($($t)*), "`"));
    };
    (@@inner $writer:expr, $sep:expr, $is_head:expr, , $($t:tt)*) => {
        $crate::iter_print!(@@inner $writer, $sep, $is_head, $($t)*);
    };
    (@@inner $writer:expr, $sep:expr, $is_head:expr, ; $($t:tt)*) => {
        $crate::iter_print!(@@line_feed $writer);
        $crate::iter_print!(@@inner $writer, $sep, $is_head, $($t)*);
    };
    (@@inner $writer:expr, $sep:expr, $is_head:expr, ! $(,)?) => {};
    (@@inner $writer:expr, $sep:expr, $is_head:expr, ! $($t:tt)*) => {
        $crate::iter_print!(@@inner $writer, $sep, $is_head, $($t)*);
    };
    (@@inner $writer:expr, $sep:expr, $is_head:expr,) => {
        $crate::iter_print!(@@line_feed $writer);
    };
    (@@inner $writer:expr, $sep:expr, $is_head:expr, { $($t:tt)* } $($rest:tt)*) => {
        $crate::iter_print!(@@inner $writer, $sep, $is_head, $($t)*, !);
        $crate::iter_print!(@@inner $writer, $sep, $is_head, $($rest)*);
    };
    (@@inner $writer:expr, $sep:expr, $is_head:expr, $($t:tt)*) => {
        $crate::iter_print!(@@inner $writer, $sep, $is_head, @item $($t)*);
    };
    ($writer:expr, $($t:tt)*) => {{
        $crate::iter_print!(@@inner $writer, ' ', true, $($t)*);
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iter_print;

    #[test]
    fn test_iter_print() {
        let mut buf = Vec::new();
        iter_print!(
            buf, 1, 2, @sep '.', 3, 4; 5, 6, @sp @it 7..=10;
            @tup (1, 2, 3); @flush 4, @fmt ("{}?{}", 5, 6.7);
            { @ns @it 8..=10; @lf @it 11..=13 },
            @it2d (0..3).map(|i| (14..=15).map(move |j| i * 2 + j));
            @ns @ittup (0..2).map(|i| (i * 2 + 20, i * 2 + 21));
            @flush,
            @cw ('a' [0, 1, 2].iter().cloned());
            @sp @it1 (0..2)
        );
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
    }
}

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
/// - `@fmt $lit => {$($expr),*}`: print `format!($lit, $($expr),*)`
/// - `@flush`: flush writer (auto insert `!`)
/// - `@iter $expr`: print iterator
/// - `@iterns $expr`: print iterator with no separators
/// - `@iterln $expr`: print iterator with separator `'\n'`
/// - `@iter2d $expr`: print 2d-iterator
/// - `@tuple $expr`: print tuple (need to import [`IterPrint`], each elements impls `Display`)
/// - `$expr`: print expr
/// - `;`: print `'\n'`
/// - `!`: not print `'\n'` at the end
#[macro_export]
macro_rules! iter_print {
    (@@fmt $writer:expr, $sep:expr, $is_head:expr, $lit:literal, $($e:expr),*) => {
        if !$is_head {
            ::std::write!($writer, "{}", $sep).expect("io error");
        }
        ::std::write!($writer, $lit, $($e),*).expect("io error");
    };
    (@@item $writer:expr, $sep:expr, $is_head:expr, $e:expr) => {
        $crate::iter_print!(@@fmt $writer, $sep, $is_head, "{}", $e);
    };
    (@@line_feed $writer:expr $(,)?) => {
        ::std::writeln!($writer).expect("io error");
    };
    (@@iter $writer:expr, $sep:expr, $is_head:expr, $iter:expr) => {{
        let mut iter = $iter.into_iter();
        if let Some(item) = iter.next() {
            $crate::iter_print!(@@item $writer, $sep, $is_head, item);
        }
        for item in iter {
            $crate::iter_print!(@@item $writer, $sep, false, item);
        }
    }};
    (@@iterns $writer:expr, $sep:expr, $is_head:expr, $iter:expr) => {{
        let mut iter = $iter.into_iter();
        if let Some(item) = iter.next() {
            $crate::iter_print!(@@item $writer, $sep, $is_head, item);
        }
        for item in iter {
            $crate::iter_print!(@@item $writer, $sep, true, item);
        }
    }};
    (@@iterln $writer:expr, $sep:expr, $is_head:expr, $iter:expr) => {{
        let mut iter = $iter.into_iter();
        if let Some(item) = iter.next() {
            $crate::iter_print!(@@item $writer, '\n', $is_head, item);
        }
        for item in iter {
            $crate::iter_print!(@@item $writer, '\n', false, item);
        }
    }};
    (@@iter2d $writer:expr, $sep:expr, $is_head:expr, $iter:expr) => {
        let mut iter = $iter.into_iter();
        if let Some(item) = iter.next() {
            $crate::iter_print!(@@iter $writer, $sep, $is_head, item);
        }
        for item in iter {
            $crate::iter_print!(@@line_feed $writer);
            $crate::iter_print!(@@iter $writer, $sep, true, item);
        }
    };
    (@@tuple $writer:expr, $sep:expr, $is_head:expr, $tuple:expr) => {
        IterPrint::iter_print($tuple, &mut $writer, $sep, $is_head).expect("io error");
    };
    (@@assert_tag item) => {};
    (@@assert_tag iter) => {};
    (@@assert_tag iterns) => {};
    (@@assert_tag iterln) => {};
    (@@assert_tag iter2d) => {};
    (@@assert_tag tuple) => {};
    (@@assert_tag $tag:ident) => {
        ::std::compile_error!(::std::concat!("invalid tag in `iter_print!`: `", std::stringify!($tag), "`"));
    };
    (@@inner $writer:expr, $sep:expr, $is_head:expr, @sep $e:expr, $($t:tt)*) => {
        $crate::iter_print!(@@inner $writer, $e, $is_head, $($t)*);
    };
    (@@inner $writer:expr, $sep:expr, $is_head:expr, @flush $($t:tt)*) => {
        $writer.flush().expect("io error");
        $crate::iter_print!(@@inner $writer, $sep, $is_head, ! $($t)*);
    };
    (@@inner $writer:expr, $sep:expr, $is_head:expr, @fmt $lit:literal => {$($e:expr),* $(,)?} $($t:tt)*) => {
        $crate::iter_print!(@@fmt $writer, $sep, $is_head, $lit, $($e),*);
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
    (@@inner $writer:expr, $sep:expr, $is_head:expr, ! $($t:tt)*) => {
        $crate::iter_print!(@@inner $writer, $sep, $is_head, $($t)*);
    };
    (@@inner $writer:expr, $sep:expr, $is_head:expr, !) => {};
    (@@inner $writer:expr, $sep:expr, $is_head:expr,) => {
        $crate::iter_print!(@@line_feed $writer);
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
            buf, 1, 2, @sep '.', 3, 4; 5, 6, @sep ' ', @iter 7..=10;
            @tuple (1, 2, 3);
            @flush,
            4, @fmt "{}?{}" => {5, 6.7}, @iterns 8..=10;
            @iterln 11..=13,
            @iter2d (0..3).map(|i| (14..=15).map(move |j| j + 2 * i)),
            @flush,
        );
        let expected =
            "1 2.3.4\n5.6 7 8 9 10\n1 2 3\n4 5?6.7 8910\n11\n12\n13 14 15\n16 17\n18 19\n";
        assert_eq!(expected, String::from_utf8_lossy(&buf));
    }
}

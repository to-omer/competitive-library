#[macro_export]
#[cargo_snippet::snippet("input")]
macro_rules! read_value {
    ($iter:expr, ( $($t:tt),* )) => {
        ( $($crate::read_value!($iter, $t)),* )
    };
    ($iter:expr, [ $t:tt ; $len:expr ]) => {
        (0..$len).map(|_| $crate::read_value!($iter, $t)).collect::<Vec<_>>()
    };
    ($iter:expr, { chars: $base:expr }) => {
        $crate::read_value!($iter, String).chars().map(|c| (c as u8 - $base as u8) as usize).collect::<Vec<usize>>()
    };
    ($iter:expr, { char: $base:expr }) => {
        $crate::read_value!($iter, { chars: $base })[0]
    };
    ($iter:expr, chars) => {
        $crate::read_value!($iter, String).chars().collect::<Vec<char>>()
    };
    ($iter:expr, char) => {
        $crate::read_value!($iter, chars)[0]
    };
    ($iter:expr, usize1) => {
        $crate::read_value!($iter, usize) - 1
    };
    ($iter:expr, $t:ty) => {
        $iter.next().unwrap().parse::<$t>().unwrap()
    };
}
#[macro_export]
#[cargo_snippet::snippet("input")]
macro_rules! input_inner {
    ($iter:expr) => {};
    ($iter:expr, ) => {};
    ($iter:expr, mut $var:ident : $t:tt $($r:tt)*) => {
        let mut $var = $crate::read_value!($iter, $t);
        $crate::input_inner!{$iter $($r)*}
    };
    ($iter:expr, mut $var:ident $($r:tt)*) => {
        $crate::input_inner!{$iter, mut $var : usize $($r)*}
    };
    ($iter:expr, $var:ident : $t:tt $($r:tt)*) => {
        let $var = $crate::read_value!($iter, $t);
        $crate::input_inner!{$iter $($r)*}
    };
    ($iter:expr, $var:ident $($r:tt)*) => {
        $crate::input_inner!{$iter, $var : usize $($r)*}
    };
}
#[macro_export]
#[cargo_snippet::snippet("input")]
macro_rules! input {
    (source = $s:expr, $($r:tt)*) => {
        let mut iter = $s.split_whitespace();
        $crate::input_inner!{iter, $($r)*}
    };
    (iter = $iter:ident, $($r:tt)*) => {
        let s = {
            use std::io::Read;
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s).unwrap();
            s
        };
        let mut $iter = s.split_whitespace();
        $crate::input_inner!{$iter, $($r)*}
    };
    ($($r:tt)*) => {
        let s = {
            use std::io::Read;
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s).unwrap();
            s
        };
        let mut iter = s.split_whitespace();
        $crate::input_inner!{iter, $($r)*}
    };
}

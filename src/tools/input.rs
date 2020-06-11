#[macro_export]
#[cargo_snippet::snippet("input")]
macro_rules! read_value {
    ($iter:expr, ( $($t:tt),* )) => {
        ($(read_value!($iter, $t)),*)
    };
    ($iter:expr, [ $t:tt ; $len:expr ]) => {
        (0..$len).map(|_| read_value!($iter, $t)).collect::<Vec<_>>()
    };
    ($iter:expr, { $t:tt => $f:expr }) => {
        $f(read_value!($iter, $t))
    };
    ($iter:expr, chars) => {
        read_value!($iter, String).chars().collect::<Vec<char>>()
    };
    ($iter:expr, usize1) => {
        read_value!($iter, usize) - 1
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
        let mut $var = read_value!($iter, $t);
        input_inner!{$iter $($r)*}
    };
    ($iter:expr, mut $var:ident $($r:tt)*) => {
        input_inner!{$iter, mut $var : usize $($r)*}
    };
    ($iter:expr, $var:ident : $t:tt $($r:tt)*) => {
        let $var = read_value!($iter, $t);
        input_inner!{$iter $($r)*}
    };
    ($iter:expr, $var:ident $($r:tt)*) => {
        input_inner!{$iter, $var : usize $($r)*}
    };
}
#[macro_export]
#[cargo_snippet::snippet("input")]
macro_rules! input {
    (source = $s:expr, $($r:tt)*) => {
        let mut iter = $s.split_whitespace();
        input_inner!{iter, $($r)*}
    };
    (iter = $iter:ident, $($r:tt)*) => {
        let s = {
            use std::io::Read;
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s).unwrap();
            s
        };
        let mut $iter = s.split_whitespace();
        input_inner!{$iter, $($r)*}
    };
    ($($r:tt)*) => {
        let s = {
            use std::io::Read;
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s).unwrap();
            s
        };
        let mut iter = s.split_whitespace();
        input_inner!{iter, $($r)*}
    };
}

#[test]
fn test_input() {
    input!(source = "1 2 3", x, y: char, z: {usize => |z| z - 1});
    assert_eq!(x, 1);
    assert_eq!(y, '2');
    assert_eq!(z, 2);

    input!(source = r#"1 2
2 3
4 5
    "#, edges: [({usize => |x| x - 1}, {usize => |x| x - 1}); 3]);
    assert_eq!(edges, vec![(0, 1), (1, 2), (3, 4)]);
}

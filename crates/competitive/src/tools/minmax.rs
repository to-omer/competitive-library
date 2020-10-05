#[snippet::entry("minmax")]
#[macro_export]
macro_rules! min {
    ($e:expr) => { $e };
    ($e:expr, $($es:expr),+) => { std::cmp::min($e, min!($($es),+)) };
}

#[snippet::entry("minmax")]
#[macro_export]
macro_rules! chmin {
    ($dst:expr, $($src:expr),+) => {{
        let x = std::cmp::min($dst, min!($($src),+));
        $dst = x;
    }};
}

#[snippet::entry("minmax")]
#[macro_export]
macro_rules! max {
    ($e:expr) => { $e };
    ($e:expr, $($es:expr),+) => { std::cmp::max($e, max!($($es),+)) };
}

#[snippet::entry("minmax")]
#[macro_export]
macro_rules! chmax {
    ($dst:expr, $($src:expr),+) => {{
        let x = std::cmp::max($dst, max!($($src),+));
        $dst = x;
    }};
}

#[test]
fn test_min() {
    assert_eq!(min!(1), 1);
    assert_eq!(min!(1, 2), 1);
    assert_eq!(min!(4, 1, 2), 1);
    assert_eq!(min!(4, 9, 2, 3), 2);
}

#[test]
fn test_chmin() {
    let mut x = 100;
    chmin!(x, 101);
    assert_eq!(x, 100);
    chmin!(x, 91, 78);
    assert_eq!(x, 78);
    chmin!(x, 61, 42, 51);
    assert_eq!(x, 42);

    let mut v = vec![31, 12];
    chmin!(v[0], v[1], 14);
    assert_eq!(v[0], v[1]);
}

#[test]
fn test_max() {
    assert_eq!(max!(1), 1);
    assert_eq!(max!(1, 2), 2);
    assert_eq!(max!(4, 1, 2), 4);
    assert_eq!(max!(4, 9, 2, 3), 9);
}

#[test]
fn test_chmax() {
    let mut x = 100;
    chmax!(x, 91);
    assert_eq!(x, 100);
    chmax!(x, 191, 178);
    assert_eq!(x, 191);
    chmax!(x, 261, 242, 251);
    assert_eq!(x, 261);

    let mut v = vec![31, 42];
    chmax!(v[0], v[1], 14);
    assert_eq!(v[0], v[1]);
}

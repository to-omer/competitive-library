use cargo_snippet::snippet;

#[snippet("binary_search")]
pub trait Bisect: Copy {
    fn halve(self, other: Self) -> Self;
    fn section_end(self, other: Self) -> bool;
}
#[snippet("binary_search")]
impl Bisect for usize {
    fn halve(self, other: Self) -> Self {
        if self > other {
            other + (self - other) / 2
        } else {
            self + (other - self) / 2
        }
    }
    fn section_end(self, other: Self) -> bool {
        (if self > other {
            self - other
        } else {
            other - self
        }) <= 1
    }
}
#[snippet("binary_search")]
impl Bisect for i64 {
    fn halve(self, other: Self) -> Self {
        (self + other) / 2
    }
    fn section_end(self, other: Self) -> bool {
        (self - other).abs() <= 1
    }
}
#[snippet("binary_search")]
impl Bisect for f64 {
    fn halve(self, other: Self) -> Self {
        (self + other) / 2.
    }
    fn section_end(self, other: Self) -> bool {
        (self - other).abs() <= 1e-8
    }
}
#[snippet("binary_search")]
pub fn binary_search<T: Bisect, F: Fn(T) -> bool>(f: F, ok: T, err: T) -> T {
    let mut ok = ok;
    let mut err = err;
    while !ok.section_end(err) {
        let m = ok.halve(err);
        if f(m) {
            ok = m;
        } else {
            err = m;
        }
    }
    ok
}

#[snippet("binary_search")]
pub fn lower_bound<T: Bisect + Ord>(v: &[T], x: T) -> usize {
    binary_search(|i| v[i as usize] >= x, v.len() as i64, -1) as usize
}

#[snippet("binary_search")]
pub fn upper_bound<T: Bisect + Ord>(v: &[T], x: T) -> usize {
    binary_search(|i| v[i as usize] > x, v.len() as i64, -1) as usize
}

#[test]
fn test_binary_search() {
    let v = vec![0, 1, 1, 1, 2, 2, 3, 4, 7, 8];
    assert_eq!(binary_search(&|x| v[x] >= 1, v.len(), 0), 1);
    assert_eq!(binary_search(&|x| v[x] >= 2, v.len(), 0), 4);
    assert_eq!(binary_search(&|x| v[x] >= 3, v.len(), 0), 6);
    assert_eq!(binary_search(&|x| v[x] <= 1, 0, v.len()), 3);
    assert_eq!(binary_search(&|x| v[x] <= 2, 0, v.len()), 5);
    assert_eq!(binary_search(&|x| v[x] <= 3, 0, v.len()), 6);

    assert_eq!(
        binary_search(&|x: i64| v[x as usize] as i64 <= -1, -1, v.len() as i64),
        -1
    );

    let sq2 = binary_search(&|x| x * x <= 2., 1., 4.);
    let expect = 1.41421356273;
    assert!(expect - 1e-8 <= sq2 && sq2 <= expect + 1e-8);
}

#[test]
fn test_lower_bound() {
    let v = vec![0i64, 1, 1, 1, 2, 2, 3, 4, 7, 8];
    assert_eq!(lower_bound(&v, -1), 0);
    assert_eq!(lower_bound(&v, 0), 0);
    assert_eq!(lower_bound(&v, 1), 1);
    assert_eq!(lower_bound(&v, 2), 4);
    assert_eq!(lower_bound(&v, 3), 6);
}

#[test]
fn test_upper_bound() {
    let v = vec![0i64, 1, 1, 1, 2, 2, 3, 4, 7, 8];
    assert_eq!(upper_bound(&v, -1), 0);
    assert_eq!(upper_bound(&v, 0), 1);
    assert_eq!(upper_bound(&v, 1), 4);
    assert_eq!(upper_bound(&v, 2), 6);
    assert_eq!(upper_bound(&v, 3), 7);
}

#[snippet("ternary_search")]
pub trait Trisect: Copy {
    fn next(self, other: Self) -> (Self, Self);
    fn section_end(self, other: Self) -> bool;
}
#[snippet("ternary_search")]
impl Trisect for usize {
    fn next(self, other: Self) -> (Self, Self) {
        ((self * 2 + other) / 3, (self + other * 2) / 3)
    }
    fn section_end(self, other: Self) -> bool {
        (if self > other {
            self - other
        } else {
            other - self
        }) <= 1
    }
}
#[snippet("ternary_search")]
impl Trisect for i64 {
    fn next(self, other: Self) -> (Self, Self) {
        ((self * 2 + other) / 3, (self + other * 2) / 3)
    }
    fn section_end(self, other: Self) -> bool {
        (self - other).abs() <= 1
    }
}
#[snippet("ternary_search")]
impl Trisect for f64 {
    fn next(self, other: Self) -> (Self, Self) {
        ((self * 2. + other) / 3., (self + other * 2.) / 3.)
    }
    fn section_end(self, other: Self) -> bool {
        (self - other).abs() <= 1e-8
    }
}
#[snippet("ternary_search")]
pub fn ternary_search<T: Trisect, F: Fn(T) -> U, U: PartialOrd>(f: F, left: T, right: T) -> T {
    let mut left = left;
    let mut right = right;
    while !left.section_end(right) {
        let (l, r) = left.next(right);
        if f(l) > f(r) {
            left = l;
        } else {
            right = r;
        }
    }
    left
}

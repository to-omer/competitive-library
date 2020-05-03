#[cargo_snippet::snippet("Modi64")]
const MOD: i64 = 1_000_000_007;
#[cargo_snippet::snippet("Modi64")]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modi64(pub i64);
#[cargo_snippet::snippet("Modi64")]
impl Modi64 {
    #[inline]
    pub fn new(i: i64) -> Self {
        Modi64(Self::modulo(i, MOD))
    }
    #[inline]
    pub fn modulo(i: i64, m: i64) -> i64 {
        let mut x = i % m;
        if x < 0 {
            x += m;
        }
        x
    }
    pub fn pow(self, y: i64) -> Self {
        let mut y = Self::modulo(y, MOD - 1);
        let mut x = Self::new(1);
        let mut base = self;
        while y > 0 {
            if y & 1 == 1 {
                x *= base;
            }
            base *= base;
            y = y >> 1;
        }
        x
    }
    #[inline]
    pub fn inv(self) -> Self {
        let (mut x, mut s, mut t, mut u) = (1, self.0, MOD, 0);
        while t != 0 {
            let k = s / t;
            s -= k * t;
            std::mem::swap(&mut s, &mut t);
            x -= k * u;
            std::mem::swap(&mut x, &mut u);
        }
        Modi64::new(x)
    }
}
#[cargo_snippet::snippet("Modi64")]
impl From<i64> for Modi64 {
    fn from(i: i64) -> Self {
        Self::new(i)
    }
}
#[cargo_snippet::snippet("Modi64")]
impl std::ops::Add for Modi64 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Modi64((self.0 + rhs.0) % MOD)
    }
}
#[cargo_snippet::snippet("Modi64")]
impl<'a> std::ops::Add<Modi64> for &'a Modi64 {
    type Output = Modi64;
    #[inline]
    fn add(self, rhs: Modi64) -> Self::Output {
        *self + rhs
    }
}
#[cargo_snippet::snippet("Modi64")]
impl<'a> std::ops::Add<&'a Modi64> for Modi64 {
    type Output = Modi64;
    #[inline]
    fn add(self, rhs: &'a Modi64) -> Self::Output {
        self + *rhs
    }
}
#[cargo_snippet::snippet("Modi64")]
impl<'a> std::ops::Add for &'a Modi64 {
    type Output = Modi64;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        *self + *rhs
    }
}
#[cargo_snippet::snippet("Modi64")]
impl std::ops::AddAssign for Modi64 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
#[cargo_snippet::snippet("Modi64")]
impl<'a> std::ops::AddAssign<&'a Modi64> for Modi64 {
    #[inline]
    fn add_assign(&mut self, rhs: &'a Modi64) {
        *self += *rhs;
    }
}
#[cargo_snippet::snippet("Modi64")]
impl std::ops::Sub for Modi64 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.0 - rhs.0)
    }
}
#[cargo_snippet::snippet("Modi64")]
impl<'a> std::ops::Sub<Modi64> for &'a Modi64 {
    type Output = Modi64;
    #[inline]
    fn sub(self, rhs: Modi64) -> Self::Output {
        *self - rhs
    }
}
#[cargo_snippet::snippet("Modi64")]
impl<'a> std::ops::Sub<&'a Modi64> for Modi64 {
    type Output = Modi64;
    #[inline]
    fn sub(self, rhs: &'a Modi64) -> Self::Output {
        self - *rhs
    }
}
#[cargo_snippet::snippet("Modi64")]
impl<'a> std::ops::Sub for &'a Modi64 {
    type Output = Modi64;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        *self - *rhs
    }
}
#[cargo_snippet::snippet("Modi64")]
impl std::ops::SubAssign for Modi64 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
#[cargo_snippet::snippet("Modi64")]
impl<'a> std::ops::SubAssign<&'a Modi64> for Modi64 {
    #[inline]
    fn sub_assign(&mut self, rhs: &'a Modi64) {
        *self -= *rhs;
    }
}
#[cargo_snippet::snippet("Modi64")]
impl std::ops::Mul for Modi64 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Modi64(self.0 * rhs.0 % MOD)
    }
}
#[cargo_snippet::snippet("Modi64")]
impl<'a> std::ops::Mul<Modi64> for &'a Modi64 {
    type Output = Modi64;
    #[inline]
    fn mul(self, rhs: Modi64) -> Self::Output {
        *self * rhs
    }
}
#[cargo_snippet::snippet("Modi64")]
impl<'a> std::ops::Mul<&'a Modi64> for Modi64 {
    type Output = Modi64;
    #[inline]
    fn mul(self, rhs: &'a Modi64) -> Self::Output {
        self * *rhs
    }
}
#[cargo_snippet::snippet("Modi64")]
impl<'a> std::ops::Mul for &'a Modi64 {
    type Output = Modi64;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        *self * *rhs
    }
}
#[cargo_snippet::snippet("Modi64")]
impl std::ops::MulAssign for Modi64 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}
#[cargo_snippet::snippet("Modi64")]
impl<'a> std::ops::MulAssign<&'a Modi64> for Modi64 {
    #[inline]
    fn mul_assign(&mut self, rhs: &'a Modi64) {
        *self *= *rhs;
    }
}
#[cargo_snippet::snippet("Modi64")]
impl std::ops::Div for Modi64 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        self * rhs.inv()
    }
}
#[cargo_snippet::snippet("Modi64")]
impl<'a> std::ops::Div<Modi64> for &'a Modi64 {
    type Output = Modi64;
    #[inline]
    fn div(self, rhs: Modi64) -> Self::Output {
        *self * rhs.inv()
    }
}
#[cargo_snippet::snippet("Modi64")]
impl<'a> std::ops::Div<&'a Modi64> for Modi64 {
    type Output = Modi64;
    #[inline]
    fn div(self, rhs: &'a Modi64) -> Self::Output {
        self * rhs.inv()
    }
}
#[cargo_snippet::snippet("Modi64")]
impl<'a> std::ops::Div for &'a Modi64 {
    type Output = Modi64;
    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        *self * rhs.inv()
    }
}
#[cargo_snippet::snippet("Modi64")]
impl std::ops::DivAssign for Modi64 {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self * rhs.inv();
    }
}
#[cargo_snippet::snippet("Modi64")]
impl<'a> std::ops::DivAssign<&'a Modi64> for Modi64 {
    #[inline]
    fn div_assign(&mut self, rhs: &'a Modi64) {
        *self *= rhs.inv();
    }
}
#[cargo_snippet::snippet("Modi64")]
impl std::ops::Neg for Modi64 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        Self::new(-self.0)
    }
}
#[cargo_snippet::snippet("Modi64")]
impl<'a> std::ops::Neg for &'a Modi64 {
    type Output = Modi64;
    #[inline]
    fn neg(self) -> Self::Output {
        -*self
    }
}
#[cargo_snippet::snippet("Modi64")]
impl std::iter::Sum for Modi64 {
    #[inline]
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::new(0), std::ops::Add::add)
    }
}
#[cargo_snippet::snippet("Modi64")]
impl<'a> std::iter::Sum<&'a Modi64> for Modi64 {
    #[inline]
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::new(0), |x, &y| x + y)
    }
}
#[cargo_snippet::snippet("Modi64")]
impl std::fmt::Display for Modi64 {
    fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}
#[cargo_snippet::snippet("Modi64")]
impl std::str::FromStr for Modi64 {
    type Err = std::num::ParseIntError;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<i64>().map(|i| Modi64::new(i))
    }
}

#[test]
fn test_modi() {
    let mut x = Modi64(0);
    x -= Modi64::new(1);
    assert_eq!(x.0, MOD - 1);
    x += Modi64::new(3);
    assert_eq!(x.0, 2);
    assert_eq!(x.pow(MOD), x);
    assert_eq!(Modi64::new(2).pow(20), Modi64::new(2i64.pow(20)));
    assert_eq!(x / Modi64::new(10000) * Modi64::new(10000), x);
}

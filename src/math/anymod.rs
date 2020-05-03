#[cargo_snippet::snippet("AnyMod")]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct AnyMod {
    x: i64,
    m: i64,
}
#[cargo_snippet::snippet("AnyMod")]
impl AnyMod {
    #[inline]
    pub fn new(x: i64, m: i64) -> Self {
        AnyMod {
            x: Self::modulo(x, m),
            m: m,
        }
    }
    #[inline]
    pub fn modulo(x: i64, m: i64) -> i64 {
        let mut x = x % m;
        if x < 0 {
            x += m;
        }
        x
    }
    pub fn pow(self, y: usize) -> Self {
        let mut x = 1i64;
        let mut y = y;
        let mut base = self.x;
        while y > 0 {
            if y & 1 == 1 {
                x = x * base % self.m;
            }
            base = base * base % self.m;
            y = y >> 1;
        }
        AnyMod { x: x, m: self.m }
    }
    #[inline]
    pub fn inv(self) -> Self {
        let (mut x, mut s, mut t, mut u) = (1, self.x, self.m, 0);
        while t != 0 {
            let k = s / t;
            s -= k * t;
            std::mem::swap(&mut s, &mut t);
            x -= k * u;
            std::mem::swap(&mut x, &mut u);
        }
        AnyMod::new(x, self.m)
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl std::ops::Add for AnyMod {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        AnyMod {
            x: (self.x + rhs.x) % self.m,
            m: self.m,
        }
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl<'a> std::ops::Add<AnyMod> for &'a AnyMod {
    type Output = AnyMod;
    #[inline]
    fn add(self, rhs: AnyMod) -> Self::Output {
        *self + rhs
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl<'a> std::ops::Add<&'a AnyMod> for AnyMod {
    type Output = AnyMod;
    #[inline]
    fn add(self, rhs: &'a AnyMod) -> Self::Output {
        self + *rhs
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl<'a> std::ops::Add for &'a AnyMod {
    type Output = AnyMod;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        *self + *rhs
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl std::ops::AddAssign for AnyMod {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl<'a> std::ops::AddAssign<&'a AnyMod> for AnyMod {
    #[inline]
    fn add_assign(&mut self, rhs: &'a AnyMod) {
        *self += *rhs;
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl std::ops::Sub for AnyMod {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.m)
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl<'a> std::ops::Sub<AnyMod> for &'a AnyMod {
    type Output = AnyMod;
    #[inline]
    fn sub(self, rhs: AnyMod) -> Self::Output {
        *self - rhs
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl<'a> std::ops::Sub<&'a AnyMod> for AnyMod {
    type Output = AnyMod;
    #[inline]
    fn sub(self, rhs: &'a AnyMod) -> Self::Output {
        self - *rhs
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl<'a> std::ops::Sub for &'a AnyMod {
    type Output = AnyMod;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        *self - *rhs
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl std::ops::SubAssign for AnyMod {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl<'a> std::ops::SubAssign<&'a AnyMod> for AnyMod {
    #[inline]
    fn sub_assign(&mut self, rhs: &'a AnyMod) {
        *self -= *rhs;
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl std::ops::Mul for AnyMod {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        AnyMod {
            x: (self.x * rhs.x) % self.m,
            m: self.m,
        }
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl<'a> std::ops::Mul<AnyMod> for &'a AnyMod {
    type Output = AnyMod;
    #[inline]
    fn mul(self, rhs: AnyMod) -> Self::Output {
        *self * rhs
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl<'a> std::ops::Mul<&'a AnyMod> for AnyMod {
    type Output = AnyMod;
    #[inline]
    fn mul(self, rhs: &'a AnyMod) -> Self::Output {
        self * *rhs
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl<'a> std::ops::Mul for &'a AnyMod {
    type Output = AnyMod;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        *self * *rhs
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl std::ops::MulAssign for AnyMod {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl<'a> std::ops::MulAssign<&'a AnyMod> for AnyMod {
    #[inline]
    fn mul_assign(&mut self, rhs: &'a AnyMod) {
        *self *= *rhs;
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl std::ops::Div for AnyMod {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        self * rhs.inv()
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl<'a> std::ops::Div<AnyMod> for &'a AnyMod {
    type Output = AnyMod;
    #[inline]
    fn div(self, rhs: AnyMod) -> Self::Output {
        *self * rhs.inv()
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl<'a> std::ops::Div<&'a AnyMod> for AnyMod {
    type Output = AnyMod;
    #[inline]
    fn div(self, rhs: &'a AnyMod) -> Self::Output {
        self * rhs.inv()
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl<'a> std::ops::Div for &'a AnyMod {
    type Output = AnyMod;
    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        *self * rhs.inv()
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl std::ops::DivAssign for AnyMod {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self * rhs.inv();
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl<'a> std::ops::DivAssign<&'a AnyMod> for AnyMod {
    #[inline]
    fn div_assign(&mut self, rhs: &'a AnyMod) {
        *self *= rhs.inv();
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl std::ops::Neg for AnyMod {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        Self::new(-self.x, self.m)
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl<'a> std::ops::Neg for &'a AnyMod {
    type Output = AnyMod;
    #[inline]
    fn neg(self) -> Self::Output {
        -*self
    }
}
#[cargo_snippet::snippet("AnyMod")]
impl std::fmt::Display for AnyMod {
    fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.x)
    }
}

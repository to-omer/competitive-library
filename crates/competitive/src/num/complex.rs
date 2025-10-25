use super::{Float, IterScan, One, Zero};
use std::{
    cmp::Ordering,
    iter::{Product, Sum},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Complex<T> {
    pub re: T,
    pub im: T,
}

impl<T> Complex<T> {
    pub fn new(re: T, im: T) -> Self {
        Self { re, im }
    }
    pub fn transpose(self) -> Self {
        Self::new(self.im, self.re)
    }
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> Complex<U> {
        Complex::new(f(self.re), f(self.im))
    }
}
impl<T> Zero for Complex<T>
where
    T: Zero,
{
    fn zero() -> Self {
        Self::new(T::zero(), T::zero())
    }
}
impl<T> One for Complex<T>
where
    T: Zero + One,
{
    fn one() -> Self {
        Self::new(T::one(), T::zero())
    }
}
impl<T> Complex<T>
where
    T: Zero + One,
{
    pub fn i() -> Self {
        Self::new(T::zero(), T::one())
    }
}
impl<T> Complex<T>
where
    T: Neg<Output = T>,
{
    pub fn conjugate(self) -> Self {
        Self::new(self.re, -self.im)
    }
}
impl<T> Complex<T>
where
    T: Mul,
    <T as Mul>::Output: Add,
{
    pub fn dot(self, rhs: Self) -> <<T as Mul>::Output as Add>::Output {
        self.re * rhs.re + self.im * rhs.im
    }
}
impl<T> Complex<T>
where
    T: Mul,
    <T as Mul>::Output: Sub,
{
    pub fn cross(self, rhs: Self) -> <<T as Mul>::Output as Sub>::Output {
        self.re * rhs.im - self.im * rhs.re
    }
}
impl<T> Complex<T>
where
    T: Mul + Clone,
    <T as Mul>::Output: Add,
{
    pub fn norm(self) -> <<T as Mul>::Output as Add>::Output {
        self.re.clone() * self.re + self.im.clone() * self.im
    }
}
impl<T> Complex<T>
where
    T: Zero + Ord + Mul,
    <T as Mul>::Output: Ord,
{
    pub fn cmp_by_arg(self, other: Self) -> Ordering {
        fn pos<T>(c: &Complex<T>) -> bool
        where
            T: Zero + Ord,
        {
            let zero = T::zero();
            c.im < zero || c.im <= zero && c.re < zero
        }
        pos(&self)
            .cmp(&pos(&other))
            .then_with(|| (self.re * other.im).cmp(&(self.im * other.re)).reverse())
    }
}
impl<T> Complex<T>
where
    T: Float,
{
    pub fn polar(r: T, theta: T) -> Self {
        Self::new(r * theta.cos(), r * theta.sin())
    }
    pub fn primitive_nth_root_of_unity(n: T) -> Self {
        let theta = T::TAU / n;
        Self::new(theta.cos(), theta.sin())
    }
    pub fn abs(self) -> T {
        self.re.hypot(self.im)
    }
    pub fn unit(self) -> Self {
        self / self.abs()
    }
    pub fn angle(self) -> T {
        self.im.atan2(self.re)
    }
}
impl<T> Add for Complex<T>
where
    T: Add,
{
    type Output = Complex<<T as Add>::Output>;
    fn add(self, rhs: Self) -> Self::Output {
        Complex::new(self.re + rhs.re, self.im + rhs.im)
    }
}
impl<T> Add<T> for Complex<T>
where
    T: Add<Output = T>,
{
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        Self::new(self.re + rhs, self.im)
    }
}
impl<T> Sub for Complex<T>
where
    T: Sub,
{
    type Output = Complex<<T as Sub>::Output>;
    fn sub(self, rhs: Self) -> Self::Output {
        Complex::new(self.re - rhs.re, self.im - rhs.im)
    }
}
impl<T> Sub<T> for Complex<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;
    fn sub(self, rhs: T) -> Self::Output {
        Self::new(self.re - rhs, self.im)
    }
}
impl<T, U> Mul for Complex<T>
where
    T: Clone + Mul,
    <T as Mul>::Output: Add<Output = U> + Sub<Output = U>,
{
    type Output = Complex<U>;
    fn mul(self, rhs: Self) -> Self::Output {
        Complex::new(
            self.re.clone() * rhs.re.clone() - self.im.clone() * rhs.im.clone(),
            self.re * rhs.im + self.im * rhs.re,
        )
    }
}
impl<T> Mul<T> for Complex<T>
where
    T: Clone + Mul,
{
    type Output = Complex<<T as Mul>::Output>;
    fn mul(self, rhs: T) -> Self::Output {
        Complex::new(self.re * rhs.clone(), self.im * rhs)
    }
}
impl<T> Div for Complex<T>
where
    T: Clone + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div,
{
    type Output = Complex<<T as Div>::Output>;
    fn div(self, rhs: Self) -> Self::Output {
        let d = rhs.re.clone() * rhs.re.clone() + rhs.im.clone() * rhs.im.clone();
        Complex::new(
            (self.re.clone() * rhs.re.clone() + self.im.clone() * rhs.im.clone()) / d.clone(),
            (self.im * rhs.re - self.re * rhs.im) / d,
        )
    }
}
impl<T> Div<T> for Complex<T>
where
    T: Clone + Div,
{
    type Output = Complex<<T as Div>::Output>;
    fn div(self, rhs: T) -> Self::Output {
        Complex::new(self.re / rhs.clone(), self.im / rhs)
    }
}
impl<T> Neg for Complex<T>
where
    T: Neg,
{
    type Output = Complex<<T as Neg>::Output>;
    fn neg(self) -> Self::Output {
        Complex::new(-self.re, -self.im)
    }
}
macro_rules! impl_complex_ref_binop {
    (impl<$T:ident> $imp:ident $method:ident ($l:ty, $r:ty) where $($w:ident)*) => {
        impl<$T> $imp<$r> for &$l
        where
            $T: Clone $(+ $w<Output = $T>)*,
        {
            type Output = <$l as $imp<$r>>::Output;
            fn $method(self, rhs: $r) -> <$l as $imp<$r>>::Output {
                $imp::$method(self.clone(), rhs)
            }
        }
        impl<$T> $imp<&$r> for $l
        where
            $T: Clone $(+ $w<Output = $T>)*,
        {
            type Output = <$l as $imp<$r>>::Output;
            fn $method(self, rhs: &$r) -> <$l as $imp<$r>>::Output {
                $imp::$method(self, rhs.clone())
            }
        }
        impl<$T> $imp<&$r> for &$l
        where
            $T: Clone $(+ $w<Output = $T>)*,
        {
            type Output = <$l as $imp<$r>>::Output;
            fn $method(self, rhs: &$r) -> <$l as $imp<$r>>::Output {
                $imp::$method(self.clone(), rhs.clone())
            }
        }
    };
}
impl_complex_ref_binop!(impl<T> Add add (Complex<T>, Complex<T>) where Add);
impl_complex_ref_binop!(impl<T> Add add (Complex<T>, T) where Add);
impl_complex_ref_binop!(impl<T> Sub sub (Complex<T>, Complex<T>) where Sub);
impl_complex_ref_binop!(impl<T> Sub sub (Complex<T>, T) where Sub);
impl_complex_ref_binop!(impl<T> Mul mul (Complex<T>, Complex<T>) where Add Sub Mul);
impl_complex_ref_binop!(impl<T> Mul mul (Complex<T>, T) where Mul);
impl_complex_ref_binop!(impl<T> Div div (Complex<T>, Complex<T>) where Add Sub Mul Div);
impl_complex_ref_binop!(impl<T> Div div (Complex<T>, T) where Div);
macro_rules! impl_complex_ref_unop {
    (impl<$T:ident> $imp:ident $method:ident ($t:ty) where $($w:ident)*) => {
        impl<$T> $imp for &$t
        where
            $T: Clone $(+ $w<Output = $T>)*,
        {
            type Output = <$t as $imp>::Output;
            fn $method(self) -> <$t as $imp>::Output {
                $imp::$method(self.clone())
            }
        }
    };
}
impl_complex_ref_unop!(impl<T> Neg neg (Complex<T>) where Neg);
macro_rules! impl_complex_op_assign {
    (impl<$T:ident> $imp:ident $method:ident ($l:ty, $r:ty) $fromimp:ident $frommethod:ident where $($w:ident)*) => {
        impl<$T> $imp<$r> for $l
        where
            $T: Clone $(+ $w<Output = $T>)*,
        {
            fn $method(&mut self, rhs: $r) {
                *self = $fromimp::$frommethod(self.clone(), rhs);
            }
        }
        impl<$T> $imp<&$r> for $l
        where
            $T: Clone $(+ $w<Output = $T>)*,
        {
            fn $method(&mut self, rhs: &$r) {
                $imp::$method(self, rhs.clone());
            }
        }
    };
}
impl_complex_op_assign!(impl<T> AddAssign add_assign (Complex<T>, Complex<T>) Add add where Add);
impl_complex_op_assign!(impl<T> AddAssign add_assign (Complex<T>, T) Add add where Add);
impl_complex_op_assign!(impl<T> SubAssign sub_assign (Complex<T>, Complex<T>) Sub sub where Sub);
impl_complex_op_assign!(impl<T> SubAssign sub_assign (Complex<T>, T) Sub sub where Sub);
impl_complex_op_assign!(impl<T> MulAssign mul_assign (Complex<T>, Complex<T>) Mul mul where Add Sub Mul);
impl_complex_op_assign!(impl<T> MulAssign mul_assign (Complex<T>, T) Mul mul where Mul);
impl_complex_op_assign!(impl<T> DivAssign div_assign (Complex<T>, Complex<T>) Div div where Add Sub Mul Div);
impl_complex_op_assign!(impl<T> DivAssign div_assign (Complex<T>, T) Div div where Div);
macro_rules! impl_complex_fold {
    (impl<$T:ident> $imp:ident $method:ident ($t:ty) $identimp:ident $identmethod:ident $fromimp:ident $frommethod:ident where $($w:ident)* $(+ $x:ident)*) => {
        impl<$T> $imp for $t
        where
            $T: $identimp $(+ $w<Output = $T>)* $(+ $x)*,
        {
            fn $method<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold(<Self as $identimp>::$identmethod(), $fromimp::$frommethod)
            }
        }
        impl<'a, $T: 'a> $imp<&'a $t> for $t
        where
            $T: Clone + $identimp $(+ $w<Output = $T>)* $(+ $x)*,
        {
            fn $method<I: Iterator<Item = &'a $t>>(iter: I) -> Self {
                iter.fold(<Self as $identimp>::$identmethod(), $fromimp::$frommethod)
            }
        }
    };
}
impl_complex_fold!(impl<T> Sum sum (Complex<T>) Zero zero Add add where Add);
impl_complex_fold!(impl<T> Product product (Complex<T>) One one Mul mul where Add Sub Mul + Zero + Clone);

impl<T: IterScan> IterScan for Complex<T> {
    type Output = Complex<<T as IterScan>::Output>;
    fn scan<'a, I: Iterator<Item = &'a str>>(iter: &mut I) -> Option<Self::Output> {
        Some(Complex::new(
            <T as IterScan>::scan(iter)?,
            <T as IterScan>::scan(iter)?,
        ))
    }
}

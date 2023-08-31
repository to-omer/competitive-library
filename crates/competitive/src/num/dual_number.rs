use super::{One, Zero};
use std::{
    iter::{Product, Sum},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DualNumber<T>(pub T, pub T);

impl<T> DualNumber<T> {
    pub fn transpose(self) -> Self {
        Self(self.1, self.0)
    }
}
impl<T> Zero for DualNumber<T>
where
    T: Zero,
{
    fn zero() -> Self {
        Self(T::zero(), T::zero())
    }
}
impl<T> One for DualNumber<T>
where
    T: Zero + One,
{
    fn one() -> Self {
        Self(T::one(), T::zero())
    }
}
impl<T> DualNumber<T>
where
    T: Zero + One,
{
    pub fn epsilon() -> Self {
        Self(T::zero(), T::one())
    }
}
impl<T> DualNumber<T>
where
    T: Neg<Output = T>,
{
    pub fn conjugate(self) -> Self {
        Self(self.0, -self.1)
    }
}
impl<T> DualNumber<T>
where
    T: Add<Output = T> + Mul<Output = T>,
{
    pub fn eval(self, eps: T) -> T {
        self.0 + self.1 * eps
    }
}
impl<T> DualNumber<T>
where
    T: Div<Output = T> + Neg<Output = T>,
{
    pub fn root(self) -> T {
        -self.0 / self.1
    }
}

impl<T> Add for DualNumber<T>
where
    T: Add<Output = T>,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}
impl<T> Add<T> for DualNumber<T>
where
    T: Add<Output = T>,
{
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        Self(self.0 + rhs, self.1)
    }
}
impl<T> Sub for DualNumber<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}
impl<T> Sub<T> for DualNumber<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;
    fn sub(self, rhs: T) -> Self::Output {
        Self(self.0 - rhs, self.1)
    }
}
impl<T> Mul for DualNumber<T>
where
    T: Clone + Add<Output = T> + Sub<Output = T> + Mul<Output = T>,
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(
            self.0.clone() * rhs.0.clone(),
            self.0 * rhs.1 + self.1 * rhs.0,
        )
    }
}
impl<T> Mul<T> for DualNumber<T>
where
    T: Clone + Mul<Output = T>,
{
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        Self(self.0 * rhs.clone(), self.1 * rhs)
    }
}
impl<T> Div for DualNumber<T>
where
    T: Clone + One + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T>,
{
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        let d = T::one() / rhs.1.clone();
        Self(
            self.0.clone() * d.clone(),
            (self.1 * rhs.0 - self.0 * rhs.1) * d.clone() * d,
        )
    }
}
impl<T> Div<T> for DualNumber<T>
where
    T: Clone + One + Div<Output = T>,
{
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        let d = T::one() / rhs.clone();
        Self(self.0 / d.clone(), self.1 / d)
    }
}
impl<T> Neg for DualNumber<T>
where
    T: Neg<Output = T>,
{
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(-self.0, -self.1)
    }
}
macro_rules! impl_dual_number_ref_binop {
    (impl<$T:ident> $imp:ident $method:ident ($l:ty, $r:ty) where $($w:ident)* $(+ $($v:ident)*)?) => {
        impl<$T> $imp<$r> for &$l
        where
            $T: Clone $(+ $w<Output = $T>)* $($(+ $v)*)?,
        {
            type Output = <$l as $imp<$r>>::Output;
            fn $method(self, rhs: $r) -> <$l as $imp<$r>>::Output {
                $imp::$method(self.clone(), rhs)
            }
        }
        impl<$T> $imp<&$r> for $l
        where
            $T: Clone $(+ $w<Output = $T>)* $($(+ $v)*)?,
        {
            type Output = <$l as $imp<$r>>::Output;
            fn $method(self, rhs: &$r) -> <$l as $imp<$r>>::Output {
                $imp::$method(self, rhs.clone())
            }
        }
        impl<$T> $imp<&$r> for &$l
        where
            $T: Clone $(+ $w<Output = $T>)* $($(+ $v)*)?,
        {
            type Output = <$l as $imp<$r>>::Output;
            fn $method(self, rhs: &$r) -> <$l as $imp<$r>>::Output {
                $imp::$method(self.clone(), rhs.clone())
            }
        }
    };
}
impl_dual_number_ref_binop!(impl<T> Add add (DualNumber<T>, DualNumber<T>) where Add);
impl_dual_number_ref_binop!(impl<T> Add add (DualNumber<T>, T) where Add);
impl_dual_number_ref_binop!(impl<T> Sub sub (DualNumber<T>, DualNumber<T>) where Sub);
impl_dual_number_ref_binop!(impl<T> Sub sub (DualNumber<T>, T) where Sub);
impl_dual_number_ref_binop!(impl<T> Mul mul (DualNumber<T>, DualNumber<T>) where Add Sub Mul);
impl_dual_number_ref_binop!(impl<T> Mul mul (DualNumber<T>, T) where Mul);
impl_dual_number_ref_binop!(impl<T> Div div (DualNumber<T>, DualNumber<T>) where Add Sub Mul Div + One);
impl_dual_number_ref_binop!(impl<T> Div div (DualNumber<T>, T) where Div + One);
macro_rules! impl_dual_number_ref_unop {
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
impl_dual_number_ref_unop!(impl<T> Neg neg (DualNumber<T>) where Neg);
macro_rules! impl_dual_number_op_assign {
    (impl<$T:ident> $imp:ident $method:ident ($l:ty, $r:ty) $fromimp:ident $frommethod:ident where $($w:ident)* $(+ $($v:ident)*)?) => {
        impl<$T> $imp<$r> for $l
        where
            $T: Clone $(+ $w<Output = $T>)* $($(+ $v)*)?,
        {
            fn $method(&mut self, rhs: $r) {
                *self = $fromimp::$frommethod(self.clone(), rhs);
            }
        }
        impl<$T> $imp<&$r> for $l
        where
            $T: Clone $(+ $w<Output = $T>)* $($(+ $v)*)?,
        {
            fn $method(&mut self, rhs: &$r) {
                $imp::$method(self, rhs.clone());
            }
        }
    };
}
impl_dual_number_op_assign!(impl<T> AddAssign add_assign (DualNumber<T>, DualNumber<T>) Add add where Add);
impl_dual_number_op_assign!(impl<T> AddAssign add_assign (DualNumber<T>, T) Add add where Add);
impl_dual_number_op_assign!(impl<T> SubAssign sub_assign (DualNumber<T>, DualNumber<T>) Sub sub where Sub);
impl_dual_number_op_assign!(impl<T> SubAssign sub_assign (DualNumber<T>, T) Sub sub where Sub);
impl_dual_number_op_assign!(impl<T> MulAssign mul_assign (DualNumber<T>, DualNumber<T>) Mul mul where Add Sub Mul);
impl_dual_number_op_assign!(impl<T> MulAssign mul_assign (DualNumber<T>, T) Mul mul where Mul);
impl_dual_number_op_assign!(impl<T> DivAssign div_assign (DualNumber<T>, DualNumber<T>) Div div where Add Sub Mul Div + One);
impl_dual_number_op_assign!(impl<T> DivAssign div_assign (DualNumber<T>, T) Div div where Div + One);
macro_rules! impl_dual_number_fold {
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
impl_dual_number_fold!(impl<T> Sum sum (DualNumber<T>) Zero zero Add add where Add);
impl_dual_number_fold!(impl<T> Product product (DualNumber<T>) One one Mul mul where Add Sub Mul + Zero + Clone);

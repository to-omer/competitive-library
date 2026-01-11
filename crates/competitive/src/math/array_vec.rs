use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Index, IndexMut, Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub,
    SubAssign,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArrayVecScalar<T>(pub T);

impl<T> From<T> for ArrayVecScalar<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

pub trait ToArrayVecScalar: Sized {
    fn to_array_vec_scalar(self) -> ArrayVecScalar<Self>;
}

impl<T> ToArrayVecScalar for T {
    fn to_array_vec_scalar(self) -> ArrayVecScalar<Self> {
        ArrayVecScalar(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArrayVec<T, const N: usize>(pub [T; N]);

pub trait ToArrayVec<T, const N: usize>: Sized {
    fn to_array_vec(self) -> ArrayVec<T, N>;
}

impl<T, const N: usize> ToArrayVec<T, N> for [T; N] {
    fn to_array_vec(self) -> ArrayVec<T, N> {
        ArrayVec(self)
    }
}

impl<T, const N: usize> Default for ArrayVec<T, N>
where
    T: Default,
{
    fn default() -> Self {
        Self(std::array::from_fn(|_| T::default()))
    }
}

impl<T, const N: usize> ArrayVec<T, N> {
    pub fn new(data: [T; N]) -> Self {
        Self(data)
    }

    pub fn map<U>(&self, transform: impl FnMut(&T) -> U) -> ArrayVec<U, N> {
        ArrayVec(array_from_iter(self.0.iter().map(transform)))
    }

    pub fn zip_with<U, V>(
        &self,
        other: &ArrayVec<U, N>,
        mut combine: impl FnMut(&T, &U) -> V,
    ) -> ArrayVec<V, N> {
        ArrayVec(array_from_iter(
            self.0
                .iter()
                .zip(other.0.iter())
                .map(|(left, right)| combine(left, right)),
        ))
    }
}

impl<T, const N: usize> From<[T; N]> for ArrayVec<T, N> {
    fn from(data: [T; N]) -> Self {
        Self(data)
    }
}

impl<T, const N: usize> From<ArrayVec<T, N>> for [T; N] {
    fn from(data: ArrayVec<T, N>) -> Self {
        data.0
    }
}

impl<T, const N: usize> AsRef<[T; N]> for ArrayVec<T, N> {
    fn as_ref(&self) -> &[T; N] {
        &self.0
    }
}

impl<T, const N: usize> AsMut<[T; N]> for ArrayVec<T, N> {
    fn as_mut(&mut self) -> &mut [T; N] {
        &mut self.0
    }
}

impl<T, const N: usize> Index<usize> for ArrayVec<T, N> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T, const N: usize> IndexMut<usize> for ArrayVec<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[inline]
fn array_from_iter<T, I, const N: usize>(mut iter: I) -> [T; N]
where
    I: Iterator<Item = T>,
{
    std::array::from_fn(|_| iter.next().unwrap())
}

macro_rules! impl_arrayvec_binop {
    ($imp:ident, $method:ident, $op:tt) => {
        impl<T, U, V, const N: usize> $imp<ArrayVec<U, N>> for ArrayVec<T, N>
        where
            T: $imp<U, Output = V>,
        {
            type Output = ArrayVec<V, N>;
            fn $method(self, rhs: ArrayVec<U, N>) -> Self::Output {
                ArrayVec(array_from_iter(
                    self.0
                        .into_iter()
                        .zip(rhs.0.into_iter())
                        .map(|(left_value, right_value)| left_value $op right_value),
                ))
            }
        }
        impl<T, U, V, const N: usize> $imp<&ArrayVec<U, N>> for ArrayVec<T, N>
        where
            T: $imp<U, Output = V>,
            U: Clone,
        {
            type Output = ArrayVec<V, N>;
            fn $method(self, rhs: &ArrayVec<U, N>) -> Self::Output {
                $imp::$method(self, rhs.clone())
            }
        }
        impl<T, U, V, const N: usize> $imp<ArrayVec<U, N>> for &ArrayVec<T, N>
        where
            T: Clone + $imp<U, Output = V>,
        {
            type Output = ArrayVec<V, N>;
            fn $method(self, rhs: ArrayVec<U, N>) -> Self::Output {
                $imp::$method(self.clone(), rhs)
            }
        }
        impl<T, U, V, const N: usize> $imp<&ArrayVec<U, N>> for &ArrayVec<T, N>
        where
            T: Clone + $imp<U, Output = V>,
            U: Clone,
        {
            type Output = ArrayVec<V, N>;
            fn $method(self, rhs: &ArrayVec<U, N>) -> Self::Output {
                $imp::$method(self.clone(), rhs.clone())
            }
        }

        impl<T, U, V, const N: usize> $imp<ArrayVecScalar<U>> for ArrayVec<T, N>
        where
            T: $imp<U, Output = V>,
            U: Clone,
        {
            type Output = ArrayVec<V, N>;
            fn $method(self, rhs: ArrayVecScalar<U>) -> Self::Output {
                let scalar_value = rhs.0;
                ArrayVec(array_from_iter(
                    self.0
                        .into_iter()
                        .map(|value| value $op scalar_value.clone()),
                ))
            }
        }
        impl<T, U, V, const N: usize> $imp<&ArrayVecScalar<U>> for ArrayVec<T, N>
        where
            T: $imp<U, Output = V>,
            U: Clone,
        {
            type Output = ArrayVec<V, N>;
            fn $method(self, rhs: &ArrayVecScalar<U>) -> Self::Output {
                $imp::$method(self, rhs.clone())
            }
        }
        impl<T, U, V, const N: usize> $imp<ArrayVecScalar<U>> for &ArrayVec<T, N>
        where
            T: Clone + $imp<U, Output = V>,
            U: Clone,
        {
            type Output = ArrayVec<V, N>;
            fn $method(self, rhs: ArrayVecScalar<U>) -> Self::Output {
                $imp::$method(self.clone(), rhs)
            }
        }
        impl<T, U, V, const N: usize> $imp<&ArrayVecScalar<U>> for &ArrayVec<T, N>
        where
            T: Clone + $imp<U, Output = V>,
            U: Clone,
        {
            type Output = ArrayVec<V, N>;
            fn $method(self, rhs: &ArrayVecScalar<U>) -> Self::Output {
                $imp::$method(self.clone(), rhs.clone())
            }
        }

        impl<T, U, V, const N: usize> $imp<ArrayVec<T, N>> for ArrayVecScalar<U>
        where
            U: Clone + $imp<T, Output = V>,
        {
            type Output = ArrayVec<V, N>;
            fn $method(self, rhs: ArrayVec<T, N>) -> Self::Output {
                let scalar_value = self.0;
                ArrayVec(array_from_iter(
                    rhs.0
                        .into_iter()
                        .map(|value| scalar_value.clone() $op value),
                ))
            }
        }
        impl<T, U, V, const N: usize> $imp<&ArrayVec<T, N>> for ArrayVecScalar<U>
        where
            U: Clone + $imp<T, Output = V>,
            T: Clone,
        {
            type Output = ArrayVec<V, N>;
            fn $method(self, rhs: &ArrayVec<T, N>) -> Self::Output {
                $imp::$method(self, rhs.clone())
            }
        }
        impl<T, U, V, const N: usize> $imp<ArrayVec<T, N>> for &ArrayVecScalar<U>
        where
            U: Clone + $imp<T, Output = V>,
        {
            type Output = ArrayVec<V, N>;
            fn $method(self, rhs: ArrayVec<T, N>) -> Self::Output {
                $imp::$method(self.clone(), rhs)
            }
        }
        impl<T, U, V, const N: usize> $imp<&ArrayVec<T, N>> for &ArrayVecScalar<U>
        where
            U: Clone + $imp<T, Output = V>,
            T: Clone,
        {
            type Output = ArrayVec<V, N>;
            fn $method(self, rhs: &ArrayVec<T, N>) -> Self::Output {
                $imp::$method(self.clone(), rhs.clone())
            }
        }
    };
}

macro_rules! impl_arrayvec_unop {
    ($imp:ident, $method:ident, $op:tt) => {
        impl<T, U, const N: usize> $imp for ArrayVec<T, N>
        where
            T: $imp<Output = U>,
        {
            type Output = ArrayVec<U, N>;
            fn $method(self) -> Self::Output {
                ArrayVec(array_from_iter(
                    self.0.into_iter().map(|value| $op value),
                ))
            }
        }
        impl<T, U, const N: usize> $imp for &ArrayVec<T, N>
        where
            T: Clone + $imp<Output = U>,
        {
            type Output = ArrayVec<U, N>;
            fn $method(self) -> Self::Output {
                $imp::$method(self.clone())
            }
        }
    };
}

macro_rules! impl_arrayvec_assign {
    ($imp:ident, $method:ident) => {
        impl<T, U, const N: usize> $imp<ArrayVec<U, N>> for ArrayVec<T, N>
        where
            T: $imp<U>,
        {
            fn $method(&mut self, rhs: ArrayVec<U, N>) {
                for (left_value, right_value) in self.0.iter_mut().zip(rhs.0.into_iter()) {
                    left_value.$method(right_value);
                }
            }
        }
        impl<T, U, const N: usize> $imp<&ArrayVec<U, N>> for ArrayVec<T, N>
        where
            T: $imp<U>,
            U: Clone,
        {
            fn $method(&mut self, rhs: &ArrayVec<U, N>) {
                for (left_value, right_value) in self.0.iter_mut().zip(rhs.0.iter()) {
                    left_value.$method(right_value.clone());
                }
            }
        }
        impl<T, U, const N: usize> $imp<ArrayVecScalar<U>> for ArrayVec<T, N>
        where
            T: $imp<U>,
            U: Clone,
        {
            fn $method(&mut self, rhs: ArrayVecScalar<U>) {
                let scalar_value = rhs.0;
                for value in self.0.iter_mut() {
                    value.$method(scalar_value.clone());
                }
            }
        }
        impl<T, U, const N: usize> $imp<&ArrayVecScalar<U>> for ArrayVec<T, N>
        where
            T: $imp<U>,
            U: Clone,
        {
            fn $method(&mut self, rhs: &ArrayVecScalar<U>) {
                self.$method(rhs.clone());
            }
        }
    };
}

impl_arrayvec_binop!(Add, add, +);
impl_arrayvec_binop!(Sub, sub, -);
impl_arrayvec_binop!(Mul, mul, *);
impl_arrayvec_binop!(Div, div, /);
impl_arrayvec_binop!(Rem, rem, %);
impl_arrayvec_binop!(BitAnd, bitand, &);
impl_arrayvec_binop!(BitOr, bitor, |);
impl_arrayvec_binop!(BitXor, bitxor, ^);
impl_arrayvec_binop!(Shl, shl, <<);
impl_arrayvec_binop!(Shr, shr, >>);

impl_arrayvec_unop!(Neg, neg, -);
impl_arrayvec_unop!(Not, not, !);

impl_arrayvec_assign!(AddAssign, add_assign);
impl_arrayvec_assign!(SubAssign, sub_assign);
impl_arrayvec_assign!(MulAssign, mul_assign);
impl_arrayvec_assign!(DivAssign, div_assign);
impl_arrayvec_assign!(RemAssign, rem_assign);
impl_arrayvec_assign!(BitAndAssign, bitand_assign);
impl_arrayvec_assign!(BitOrAssign, bitor_assign);
impl_arrayvec_assign!(BitXorAssign, bitxor_assign);
impl_arrayvec_assign!(ShlAssign, shl_assign);
impl_arrayvec_assign!(ShrAssign, shr_assign);

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Add;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct LeftValue(i32);

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct RightValue(i32);

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct SumValue(i32);

    impl Add<RightValue> for LeftValue {
        type Output = SumValue;
        fn add(self, rhs: RightValue) -> Self::Output {
            SumValue(self.0 + rhs.0)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct ScalarValue(i32);

    impl Add<i32> for ScalarValue {
        type Output = i64;
        fn add(self, rhs: i32) -> Self::Output {
            self.0 as i64 + rhs as i64
        }
    }

    #[test]
    fn test_vec_vec_output_change() {
        let left = [LeftValue(1), LeftValue(2)].to_array_vec();
        let right = [RightValue(3), RightValue(4)].to_array_vec();
        let sum = left + right;
        assert_eq!(sum.0, [SumValue(4), SumValue(6)]);
    }

    #[test]
    fn test_vec_scalar_output_change() {
        let vector = [ScalarValue(1), ScalarValue(2)].to_array_vec();
        let output = vector + 3.to_array_vec_scalar();
        assert_eq!(output.0, [4i64, 5i64]);
    }

    #[test]
    fn test_binary_ops() {
        let left = [10i32, 20i32].to_array_vec();
        let right = [3i32, 4i32].to_array_vec();
        assert_eq!((left + right).0, [13, 24]);
        assert_eq!((left - right).0, [7, 16]);
        assert_eq!((left * 2.to_array_vec_scalar()).0, [20, 40]);
        assert_eq!((left / 2.to_array_vec_scalar()).0, [5, 10]);
        assert_eq!((left % 7.to_array_vec_scalar()).0, [3, 6]);
        assert_eq!((2.to_array_vec_scalar() * left).0, [20, 40]);
    }

    #[test]
    fn test_bit_and_shift_ops() {
        let vector = [0b1100u8, 0b1010u8].to_array_vec();
        let other = [0b1010u8, 0b1100u8].to_array_vec();
        assert_eq!((vector & other).0, [0b1000, 0b1000]);
        assert_eq!((vector | 0b0001.to_array_vec_scalar()).0, [0b1101, 0b1011]);
        assert_eq!((vector ^ other).0, [0b0110, 0b0110]);
        let shift_amounts = [1u32, 2u32].to_array_vec();
        assert_eq!((vector << shift_amounts).0, [0b11000, 0b101000]);
        assert_eq!((vector >> 1u32.to_array_vec_scalar()).0, [0b0110, 0b0101]);
    }

    #[test]
    fn test_assign_ops() {
        let mut values = [10i32, 20i32].to_array_vec();
        values += [1, 2].to_array_vec();
        values -= &[2, 3].to_array_vec();
        values *= 2.to_array_vec_scalar();
        values /= 3.to_array_vec_scalar();
        values %= 5.to_array_vec_scalar();
        assert_eq!(values.0, [1, 2]);

        let mut bits = [0b1100u8, 0b1010u8].to_array_vec();
        bits &= [0b1010u8, 0b1100u8].to_array_vec();
        bits |= &[0b0001u8, 0b0010u8].to_array_vec();
        bits ^= 0b0011.to_array_vec_scalar();
        bits <<= 1u32.to_array_vec_scalar();
        bits >>= 1u32.to_array_vec_scalar();
        assert_eq!(bits.0, [0b1010u8, 0b1001u8]);
    }
}

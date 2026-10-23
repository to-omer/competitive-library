use super::*;
use std::{
    cmp::Ordering,
    marker::PhantomData,
    ops::{Add, Mul, Sub},
};

pub type LazyMapAgg<L> = <<L as LazyMapMonoid>::AggMonoid as Magma>::T;
pub type LazyMapAct<L> = <<L as LazyMapMonoid>::ActMonoid as Magma>::T;

pub trait LazyMapMonoid {
    type Key;
    type AggMonoid: Monoid;
    type ActMonoid: Monoid<T: PartialEq>;
    fn single_agg(key: &Self::Key) -> LazyMapAgg<Self>;
    fn toggle(_x: &mut LazyMapAgg<Self>) {}
    #[inline]
    fn is_act_unit(act: &LazyMapAct<Self>) -> bool {
        <Self::ActMonoid as Unital>::is_unit(act)
    }
    /// Returns `None` when descent is needed; must succeed on single-key aggregates.
    fn act_agg(x: &LazyMapAgg<Self>, a: &LazyMapAct<Self>) -> Option<LazyMapAgg<Self>>;

    #[inline]
    fn agg_unit() -> LazyMapAgg<Self> {
        <Self::AggMonoid as Unital>::unit()
    }
    #[inline]
    fn act_unit() -> LazyMapAct<Self> {
        <Self::ActMonoid as Unital>::unit()
    }
    #[inline]
    fn agg_operate(x: &LazyMapAgg<Self>, y: &LazyMapAgg<Self>) -> LazyMapAgg<Self> {
        <Self::AggMonoid as Magma>::operate(x, y)
    }
    #[inline]
    fn act_operate(x: &LazyMapAct<Self>, y: &LazyMapAct<Self>) -> LazyMapAct<Self> {
        <Self::ActMonoid as Magma>::operate(x, y)
    }
    #[inline]
    fn agg_operate_assign(x: &mut LazyMapAgg<Self>, y: &LazyMapAgg<Self>) {
        *x = <Self::AggMonoid as Magma>::operate(x, y);
    }
    #[inline]
    fn act_operate_assign(x: &mut LazyMapAct<Self>, y: &LazyMapAct<Self>) {
        *x = <Self::ActMonoid as Magma>::operate(x, y);
    }
}

pub trait LazyMapMonoidWithKeyAct: LazyMapMonoid {
    fn act_key(key: &Self::Key, act: &LazyMapAct<Self>) -> Self::Key;
}

pub struct EmptyActLazy<M> {
    _marker: PhantomData<fn() -> M>,
}
impl<M> LazyMapMonoid for EmptyActLazy<M>
where
    M: Monoid,
{
    type Key = M::T;
    type AggMonoid = M;
    type ActMonoid = ();
    fn single_agg(key: &Self::Key) -> LazyMapAgg<Self> {
        key.clone()
    }
    fn act_agg(x: &LazyMapAgg<Self>, _a: &LazyMapAct<Self>) -> Option<LazyMapAgg<Self>> {
        Some(x.clone())
    }
}
impl<M> LazyMapMonoidWithKeyAct for EmptyActLazy<M>
where
    M: Monoid,
{
    fn act_key(key: &Self::Key, act: &LazyMapAct<Self>) -> Self::Key {
        EmptyAct::act(key, act)
    }
}

pub struct EmptyAggActLazy<T> {
    _marker: PhantomData<fn() -> T>,
}
impl<T> LazyMapMonoid for EmptyAggActLazy<T> {
    type Key = T;
    type AggMonoid = ();
    type ActMonoid = ();
    fn single_agg(_key: &Self::Key) -> LazyMapAgg<Self> {}
    fn act_agg(_x: &LazyMapAgg<Self>, _a: &LazyMapAct<Self>) -> Option<LazyMapAgg<Self>> {
        Some(())
    }
}
impl<T> LazyMapMonoidWithKeyAct for EmptyAggActLazy<T>
where
    T: Clone,
{
    fn act_key(key: &Self::Key, act: &LazyMapAct<Self>) -> Self::Key {
        EmptyAct::act(key, act)
    }
}

pub struct FlattenLazy<M> {
    _marker: PhantomData<fn() -> M>,
}
impl<M> LazyMapMonoid for FlattenLazy<M>
where
    M: Monoid,
    M::T: PartialEq,
{
    type Key = M::T;
    type AggMonoid = M;
    type ActMonoid = M;
    fn single_agg(key: &Self::Key) -> LazyMapAgg<Self> {
        key.clone()
    }
    fn act_agg(x: &LazyMapAgg<Self>, a: &LazyMapAct<Self>) -> Option<LazyMapAgg<Self>> {
        Some(M::operate(x, a))
    }
}
impl<M> LazyMapMonoidWithKeyAct for FlattenLazy<M>
where
    M: Monoid,
    M::T: PartialEq,
{
    fn act_key(key: &Self::Key, act: &LazyMapAct<Self>) -> Self::Key {
        FlattenAct::<M>::act(key, act)
    }
}

pub struct RangeSumRangeAdd<T> {
    _marker: PhantomData<fn() -> T>,
}
impl<T> LazyMapMonoid for RangeSumRangeAdd<T>
where
    T: Copy + Zero + One + Add<Output = T> + Mul<Output = T> + PartialEq,
{
    type Key = T;
    type AggMonoid = (AdditiveOperation<T>, AdditiveOperation<T>);
    type ActMonoid = AdditiveOperation<T>;
    fn single_agg(key: &Self::Key) -> LazyMapAgg<Self> {
        (*key, T::one())
    }
    fn act_agg(&(x, y): &LazyMapAgg<Self>, a: &LazyMapAct<Self>) -> Option<LazyMapAgg<Self>> {
        Some(if Self::is_act_unit(a) {
            (x, y)
        } else {
            let a = *a;
            (x + a * y, y)
        })
    }
}
impl<T> LazyMapMonoidWithKeyAct for RangeSumRangeAdd<T>
where
    T: Copy + Zero + One + Add<Output = T> + Mul<Output = T> + PartialEq,
{
    fn act_key(key: &Self::Key, act: &LazyMapAct<Self>) -> Self::Key {
        FlattenAct::<Self::ActMonoid>::act(key, act)
    }
}

pub struct RangeSumRangeLinear<T> {
    _marker: PhantomData<fn() -> T>,
}
impl<T> LazyMapMonoid for RangeSumRangeLinear<T>
where
    T: Copy + Zero + One + Add<Output = T> + Mul<Output = T> + PartialEq,
{
    type Key = T;
    type AggMonoid = (AdditiveOperation<T>, AdditiveOperation<T>);
    type ActMonoid = LinearOperation<T>;
    fn single_agg(key: &Self::Key) -> LazyMapAgg<Self> {
        (*key, T::one())
    }
    fn act_agg(&(x, y): &LazyMapAgg<Self>, &(a, b): &LazyMapAct<Self>) -> Option<LazyMapAgg<Self>> {
        Some((a * x + b * y, y))
    }
}
impl<T> LazyMapMonoidWithKeyAct for RangeSumRangeLinear<T>
where
    T: Copy + Zero + One + Add<Output = T> + Mul<Output = T> + PartialEq,
{
    fn act_key(key: &Self::Key, act: &LazyMapAct<Self>) -> Self::Key {
        LinearAct::act(key, act)
    }
}

pub struct RangePowerSumRangeLinear<T, const K: usize> {
    _marker: PhantomData<fn() -> T>,
}
impl<T, const K: usize> LazyMapMonoid for RangePowerSumRangeLinear<T, K>
where
    T: Copy + Zero + One + Add<Output = T> + Mul<Output = T> + PartialEq,
{
    type Key = T;
    type AggMonoid = ArrayOperation<AdditiveOperation<T>, K>;
    type ActMonoid = LinearOperation<T>;
    fn single_agg(key: &Self::Key) -> LazyMapAgg<Self> {
        let mut power = T::one();
        std::array::from_fn(|k| {
            if k != 0 {
                power = power * *key;
            }
            power
        })
    }
    fn act_agg(x: &LazyMapAgg<Self>, &(a, b): &LazyMapAct<Self>) -> Option<LazyMapAgg<Self>> {
        let mut coefficients = [T::one(); K];
        let mut result = *x;
        for k in 1..K {
            coefficients[k] = a * coefficients[k - 1];
            for j in (1..k).rev() {
                coefficients[j] = b * coefficients[j] + a * coefficients[j - 1];
            }
            coefficients[0] = b * coefficients[0];
            result[k] = coefficients[..=k]
                .iter()
                .zip(x)
                .fold(T::zero(), |sum, (&c, &p)| sum + c * p);
        }
        Some(result)
    }
}
impl<T, const K: usize> LazyMapMonoidWithKeyAct for RangePowerSumRangeLinear<T, K>
where
    T: Copy + Zero + One + Add<Output = T> + Mul<Output = T> + PartialEq,
{
    fn act_key(key: &Self::Key, act: &LazyMapAct<Self>) -> Self::Key {
        LinearAct::act(key, act)
    }
}

pub struct RangeSumRangeUpdate<T> {
    _marker: PhantomData<fn() -> T>,
}
impl<T> LazyMapMonoid for RangeSumRangeUpdate<T>
where
    T: Copy + Zero + One + Add<Output = T> + Mul<Output = T> + PartialEq,
{
    type Key = T;
    type AggMonoid = (AdditiveOperation<T>, AdditiveOperation<T>);
    type ActMonoid = LastOperation<T>;
    fn single_agg(key: &Self::Key) -> LazyMapAgg<Self> {
        (*key, T::one())
    }
    fn act_agg(&(x, y): &LazyMapAgg<Self>, a: &LazyMapAct<Self>) -> Option<LazyMapAgg<Self>> {
        Some((a.map(|a| a * y).unwrap_or(x), y))
    }
}
impl<T> LazyMapMonoidWithKeyAct for RangeSumRangeUpdate<T>
where
    T: Copy + Zero + One + Add<Output = T> + Mul<Output = T> + PartialEq,
{
    fn act_key(key: &Self::Key, act: &LazyMapAct<Self>) -> Self::Key {
        UpdateAct::act(key, act)
    }
}

pub struct RangeMaxRangeUpdate<T> {
    _marker: PhantomData<fn() -> T>,
}
impl<T> LazyMapMonoid for RangeMaxRangeUpdate<T>
where
    T: Clone + PartialEq + Ord + Bounded,
{
    type Key = T;
    type AggMonoid = MaxOperation<T>;
    type ActMonoid = LastOperation<T>;
    fn single_agg(key: &Self::Key) -> LazyMapAgg<Self> {
        key.clone()
    }
    fn act_agg(x: &LazyMapAgg<Self>, a: &LazyMapAct<Self>) -> Option<LazyMapAgg<Self>> {
        Some(a.as_ref().unwrap_or(x).clone())
    }
}
impl<T> LazyMapMonoidWithKeyAct for RangeMaxRangeUpdate<T>
where
    T: Clone + PartialEq + Ord + Bounded,
{
    fn act_key(key: &Self::Key, act: &LazyMapAct<Self>) -> Self::Key {
        UpdateAct::act(key, act)
    }
}

pub struct RangeMinRangeUpdate<T> {
    _marker: PhantomData<fn() -> T>,
}
impl<T> LazyMapMonoid for RangeMinRangeUpdate<T>
where
    T: Clone + PartialEq + Ord + Bounded,
{
    type Key = T;
    type AggMonoid = MinOperation<T>;
    type ActMonoid = LastOperation<T>;
    fn single_agg(key: &Self::Key) -> LazyMapAgg<Self> {
        key.clone()
    }
    fn act_agg(x: &LazyMapAgg<Self>, a: &LazyMapAct<Self>) -> Option<LazyMapAgg<Self>> {
        Some(a.as_ref().unwrap_or(x).clone())
    }
}
impl<T> LazyMapMonoidWithKeyAct for RangeMinRangeUpdate<T>
where
    T: Clone + PartialEq + Ord + Bounded,
{
    fn act_key(key: &Self::Key, act: &LazyMapAct<Self>) -> Self::Key {
        UpdateAct::act(key, act)
    }
}

pub struct RangeMaxRangeAdd<T> {
    _marker: PhantomData<fn() -> T>,
}
impl<T> LazyMapMonoid for RangeMaxRangeAdd<T>
where
    T: Clone + Ord + Bounded + Zero + Add<Output = T>,
{
    type Key = T;
    type AggMonoid = MaxOperation<T>;
    type ActMonoid = AdditiveOperation<T>;
    fn single_agg(key: &Self::Key) -> LazyMapAgg<Self> {
        key.clone()
    }
    fn act_agg(x: &LazyMapAgg<Self>, a: &LazyMapAct<Self>) -> Option<LazyMapAgg<Self>> {
        Some(if Self::is_act_unit(a) {
            x.clone()
        } else {
            x.clone() + a.clone()
        })
    }
}
impl<T> LazyMapMonoidWithKeyAct for RangeMaxRangeAdd<T>
where
    T: Clone + Ord + Bounded + Zero + Add<Output = T>,
{
    fn act_key(key: &Self::Key, act: &LazyMapAct<Self>) -> Self::Key {
        FlattenAct::<Self::ActMonoid>::act(key, act)
    }
}

pub struct RangeMinRangeAdd<T> {
    _marker: PhantomData<fn() -> T>,
}
impl<T> LazyMapMonoid for RangeMinRangeAdd<T>
where
    T: Clone + Ord + Bounded + Zero + Add<Output = T>,
{
    type Key = T;
    type AggMonoid = MinOperation<T>;
    type ActMonoid = AdditiveOperation<T>;
    fn single_agg(key: &Self::Key) -> LazyMapAgg<Self> {
        key.clone()
    }
    fn act_agg(x: &LazyMapAgg<Self>, a: &LazyMapAct<Self>) -> Option<LazyMapAgg<Self>> {
        Some(if Self::is_act_unit(a) {
            x.clone()
        } else {
            x.clone() + a.clone()
        })
    }
}
impl<T> LazyMapMonoidWithKeyAct for RangeMinRangeAdd<T>
where
    T: Clone + Ord + Bounded + Zero + Add<Output = T>,
{
    fn act_key(key: &Self::Key, act: &LazyMapAct<Self>) -> Self::Key {
        FlattenAct::<Self::ActMonoid>::act(key, act)
    }
}

pub struct RangeMinCountRangeAdd<T> {
    _marker: PhantomData<fn() -> T>,
}
impl<T> LazyMapMonoid for RangeMinCountRangeAdd<T>
where
    T: Clone + Ord + Bounded + Zero + Add<Output = T>,
{
    type Key = T;
    type AggMonoid = CountingOperation<MinOperation<T>>;
    type ActMonoid = AdditiveOperation<T>;
    fn single_agg(key: &Self::Key) -> LazyMapAgg<Self> {
        (key.clone(), 1)
    }
    fn act_agg(x: &LazyMapAgg<Self>, a: &LazyMapAct<Self>) -> Option<LazyMapAgg<Self>> {
        Some(if x.1 == 0 {
            x.clone()
        } else {
            (x.0.clone() + a.clone(), x.1)
        })
    }
}
impl<T> LazyMapMonoidWithKeyAct for RangeMinCountRangeAdd<T>
where
    T: Clone + Ord + Bounded + Zero + Add<Output = T>,
{
    fn act_key(key: &Self::Key, act: &LazyMapAct<Self>) -> Self::Key {
        FlattenAct::<Self::ActMonoid>::act(key, act)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangeChminChmaxAdd<T> {
    lb: T,
    ub: T,
    bias: T,
}
impl<T> RangeChminChmaxAdd<T>
where
    T: Zero + Bounded,
{
    pub fn chmin(x: T) -> Self {
        Self {
            lb: T::minimum(),
            ub: x,
            bias: T::zero(),
        }
    }
    pub fn chmax(x: T) -> Self {
        Self {
            lb: x,
            ub: T::maximum(),
            bias: T::zero(),
        }
    }
    pub fn add(x: T) -> Self {
        Self {
            lb: T::minimum(),
            ub: T::maximum(),
            bias: x,
        }
    }
}
impl<T> Magma for RangeChminChmaxAdd<T>
where
    T: Copy
        + Zero
        + One
        + Ord
        + Bounded
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + PartialEq,
{
    type T = Self;
    fn operate(x: &Self::T, y: &Self::T) -> Self::T {
        Self {
            lb: (x.lb + x.bias).min(y.ub).max(y.lb) - x.bias,
            ub: (x.ub + x.bias).max(y.lb).min(y.ub) - x.bias,
            bias: x.bias + y.bias,
        }
    }
}
impl<T> Associative for RangeChminChmaxAdd<T> where
    T: Copy
        + Zero
        + One
        + Ord
        + Bounded
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + PartialEq
{
}
impl<T> Unital for RangeChminChmaxAdd<T>
where
    T: Copy
        + Zero
        + One
        + Ord
        + Bounded
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + PartialEq,
{
    fn unit() -> Self::T {
        Self {
            lb: T::minimum(),
            ub: T::maximum(),
            bias: T::zero(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeSumRangeChminChmaxAdd<T> {
    min: T,
    max: T,
    min2: T,
    max2: T,
    pub sum: T,
    size: T,
    n_min: T,
    n_max: T,
}

impl<T> RangeSumRangeChminChmaxAdd<T>
where
    T: Copy
        + Zero
        + One
        + Ord
        + Bounded
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + PartialEq,
{
    pub fn single(key: T, size: T) -> Self {
        Self {
            min: key,
            max: key,
            min2: T::maximum(),
            max2: T::minimum(),
            sum: key * size,
            size,
            n_min: size,
            n_max: size,
        }
    }
}
impl<T> Magma for RangeSumRangeChminChmaxAdd<T>
where
    T: Copy
        + Zero
        + One
        + Ord
        + Bounded
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + PartialEq,
{
    type T = Self;
    fn operate(x: &Self::T, y: &Self::T) -> Self::T {
        Self {
            min: x.min.min(y.min),
            max: x.max.max(y.max),
            min2: if x.min == y.min {
                x.min2.min(y.min2)
            } else if x.min2 <= y.min {
                x.min2
            } else if y.min2 <= x.min {
                y.min2
            } else {
                x.min.max(y.min)
            },
            max2: if x.max == y.max {
                x.max2.max(y.max2)
            } else if x.max2 >= y.max {
                x.max2
            } else if y.max2 >= x.max {
                y.max2
            } else {
                x.max.min(y.max)
            },
            sum: x.sum + y.sum,
            size: x.size + y.size,
            n_min: match x.min.cmp(&y.min) {
                Ordering::Less => x.n_min,
                Ordering::Equal => x.n_min + y.n_min,
                Ordering::Greater => y.n_min,
            },
            n_max: match x.max.cmp(&y.max) {
                Ordering::Less => y.n_max,
                Ordering::Equal => x.n_max + y.n_max,
                Ordering::Greater => x.n_max,
            },
        }
    }
}
impl<T> Associative for RangeSumRangeChminChmaxAdd<T> where
    T: Copy
        + Zero
        + One
        + Ord
        + Bounded
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + PartialEq
{
}
impl<T> Unital for RangeSumRangeChminChmaxAdd<T>
where
    T: Copy
        + Zero
        + One
        + Ord
        + Bounded
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + PartialEq,
{
    fn unit() -> Self::T {
        Self {
            min: T::maximum(),
            max: T::minimum(),
            min2: T::maximum(),
            max2: T::minimum(),
            sum: T::zero(),
            size: T::zero(),
            n_min: T::zero(),
            n_max: T::zero(),
        }
    }
}

impl<T> MonoidAct for RangeChminChmaxAdd<T>
where
    T: Copy
        + Zero
        + One
        + Ord
        + Bounded
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + PartialEq,
{
    type Key = T;
    type Act = RangeChminChmaxAdd<T>;
    type ActMonoid = RangeChminChmaxAdd<T>;
    fn act(x: &Self::Key, a: &Self::Act) -> Self::Key {
        (*x).max(a.lb).min(a.ub) + a.bias
    }
}

impl<T> LazyMapMonoid for RangeSumRangeChminChmaxAdd<T>
where
    T: Copy
        + Zero
        + One
        + Ord
        + Bounded
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + PartialEq,
{
    type Key = T;
    type AggMonoid = Self;
    type ActMonoid = RangeChminChmaxAdd<T>;
    fn single_agg(&key: &Self::Key) -> LazyMapAgg<Self> {
        Self::single(key, T::one())
    }
    fn act_agg(x: &LazyMapAgg<Self>, a: &LazyMapAct<Self>) -> Option<LazyMapAgg<Self>> {
        Some(if Self::is_act_unit(a) {
            x.clone()
        } else if x.size.is_zero() {
            Self::unit()
        } else if x.min == x.max || a.lb == a.ub || a.lb >= x.max || a.ub <= x.min {
            Self::single(x.min.max(a.lb).min(a.ub) + a.bias, x.size)
        } else if x.min2 == x.max {
            let mut x = x.clone();
            let min = x.min.max(a.lb) + a.bias;
            let max = x.max.min(a.ub) + a.bias;
            x.min = min;
            x.max2 = min;
            x.max = max;
            x.min2 = max;
            x.sum = min * x.n_min + max * x.n_max;
            x
        } else if a.lb < x.min2 && x.max2 < a.ub {
            let mut x = x.clone();
            let min = x.min.max(a.lb);
            let max = x.max.min(a.ub);
            x.sum = x.sum + (min - x.min) * x.n_min + (max - x.max) * x.n_max + a.bias * x.size;
            x.min = min + a.bias;
            x.max = max + a.bias;
            x.min2 = x.min2 + a.bias;
            x.max2 = x.max2 + a.bias;
            x
        } else {
            return None;
        })
    }
}

impl<T> LazyMapMonoidWithKeyAct for RangeSumRangeChminChmaxAdd<T>
where
    T: Copy
        + Zero
        + One
        + Ord
        + Bounded
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + PartialEq,
{
    fn act_key(key: &Self::Key, act: &LazyMapAct<Self>) -> Self::Key {
        RangeChminChmaxAdd::act(key, act)
    }
}

#[cfg(kani)]
mod verification {
    use super::*;
    use crate::num::Saturating;

    #[kani::proof]
    #[kani::unwind(7)]
    fn proof_clamp_add_composition() {
        let key: i32 = kani::any();
        let values: [i16; 3] = kani::any();
        let kinds: [u8; 3] = kani::any();
        let mut composed = <RangeChminChmaxAdd<Saturating<i64>> as Unital>::unit();
        let mut expected = key as i64;
        for i in 0..3 {
            kani::assume(kinds[i] < 3);
            let value = values[i] as i64;
            let action = match kinds[i] {
                0 => {
                    expected = expected.min(value);
                    RangeChminChmaxAdd::chmin(Saturating(value))
                }
                1 => {
                    expected = expected.max(value);
                    RangeChminChmaxAdd::chmax(Saturating(value))
                }
                _ => {
                    expected += value;
                    RangeChminChmaxAdd::add(Saturating(value))
                }
            };
            composed = <RangeChminChmaxAdd<_> as Magma>::operate(&composed, &action);
            kani::assert(
                RangeChminChmaxAdd::act(&Saturating(key as i64), &composed).0 == expected,
                "For every i32 key and three i16 chmin/chmax/add updates widened to Saturating<i64>, composition equals direct commands in order, including all operation mixtures",
            );
        }
        kani::cover!(kinds == [0, 1, 2]);
        kani::cover!(kinds == [2, 0, 1]);
        kani::cover!(key == i32::MAX && values == [i16::MAX; 3] && kinds == [2; 3]);
    }

    macro_rules! proof_extremum_update {
        ($name:ident, $map:ty) => {
            #[kani::proof]
            fn $name() {
                type M = $map;
                let x: i64 = kani::any();
                let y: i64 = kani::any();
                let a: Option<i64> = kani::any();
                let b: Option<i64> = kani::any();
                let mapped_x = M::act_agg(&x, &a).unwrap();
                let mapped_y = M::act_agg(&y, &a).unwrap();
                kani::assert(
                    M::act_agg(&x, &M::act_unit()) == Some(x),
                    "For every i64 extremum aggregate, the identity update preserves its value",
                );
                kani::assert(
                    M::act_agg(&M::agg_operate(&x, &y), &a)
                        == Some(M::agg_operate(&mapped_x, &mapped_y)),
                    "For all nonempty i64 extrema aggregates and optional assignments, updating after merging equals merging after updating",
                );
                let expected = Some(b.or(a).unwrap_or(x));
                kani::assert(
                    M::act_agg(&mapped_x, &b) == expected
                        && M::act_agg(&x, &M::act_operate(&a, &b)) == expected,
                    "For every i64 aggregate and pair of optional assignments, composition applies them in order and the last present assignment wins",
                );
                kani::assert(
                    M::act_agg(&M::single_agg(&x), &a)
                        == Some(M::single_agg(&M::act_key(&x, &a))),
                    "For every i64 key and optional assignment, updating its singleton aggregate agrees with updating the key",
                );
                kani::cover!(a.is_none());
                kani::cover!(a.is_some() && b.is_some() && a != b);
            }
        };
    }

    proof_extremum_update!(proof_min_update, RangeMinRangeUpdate<i64>);
    proof_extremum_update!(proof_max_update, RangeMaxRangeUpdate<i64>);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        data_structure::LazySegmentTree,
        num::mint_basic::MInt998244353 as M,
        tools::{
            Xorshift,
            testutil::{exhaustive_sequences, sequence_cases},
        },
    };
    use std::num::Wrapping;

    fn check_mapping<const K: usize>() {
        type W = Wrapping<u64>;
        let mut rng = Xorshift::default();
        let inputs =
            exhaustive_sequences(-2i64..=2, 0..=3).chain(sequence_cases(&mut rng, -8..=8, 0..=65));
        for values in inputs {
            let leaves: Vec<_> = values
                .iter()
                .enumerate()
                .map(|(i, &x)| (Wrapping(x as u64), Wrapping([0, 1, u64::MAX, 2][i % 4])))
                .collect();
            let agg: [W; K] = std::array::from_fn(|k| {
                leaves
                    .iter()
                    .map(|&(x, w)| w * Wrapping(x.0.wrapping_pow(k as u32)))
                    .sum()
            });
            assert_eq!(
                RangePowerSumRangeLinear::act_agg(&agg, &(W::one(), W::zero())),
                Some(agg)
            );
            for a in -2i64..=2 {
                for b in -2i64..=2 {
                    let act = (Wrapping(a as u64), Wrapping(b as u64));
                    let mapped = RangePowerSumRangeLinear::act_agg(&agg, &act).unwrap();
                    let expected: [W; K] = std::array::from_fn(|k| {
                        leaves
                            .iter()
                            .map(|&(x, w)| {
                                w * Wrapping((act.0 * x + act.1).0.wrapping_pow(k as u32))
                            })
                            .sum()
                    });
                    assert_eq!(mapped, expected);
                    for &(key, _) in &leaves {
                        let leaf = RangePowerSumRangeLinear::<W, K>::single_agg(&key);
                        let updated = RangePowerSumRangeLinear::<W, K>::act_key(&key, &act);
                        assert_eq!(
                            RangePowerSumRangeLinear::act_agg(&leaf, &act),
                            Some(RangePowerSumRangeLinear::single_agg(&updated))
                        );
                    }
                    for c in -2i64..=2 {
                        let next = (Wrapping(c as u64), Wrapping(3));
                        let composed = RangePowerSumRangeLinear::<W, K>::act_operate(&act, &next);
                        assert_eq!(
                            RangePowerSumRangeLinear::act_agg(&mapped, &next),
                            RangePowerSumRangeLinear::act_agg(&agg, &composed)
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_power_sum_mapping() {
        check_mapping::<0>();
        check_mapping::<1>();
        check_mapping::<2>();
        check_mapping::<5>();
        check_mapping::<8>();
    }

    #[test]
    fn test_power_sum_range_updates() {
        let mut rng = Xorshift::default();
        let inputs: Vec<_> = sequence_cases(&mut rng, -100i64..=100, 0..=257).collect();
        for input in inputs {
            let mut values: Vec<_> = input.into_iter().map(M::from).collect();
            let n = values.len();
            let mut seg = LazySegmentTree::<RangePowerSumRangeLinear<M, 5>>::from_keys(
                values.iter().copied(),
            );
            for step in 0..128 {
                let (l, r) = match step % 8 {
                    0 | 1 => (0, n),
                    2 => (n, n),
                    3 => (0, n.min(1)),
                    _ => {
                        let a = rng.random(0..=n);
                        let b = rng.random(0..=n);
                        (a.min(b), a.max(b))
                    }
                };
                let act = match step % 6 {
                    0 => (M::from(2), M::from(3)),
                    1 => (M::from(3), M::from(4)),
                    2 => (M::zero(), rng.random(..)),
                    3 => (M::one(), rng.random(..)),
                    4 => (rng.random(..), M::zero()),
                    _ => (M::one(), M::zero()),
                };
                seg.update(l..r, act);
                for x in &mut values[l..r] {
                    *x = act.0 * *x + act.1;
                }
                if n != 0 && step % 7 == 0 {
                    let i = rng.random(0..n);
                    values[i] = rng.random(..);
                    seg.set(i, RangePowerSumRangeLinear::single_agg(&values[i]));
                }
                let expected: [M; 5] =
                    std::array::from_fn(|k| values[l..r].iter().map(|&x| x.pow(k)).sum());
                assert_eq!(seg.fold(l..r), expected);
                assert_eq!(
                    seg.fold_all(),
                    std::array::from_fn(|k| values.iter().map(|&x| x.pow(k)).sum())
                );
                if n != 0 {
                    let i = rng.random(0..n);
                    assert_eq!(seg.get(i), std::array::from_fn(|k| values[i].pow(k)));
                }
            }
        }
    }
}

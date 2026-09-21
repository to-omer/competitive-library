use super::Bounded;
use std::{
    convert::TryFrom,
    ops::{Bound, Range, RangeBounds, RangeInclusive},
};

pub trait DiscreteSteps<Delta>: Clone {
    fn delta() -> Delta;
    fn steps_between(start: &Self, end: &Self) -> Option<Delta>;
    fn forward_checked(start: Self, delta: Delta) -> Option<Self>;
    fn backward_checked(start: Self, delta: Delta) -> Option<Self>;
    fn forward(start: Self, delta: Delta) -> Self {
        Self::forward_checked(start, delta).expect("overflow in `DiscreteSteps::forward`")
    }
    fn backward(start: Self, delta: Delta) -> Self {
        Self::backward_checked(start, delta).expect("overflow in `DiscreteSteps::backward`")
    }
    fn forward_delta_checked(start: Self) -> Option<Self> {
        Self::forward_checked(start, Self::delta())
    }
    fn backward_delta_checked(start: Self) -> Option<Self> {
        Self::backward_checked(start, Self::delta())
    }
    fn forward_delta(start: Self) -> Self {
        Self::forward(start, Self::delta())
    }
    fn backward_delta(start: Self) -> Self {
        Self::backward(start, Self::delta())
    }
}

macro_rules! impl_discrete_steps_integer {
    (@common $u_source:ident) => {
        fn delta() -> $u_source {
            1
        }
        fn forward(start: Self, delta: $u_source) -> Self {
            assert!(Self::forward_checked(start, delta).is_some(), "attempt to add with overflow");
            start.wrapping_add(delta as Self)
        }
        fn backward(start: Self, delta: $u_source) -> Self {
            assert!(Self::backward_checked(start, delta).is_some(), "attempt to subtract with overflow");
            start.wrapping_sub(delta as Self)
        }
    };
    ($u_source:ident $i_source:ident; $($u_narrower:ident $i_narrower:ident),*; $($u_wider:ident $i_wider:ident),*) => {
        $(
            impl DiscreteSteps<$u_source> for $u_narrower {
                impl_discrete_steps_integer!(@common $u_source);
                fn steps_between(start: &Self, end: &Self) -> Option<$u_source> {
                    if *start <= *end {
                        Some((*end - *start) as $u_source)
                    } else {
                        None
                    }
                }
                fn forward_checked(start: Self, delta: $u_source) -> Option<Self> {
                    Self::try_from(delta).ok().and_then(|delta| start.checked_add(delta))
                }
                fn backward_checked(start: Self, delta: $u_source) -> Option<Self> {
                    Self::try_from(delta).ok().and_then(|delta| start.checked_sub(delta))
                }
            }
            impl DiscreteSteps<$u_source> for $i_narrower {
                impl_discrete_steps_integer!(@common $u_source);
                fn steps_between(start: &Self, end: &Self) -> Option<$u_source> {
                    if *start <= *end {
                        Some((*end as $i_source).wrapping_sub(*start as $i_source) as $u_source)
                    } else {
                        None
                    }
                }
                fn forward_checked(start: Self, delta: $u_source) -> Option<Self> {
                    $u_narrower::try_from(delta).ok().and_then(|delta| {
                        let wrapped = start.wrapping_add(delta as Self);
                        if wrapped >= start { Some(wrapped) } else { None }
                    })
                }
                fn backward_checked(start: Self, delta: $u_source) -> Option<Self> {
                    $u_narrower::try_from(delta).ok().and_then(|delta| {
                        let wrapped = start.wrapping_sub(delta as Self);
                        if wrapped <= start { Some(wrapped) } else { None }
                    })
                }
            }
        )*
        $(
            impl DiscreteSteps<$u_source> for $u_wider {
                impl_discrete_steps_integer!(@common $u_source);
                fn steps_between(start: &Self, end: &Self) -> Option<$u_source> {
                    if *start <= *end {
                        $u_source::try_from(*end - *start).ok()
                    } else {
                        None
                    }
                }
                fn forward_checked(start: Self, delta: $u_source) -> Option<Self> {
                    start.checked_add(delta as Self)
                }
                fn backward_checked(start: Self, delta: $u_source) -> Option<Self> {
                    start.checked_sub(delta as Self)
                }
            }
            impl DiscreteSteps<$u_source> for $i_wider {
                impl_discrete_steps_integer!(@common $u_source);
                fn steps_between(start: &Self, end: &Self) -> Option<$u_source> {
                    if *start <= *end {
                        end.checked_sub(*start).and_then(|result| $u_source::try_from(result).ok())
                    } else {
                        None
                    }
                }
                fn forward_checked(start: Self, delta: $u_source) -> Option<Self> {
                    start.checked_add(delta as Self)
                }
                fn backward_checked(start: Self, delta: $u_source) -> Option<Self> {
                    start.checked_sub(delta as Self)
                }
            }
        )*
    };
}
impl_discrete_steps_integer!(u16 i16; u8 i8, u16 i16, usize isize; u32 i32, u64 i64, u128 i128);
impl_discrete_steps_integer!(u32 i32; u8 i8, u16 i16, u32 i32, usize isize; u64 i64, u128 i128);
impl_discrete_steps_integer!(u64 i64; u8 i8, u16 i16, u32 i32, u64 i64, usize isize; u128 i128);
impl_discrete_steps_integer!(u128 i128; u8 i8, u16 i16, u32 i32, u64 i64, u128 i128, usize isize;);
// #[cfg(target_pointer_width = "16")]
// impl_discrete_steps_integer!(usize isize; u8 i8, u16 i16, usize isize; u32 i32, u64 i64, u128 i128);
// #[cfg(target_pointer_width = "32")]
// impl_discrete_steps_integer!(usize isize; u8 i8, u16 i16, u32 i32, usize isize; u64 i64, u128 i128);
// #[cfg(target_pointer_width = "64")]
impl_discrete_steps_integer!(usize isize; u8 i8, u16 i16, u32 i32, u64 i64, usize isize; u128 i128);

pub trait RangeBoundsExt<T> {
    fn start_bound_included_checked(&self) -> Option<T>;
    fn start_bound_excluded_checked(&self) -> Option<T>;
    fn end_bound_included_checked(&self) -> Option<T>;
    fn end_bound_excluded_checked(&self) -> Option<T>;
    fn start_bound_included(&self) -> T;
    fn start_bound_excluded(&self) -> T;
    fn end_bound_included(&self) -> T;
    fn end_bound_excluded(&self) -> T;
    fn start_bound_included_bounded(&self, lb: T) -> Option<T>
    where
        T: Ord;
    fn start_bound_excluded_bounded(&self, lb: T) -> Option<T>
    where
        T: Ord;
    fn end_bound_included_bounded(&self, ub: T) -> Option<T>
    where
        T: Ord;
    fn end_bound_excluded_bounded(&self, ub: T) -> Option<T>
    where
        T: Ord;

    fn to_range_checked(&self) -> Option<Range<T>> {
        match (
            self.start_bound_included_checked(),
            self.end_bound_excluded_checked(),
        ) {
            (Some(start), Some(end)) => Some(start..end),
            _ => None,
        }
    }
    fn to_range(&self) -> Range<T> {
        self.start_bound_included()..self.end_bound_excluded()
    }
    fn to_range_bounded(&self, min: T, max: T) -> Option<Range<T>>
    where
        T: Ord,
    {
        Some(self.start_bound_included_bounded(min)?..self.end_bound_excluded_bounded(max)?)
    }
    fn to_range_inclusive_checked(&self) -> Option<RangeInclusive<T>> {
        match (
            self.start_bound_included_checked(),
            self.end_bound_included_checked(),
        ) {
            (Some(start), Some(end)) => Some(start..=end),
            _ => None,
        }
    }
    fn to_range_inclusive(&self) -> RangeInclusive<T> {
        self.start_bound_included()..=self.end_bound_included()
    }
    fn to_range_inclusive_bounded(&self, min: T, max: T) -> Option<RangeInclusive<T>>
    where
        T: Ord,
    {
        Some(self.start_bound_included_bounded(min)?..=self.end_bound_included_bounded(max)?)
    }
}

macro_rules! impl_range_bounds_ext {
    ($($source:ident => $($target:ident)+);* $(;)?) => {
        $($(
            impl<R> RangeBoundsExt<$target> for R
            where
                R: RangeBounds<$target>,
            {
                fn start_bound_included_checked(&self) -> Option<$target> {
                    match self.start_bound() {
                        Bound::Included(x) => Some(*x),
                        Bound::Excluded(x) => DiscreteSteps::<$source>::forward_delta_checked(*x),
                        Bound::Unbounded => Some(Bounded::minimum()),
                    }
                }
                fn start_bound_excluded_checked(&self) -> Option<$target> {
                    match self.start_bound() {
                        Bound::Included(x) => DiscreteSteps::<$source>::backward_delta_checked(*x),
                        Bound::Excluded(x) => Some(*x),
                        Bound::Unbounded => None,
                    }
                }
                fn end_bound_included_checked(&self) -> Option<$target> {
                    match self.end_bound() {
                        Bound::Included(x) => Some(*x),
                        Bound::Excluded(x) => DiscreteSteps::<$source>::backward_delta_checked(*x),
                        Bound::Unbounded => Some(Bounded::maximum()),
                    }
                }
                fn end_bound_excluded_checked(&self) -> Option<$target> {
                    match self.end_bound() {
                        Bound::Included(x) => DiscreteSteps::<$source>::forward_delta_checked(*x),
                        Bound::Excluded(x) => Some(*x),
                        Bound::Unbounded => None,
                    }
                }
                fn start_bound_included(&self) -> $target {
                    match self.start_bound() {
                        Bound::Included(x) => *x,
                        Bound::Excluded(x) => DiscreteSteps::<$source>::forward_delta(*x),
                        Bound::Unbounded => Bounded::minimum(),
                    }
                }
                fn start_bound_excluded(&self) -> $target {
                    match self.start_bound() {
                        Bound::Included(x) => DiscreteSteps::<$source>::backward_delta(*x),
                        Bound::Excluded(x) => *x,
                        Bound::Unbounded => DiscreteSteps::<$source>::backward_delta(Bounded::minimum()),
                    }
                }
                fn end_bound_included(&self) -> $target {
                    match self.end_bound() {
                        Bound::Included(x) => *x,
                        Bound::Excluded(x) => DiscreteSteps::<$source>::backward_delta(*x),
                        Bound::Unbounded => Bounded::maximum(),
                    }
                }
                fn end_bound_excluded(&self) -> $target {
                    match self.end_bound() {
                        Bound::Included(x) => DiscreteSteps::<$source>::forward_delta(*x),
                        Bound::Excluded(x) => *x,
                        Bound::Unbounded => DiscreteSteps::<$source>::forward_delta(Bounded::maximum()),
                    }
                }
                fn start_bound_included_bounded(&self, lb: $target) -> Option<$target>
                where
                    $target: Ord
                {
                    match self.start_bound() {
                        Bound::Included(x) => Some(*x).filter(|&x| lb <= x),
                        Bound::Excluded(x) => DiscreteSteps::<$source>::forward_delta_checked(*x).filter(|&x| lb <= x),
                        Bound::Unbounded => Some(lb),
                    }
                }
                fn start_bound_excluded_bounded(&self, lb: $target) -> Option<$target>
                where
                    $target: Ord
                {
                    match self.start_bound() {
                        Bound::Included(x) => DiscreteSteps::<$source>::backward_delta_checked(*x).filter(|&x| lb <= x),
                        Bound::Excluded(x) => Some(*x).filter(|&x| lb <= x),
                        Bound::Unbounded => Some(lb),
                    }
                }
                fn end_bound_included_bounded(&self, ub: $target) -> Option<$target>
                where
                    $target: Ord
                {
                    match self.end_bound() {
                        Bound::Included(x) => Some(*x).filter(|&x| x <= ub),
                        Bound::Excluded(x) => DiscreteSteps::<$source>::backward_delta_checked(*x).filter(|&x| x <= ub),
                        Bound::Unbounded => Some(ub),
                    }
                }
                fn end_bound_excluded_bounded(&self, ub: $target) -> Option<$target>
                where
                    $target: Ord
                {
                    match self.end_bound() {
                        Bound::Included(x) => DiscreteSteps::<$source>::forward_delta_checked(*x).filter(|&x| x <= ub),
                        Bound::Excluded(x) => Some(*x).filter(|&x| x <= ub),
                        Bound::Unbounded => Some(ub),
                    }
                }
            }
        )+)*
    };
}
impl_range_bounds_ext!(
    u16 => u8 i8 u16 i16;
    u32 => u32 i32;
    u64 => u64 i64;
    u128 => u128 i128;
    usize => isize usize;
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn test_range_bounds() {
        let mut rng = Xorshift::default();
        let bounds: Vec<_> = std::iter::once(Bound::Unbounded)
            .chain((0..=u8::MAX).flat_map(|x| [Bound::Included(x), Bound::Excluded(x)]))
            .collect();
        for (lower, upper) in bounds
            .iter()
            .flat_map(|&lower| bounds.iter().map(move |&upper| (lower, upper)))
        {
            let range = (lower, upper);
            let start = (0..=u8::MAX).find(|x| match lower {
                Bound::Included(a) => *x >= a,
                Bound::Excluded(a) => *x > a,
                Bound::Unbounded => true,
            });
            let end = (0..=u8::MAX).rev().find(|x| match upper {
                Bound::Included(b) => *x <= b,
                Bound::Excluded(b) => *x < b,
                Bound::Unbounded => true,
            });
            let before = match lower {
                Bound::Excluded(a) => Some(a),
                _ => start.and_then(|x| x.checked_sub(1)),
            };
            let after = match upper {
                Bound::Excluded(b) => Some(b),
                _ => end.and_then(|x| x.checked_add(1)),
            };
            assert_eq!(range.start_bound_included_checked(), start);
            assert_eq!(range.start_bound_excluded_checked(), before);
            assert_eq!(range.end_bound_included_checked(), end);
            assert_eq!(range.end_bound_excluded_checked(), after);
            assert_eq!(
                range.to_range_checked(),
                start.zip(after).map(|(a, b)| a..b)
            );
            assert_eq!(
                range.to_range_inclusive_checked(),
                start.zip(end).map(|(a, b)| a..=b)
            );
            if let Some(expected) = range.to_range_checked() {
                assert_eq!(range.to_range(), expected);
            }
            if let Some(expected) = range.to_range_inclusive_checked() {
                assert_eq!(range.to_range_inclusive(), expected);
            }
            let lb: u8 = rng.random(..);
            let ub: u8 = rng.random(..);
            let start = if lower == Bound::Unbounded {
                Some(lb)
            } else {
                start.filter(|&x| x >= lb)
            };
            let end = if upper == Bound::Unbounded {
                Some(ub)
            } else {
                end.filter(|&x| x <= ub)
            };
            let after = if upper == Bound::Unbounded {
                Some(ub)
            } else {
                after.filter(|&x| x <= ub)
            };
            assert_eq!(
                range.to_range_bounded(lb, ub),
                start.zip(after).map(|(a, b)| a..b)
            );
            assert_eq!(
                range.to_range_inclusive_bounded(lb, ub),
                start.zip(end).map(|(a, b)| a..=b)
            );
        }
    }
}

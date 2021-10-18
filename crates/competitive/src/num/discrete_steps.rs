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

    #[test]
    fn test_start_bound_included() {
        assert_eq!((2..3).start_bound_included(), 2);
        assert_eq!((..3usize).start_bound_included(), 0);
    }

    #[test]
    fn test_start_bound_excluded() {
        assert_eq!((2..3).start_bound_excluded(), 1);
        assert_eq!((..3usize).start_bound_excluded_checked(), None);
    }

    #[test]
    fn test_end_bound_included() {
        assert_eq!((2..3).end_bound_included(), 2);
        assert_eq!((2usize..).end_bound_included(), !0usize);
    }

    #[test]
    fn test_end_bound_excluded() {
        assert_eq!((2..3).end_bound_excluded(), 3);
        assert_eq!((2usize..).end_bound_excluded_checked(), None);
    }

    #[test]
    fn test_to_range() {
        assert_eq!((2..3).to_range(), 2..3);
        assert_eq!((2..=3).to_range(), 2..4);
        assert_eq!((..3usize).to_range(), 0..3);
    }

    #[test]
    fn test_to_range_inclusive() {
        assert_eq!((2..3).to_range_inclusive(), 2..=2);
        assert_eq!((2..=3).to_range_inclusive(), 2..=3);
        assert_eq!((..3usize).to_range_inclusive(), 0..=2);
    }
}

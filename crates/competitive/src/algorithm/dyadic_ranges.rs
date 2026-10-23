/// Maximal aligned power-of-two intervals.
pub fn dyadic_ranges(range: std::ops::Range<u64>) -> impl Iterator<Item = std::ops::Range<u64>> {
    let mut left = range.start;
    let right = range.end;
    std::iter::from_fn(move || {
        if left >= right {
            return None;
        }
        let remaining = right - left;
        let exponent = left.trailing_zeros().min(remaining.ilog2());
        let end = left + (1u64 << exponent);
        let result = left..end;
        left = end;
        Some(result)
    })
}

#[cfg(kani)]
mod verification {
    use super::*;

    #[kani::proof]
    fn proof_dyadic_step() {
        let left: u64 = kani::any();
        let right: u64 = kani::any();
        let mut iter = dyadic_ranges(left..right);
        match iter.next() {
            Some(range) => {
                let length = range.end - range.start;
                kani::assert(
                    range.start == left
                        && range.end <= right
                        && length.is_power_of_two()
                        && range.start & (length - 1) == 0,
                    "For all nonempty u64 intervals, the first dyadic range starts at the left endpoint, is aligned, has power-of-two length and fits within the interval",
                );
                let doubled = 2 * length as u128;
                kani::assert(
                    left as u128 & (doubled - 1) != 0 || left as u128 + doubled > right as u128,
                    "For all u64 intervals, the emitted dyadic range has the largest aligned power-of-two length that fits at the current left endpoint",
                );
                kani::assert(
                    match iter.next() {
                        Some(next) => next.start == range.end && next.end <= right,
                        None => range.end == right,
                    },
                    "For all u64 intervals, dyadic iteration advances without gaps and stops only at the right endpoint",
                );
            }
            None => kani::assert(
                left >= right && iter.next().is_none(),
                "Empty and reversed u64 intervals produce no dyadic ranges and stay exhausted",
            ),
        }
        kani::cover!(left == 0 && right == u64::MAX);
        kani::cover!(left == u64::MAX - 1 && right == u64::MAX);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn test_dyadic_ranges() {
        let mut rng = Xorshift::default();
        let ranges = (0..=256)
            .flat_map(|left| (0..=256).map(move |right| (left, right)))
            .chain([
                (0, u64::MAX),
                (u64::MAX - 1000, u64::MAX),
                (1 << 63, u64::MAX),
                (u64::MAX, u64::MAX),
                (u64::MAX, 0),
            ])
            .chain(rng.random_iter((0..=u64::MAX, 0..=u64::MAX)).take(1000));
        for (left, right) in ranges {
            if left > right {
                assert!(dyadic_ranges(left..right).next().is_none());
                continue;
            }
            let mut expected_start = left;
            for range in dyadic_ranges(left..right) {
                let length = range.end - range.start;
                assert!(length.is_power_of_two());
                assert_eq!(range.start % length, 0);
                assert_eq!(range.start, expected_start);
                assert!(range.end <= right);
                let parent_length = 2 * length as u128;
                let parent_start = range.start as u128 & !(parent_length - 1);
                assert!(
                    parent_start < left as u128 || parent_start + parent_length > right as u128
                );
                expected_start = range.end;
            }
            assert_eq!(expected_start, right);
        }
    }
}

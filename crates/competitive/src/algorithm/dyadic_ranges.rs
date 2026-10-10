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

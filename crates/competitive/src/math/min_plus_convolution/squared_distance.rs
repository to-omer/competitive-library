use super::Signed;

fn index_as_value<T>(index: usize) -> T
where
    T: Signed + TryFrom<usize>,
{
    T::try_from(index)
        .ok()
        .expect("squared-distance index must fit the value type")
}

fn first_position_of_new_source<T>(values: &[T], previous_source: usize, new_source: usize) -> T
where
    T: Signed + TryFrom<usize>,
{
    let previous_index: T = index_as_value(previous_source);
    let new_index: T = index_as_value(new_source);
    let numerator = values[new_source] + new_index * new_index
        - values[previous_source]
        - previous_index * previous_index;
    let denominator = (new_index - previous_index) + (new_index - previous_index);
    numerator.div_euclid(denominator)
        + if numerator.rem_euclid(denominator).is_zero() {
            T::zero()
        } else {
            T::one()
        }
}

/// Computes min-plus convolution with squared distance in linear time.
///
/// The value at `p` is `min_q(values[q] + (p - q)^2)`. `T::maximum()`
/// represents an unreachable source.
///
/// # Panics
///
/// Panics if an index cannot be represented by `T`. Arithmetic overflow is
/// the caller's responsibility.
pub fn min_plus_convolution_with_squared_distance<T>(values: &[T]) -> Vec<T>
where
    T: Signed + TryFrom<usize>,
{
    if values.is_empty() {
        return Vec::new();
    }
    let mut sources = Vec::with_capacity(values.len());
    let mut first_positions = Vec::with_capacity(values.len());
    for (new_source, &value) in values.iter().enumerate() {
        if value.is_maximum() {
            continue;
        }
        let mut first_position = T::zero();
        while let Some(&previous_source) = sources.last() {
            first_position = first_position_of_new_source(values, previous_source, new_source);
            if first_position > first_positions[first_positions.len() - 1] {
                break;
            }
            sources.pop();
            first_positions.pop();
        }
        if sources.is_empty() {
            first_position = T::zero();
        }
        if first_position < index_as_value(values.len()) {
            sources.push(new_source);
            first_positions.push(first_position.max(T::zero()));
        }
    }
    if sources.is_empty() {
        return vec![T::maximum(); values.len()];
    }
    let mut active_source_index = 0;
    let mut result = Vec::with_capacity(values.len());
    for position in 0..values.len() {
        while active_source_index + 1 < sources.len()
            && first_positions[active_source_index + 1] <= index_as_value(position)
        {
            active_source_index += 1;
        }
        let source = sources[active_source_index];
        let distance: T = index_as_value(source.abs_diff(position));
        result.push(values[source] + distance * distance);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn test_min_plus_convolution_with_squared_distance() {
        let mut rng = Xorshift::default();
        for len in 0..=32 {
            for case in 0..32 {
                let values: Vec<_> = (0..len)
                    .map(|_| {
                        if case == 0 || rng.random(0_u64..5) == 0 {
                            i64::MAX
                        } else {
                            rng.random(-50_i64..=50)
                        }
                    })
                    .collect();
                let expected: Vec<_> = (0..len)
                    .map(|position| {
                        values
                            .iter()
                            .copied()
                            .enumerate()
                            .filter(|(_, value)| *value != i64::MAX)
                            .map(|(source, value)| {
                                let distance = source.abs_diff(position) as i64;
                                value + distance * distance
                            })
                            .min()
                            .unwrap_or(i64::MAX)
                    })
                    .collect();
                assert_eq!(
                    min_plus_convolution_with_squared_distance(&values),
                    expected
                );
            }
        }
    }
}

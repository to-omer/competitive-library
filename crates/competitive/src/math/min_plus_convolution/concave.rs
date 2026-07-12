use super::{Signed, assert_finite, output_len};

pub(super) fn is_concave<T>(values: &[T]) -> bool
where
    T: Signed,
{
    values
        .windows(3)
        .all(|window| window[1] - window[0] >= window[2] - window[1])
}

fn bit_width(value: usize) -> u32 {
    value.checked_ilog2().map_or(0, |log| log + 1)
}

struct ConcaveEnvelope<'a, T> {
    arbitrary: &'a [T],
    concave: &'a [T],
    leaf_count: usize,
    query_root: usize,
    node_curves: Vec<Option<usize>>,
    result: Vec<T>,
}

impl<'a, T> ConcaveEnvelope<'a, T>
where
    T: Signed,
{
    fn new(arbitrary: &'a [T], concave: &'a [T]) -> Self {
        let output_len = output_len(arbitrary.len(), concave.len());
        let leaf_count = 1_usize
            .checked_shl(bit_width(output_len))
            .expect("min-plus convolution envelope size must fit usize");
        ConcaveEnvelope {
            arbitrary,
            concave,
            leaf_count,
            query_root: leaf_count >> bit_width(concave.len() - 1),
            node_curves: vec![None; leaf_count],
            result: vec![T::maximum(); output_len],
        }
    }

    #[inline]
    fn value(&self, curve: usize, output: usize) -> T {
        self.arbitrary[curve] + self.concave[output - curve]
    }

    #[inline]
    fn query(&mut self, output: usize) {
        let mut best = self.result[output];
        let mut node = (output + self.leaf_count) >> 1;
        while node >= self.query_root {
            if let Some(curve) = self.node_curves[node] {
                best = best.min(self.value(curve, output));
            }
            node >>= 1;
        }
        self.result[output] = best;
    }

    #[inline]
    fn insert_from_left(&mut self, left: usize) {
        let mut right = left + self.concave.len();
        let block = 1_usize << (left ^ right).ilog2();
        right &= !(block - 1);
        let mut depth = bit_width(right - left - 1);
        let mut node = (self.leaf_count + left) >> depth;
        let mut pending = (!self.arbitrary[left].is_maximum()).then_some(left);
        while depth != 0 {
            let Some(curve) = pending else {
                break;
            };
            depth -= 1;
            let middle = ((node << 1 | 1) << depth) - self.leaf_count - 1;
            if middle < left {
                node = node << 1 | 1;
            } else if self.node_curves[node]
                .is_some_and(|old| self.value(old, middle) < self.value(curve, middle))
            {
                node <<= 1;
            } else {
                std::mem::swap(&mut self.node_curves[node], &mut pending);
                node = node << 1 | 1;
            }
        }
        if let Some(curve) = pending {
            let output = node - self.leaf_count;
            self.result[output] = self.result[output].min(self.value(curve, output));
        }
    }

    #[inline]
    fn insert_from_right(&mut self, right: usize) {
        let curve = right - self.concave.len();
        let block = 1_usize << (curve ^ right).ilog2();
        let left = right & !(block - 1);
        if left == right {
            return;
        }
        let mut depth = bit_width(right - left - 1);
        let mut node = (self.leaf_count + left) >> depth;
        let mut pending = (!self.arbitrary[curve].is_maximum()).then_some(curve);
        while depth != 0 {
            let Some(curve) = pending else {
                break;
            };
            depth -= 1;
            let middle = ((node << 1 | 1) << depth) - self.leaf_count;
            if middle >= right {
                node <<= 1;
            } else if self.node_curves[node]
                .is_some_and(|old| self.value(old, middle) < self.value(curve, middle))
            {
                node = node << 1 | 1;
            } else {
                std::mem::swap(&mut self.node_curves[node], &mut pending);
                node <<= 1;
            }
        }
        if let Some(curve) = pending {
            let output = node - self.leaf_count;
            self.result[output] = self.result[output].min(self.value(curve, output));
        }
    }

    fn convolve(mut self) -> Vec<T> {
        // Curve i is valid on [i, i + concave.len()). The two passes insert
        // opposite sides of each validity interval into the segment envelope.
        for left in 0..self.arbitrary.len() {
            self.insert_from_left(left);
            self.query(left);
        }
        for output in self.arbitrary.len()..self.result.len() {
            self.query(output);
        }

        self.node_curves.fill(None);
        let mut right = self.result.len();
        while right >= self.concave.len() {
            self.insert_from_right(right);
            right -= 1;
            self.query(right);
        }
        for output in 0..self.concave.len() {
            self.query(output);
        }
        self.result
    }
}

/// Computes convolution when one finite input is concave using offline envelopes.
///
/// The running time is `O((n + m) log(n + m))`. The arbitrary input may
/// contain `T::maximum()`.
///
/// # Panics
///
/// Panics unless at least one input is finite and concave, or if the
/// interval-tree size cannot be represented.
pub fn min_plus_convolution_concave_envelope<T>(a: &[T], b: &[T]) -> Vec<T>
where
    T: Signed,
{
    let len = output_len(a.len(), b.len());
    if len == 0 {
        return Vec::new();
    }
    let a_is_concave = !a.iter().any(T::is_maximum) && is_concave(a);
    let b_is_concave = !b.iter().any(T::is_maximum) && is_concave(b);
    let (arbitrary, concave) = if b_is_concave {
        (a, b)
    } else if a_is_concave {
        (b, a)
    } else {
        panic!("at least one min-plus convolution input must be finite and concave")
    };
    concave_envelope(arbitrary, concave)
}

pub(super) fn concave_envelope<T>(arbitrary: &[T], concave: &[T]) -> Vec<T>
where
    T: Signed,
{
    if concave.len() == 1 {
        return arbitrary
            .iter()
            .map(|&value| {
                if value.is_maximum() {
                    T::maximum()
                } else {
                    value + concave[0]
                }
            })
            .collect();
    }
    if arbitrary.len() == 1 {
        return if arbitrary[0].is_maximum() {
            vec![T::maximum(); concave.len()]
        } else {
            concave.iter().map(|&value| arbitrary[0] + value).collect()
        };
    }

    ConcaveEnvelope::new(arbitrary, concave).convolve()
}

/// Computes convolution of two concave inputs from antidiagonal endpoints.
///
/// The running time is `O(n + m)`.
///
/// # Panics
///
/// Panics unless both inputs are finite and concave.
pub fn min_plus_convolution_concave_both<T>(a: &[T], b: &[T]) -> Vec<T>
where
    T: Signed,
{
    let len = output_len(a.len(), b.len());
    if len == 0 {
        return Vec::new();
    }
    assert_finite(a);
    assert_finite(b);
    assert!(
        is_concave(a) && is_concave(b),
        "both inputs must be concave"
    );
    concave_both(a, b)
}

pub(super) fn concave_both<T>(a: &[T], b: &[T]) -> Vec<T>
where
    T: Signed,
{
    let len = output_len(a.len(), b.len());
    let mut result = Vec::with_capacity(len);
    for output in 0..len {
        let first = output.saturating_sub(b.len() - 1);
        let last = output.min(a.len() - 1);
        result.push((a[first] + b[output - first]).min(a[last] + b[output - last]));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{math::min_plus_convolution::min_plus_convolution_naive, tools::Xorshift};

    #[test]
    fn test_concave_envelope_exhaustively() {
        let values = [-2_i64, 0, 3, i64::MAX];
        let mut inputs = vec![Vec::new()];
        for _ in 0..3 {
            let prefixes = inputs.clone();
            for prefix in prefixes {
                for &value in &values {
                    let mut input = prefix.clone();
                    input.push(value);
                    inputs.push(input);
                }
            }
        }
        inputs.sort();
        inputs.dedup();
        let concave: Vec<_> = inputs
            .iter()
            .filter(|input| !input.contains(&i64::MAX) && is_concave(input))
            .collect();
        for arbitrary in &inputs {
            for &structured in &concave {
                assert_eq!(
                    min_plus_convolution_concave_envelope(arbitrary, structured),
                    min_plus_convolution_naive(arbitrary, structured)
                );
            }
        }
    }

    #[test]
    fn test_concave_algorithms_randomly() {
        let mut rng = Xorshift::default();
        for _ in 0..1_000 {
            let arbitrary_len = rng.random(0..=32);
            let concave_len: usize = rng.random(0..=32);
            let arbitrary: Vec<_> = (0..arbitrary_len)
                .map(|_| {
                    if rng.random(0_u64..8) == 0 {
                        i64::MAX
                    } else {
                        rng.random(-50_i64..=50)
                    }
                })
                .collect();
            let mut slopes: Vec<_> = rng
                .random_iter(-20_i64..=20)
                .take(concave_len.saturating_sub(1))
                .collect();
            slopes.sort_unstable_by(|a, b| b.cmp(a));
            let mut concave = Vec::with_capacity(concave_len);
            if concave_len != 0 {
                concave.push(rng.random(-50_i64..=50));
            }
            for slope in slopes {
                concave.push(concave[concave.len() - 1] + slope);
            }
            let other_len: usize = rng.random(0..=32);
            let mut slopes: Vec<_> = rng
                .random_iter(-20_i64..=20)
                .take(other_len.saturating_sub(1))
                .collect();
            slopes.sort_unstable_by(|a, b| b.cmp(a));
            let mut other = Vec::with_capacity(other_len);
            if other_len != 0 {
                other.push(rng.random(-50_i64..=50));
            }
            for slope in slopes {
                other.push(other[other.len() - 1] + slope);
            }
            assert_eq!(
                min_plus_convolution_concave_envelope(&arbitrary, &concave),
                min_plus_convolution_naive(&arbitrary, &concave)
            );
            assert_eq!(
                min_plus_convolution_concave_both(&concave, &other),
                min_plus_convolution_naive(&concave, &other)
            );
        }
    }
}

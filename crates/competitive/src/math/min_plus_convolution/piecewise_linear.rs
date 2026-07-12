use super::{Signed, output_len};
use std::collections::VecDeque;

#[derive(Clone, Copy)]
struct LinearPiece<T> {
    start: usize,
    end: usize,
    slope: T,
    intercept: T,
}

fn pieces<T>(values: &[T]) -> Vec<LinearPiece<T>>
where
    T: Signed + TryFrom<usize>,
{
    match values.len() {
        0 => Vec::new(),
        1 => vec![LinearPiece {
            start: 0,
            end: 0,
            slope: T::zero(),
            intercept: values[0],
        }],
        _ => {
            let mut result = Vec::new();
            let mut start = 0;
            let mut slope = values[1] - values[0];
            for edge in 1..values.len() - 1 {
                let next_slope = values[edge + 1] - values[edge];
                if next_slope != slope {
                    let index = T::try_from(start)
                        .ok()
                        .expect("piecewise-linear index must fit the value type");
                    result.push(LinearPiece {
                        start,
                        end: edge,
                        slope,
                        intercept: values[start] - slope * index,
                    });
                    start = edge;
                    slope = next_slope;
                }
            }
            let index = T::try_from(start)
                .ok()
                .expect("piecewise-linear index must fit the value type");
            result.push(LinearPiece {
                start,
                end: values.len() - 1,
                slope,
                intercept: values[start] - slope * index,
            });
            result
        }
    }
}

fn finite_pieces<T>(values: &[T]) -> Option<Vec<LinearPiece<T>>>
where
    T: Signed + TryFrom<usize>,
{
    if values.iter().any(T::is_maximum) {
        None
    } else {
        Some(pieces(values))
    }
}

fn convolve_piece<T>(arbitrary: &[T], piece: LinearPiece<T>, result: &mut [T])
where
    T: Signed + TryFrom<usize>,
{
    let mut deque = VecDeque::with_capacity(arbitrary.len());
    let mut next_to_add = 0;
    for (output, slot) in result.iter_mut().enumerate() {
        if output < piece.start {
            continue;
        }
        let upper = (output - piece.start).min(arbitrary.len() - 1);
        while next_to_add <= upper {
            if !arbitrary[next_to_add].is_maximum() {
                let index = T::try_from(next_to_add)
                    .ok()
                    .expect("piecewise-linear index must fit the value type");
                let transformed = arbitrary[next_to_add] - piece.slope * index;
                while deque.back().is_some_and(|&(_, value)| value >= transformed) {
                    deque.pop_back();
                }
                deque.push_back((next_to_add, transformed));
            }
            next_to_add += 1;
        }
        let lower = output.saturating_sub(piece.end);
        while deque.front().is_some_and(|&(index, _)| index < lower) {
            deque.pop_front();
        }
        if let Some(&(_, minimum)) = deque.front() {
            let output = T::try_from(output)
                .ok()
                .expect("piecewise-linear output index must fit the value type");
            *slot = (*slot).min(piece.slope * output + piece.intercept + minimum);
        }
    }
}

fn convolve_pieces<T>(
    arbitrary: &[T],
    structured: impl IntoIterator<Item = LinearPiece<T>>,
    len: usize,
) -> Vec<T>
where
    T: Signed + TryFrom<usize>,
{
    let mut result = vec![T::maximum(); len];
    for piece in structured {
        convolve_piece(arbitrary, piece, &mut result);
    }
    result
}

/// Computes convolution when at least one input is finite and linear.
///
/// # Panics
///
/// Panics unless at least one input is finite and linear, or an index cannot
/// be represented by `T`.
pub fn min_plus_convolution_linear<T>(a: &[T], b: &[T]) -> Vec<T>
where
    T: Signed + TryFrom<usize>,
{
    let len = output_len(a.len(), b.len());
    if len == 0 {
        return Vec::new();
    }
    let a_piece = finite_pieces(a).filter(|pieces| pieces.len() == 1);
    let b_piece = finite_pieces(b).filter(|pieces| pieces.len() == 1);
    let (arbitrary, piece) = if let Some(pieces) = b_piece {
        (a, pieces[0])
    } else if let Some(pieces) = a_piece {
        (b, pieces[0])
    } else {
        panic!("at least one min-plus convolution input must be finite and linear")
    };
    convolve_pieces(arbitrary, std::iter::once(piece), len)
}

pub(super) fn linear<T>(arbitrary: &[T], structured: &[T]) -> Vec<T>
where
    T: Signed + TryFrom<usize>,
{
    let len = output_len(arbitrary.len(), structured.len());
    convolve_pieces(arbitrary, pieces(structured), len)
}

/// Computes convolution using the input with fewer maximal linear pieces.
///
/// If that input has `p` pieces, the running time is `O(p * (n + m))`.
///
/// # Panics
///
/// Panics unless at least one input is finite, or an index cannot be
/// represented by `T`.
pub fn min_plus_convolution_piecewise_linear<T>(a: &[T], b: &[T]) -> Vec<T>
where
    T: Signed + TryFrom<usize>,
{
    let len = output_len(a.len(), b.len());
    if len == 0 {
        return Vec::new();
    }
    let a_pieces = finite_pieces(a);
    let b_pieces = finite_pieces(b);
    let (arbitrary, structured) = match (a_pieces, b_pieces) {
        (Some(a_pieces), Some(b_pieces)) if a_pieces.len() < b_pieces.len() => (b, a_pieces),
        (Some(_), Some(b_pieces)) => (a, b_pieces),
        (Some(a_pieces), None) => (b, a_pieces),
        (None, Some(b_pieces)) => (a, b_pieces),
        (None, None) => {
            panic!("at least one min-plus convolution input must be finite")
        }
    };
    convolve_pieces(arbitrary, structured, len)
}

pub(super) fn piecewise_linear<T>(arbitrary: &[T], structured: &[T]) -> Vec<T>
where
    T: Signed + TryFrom<usize>,
{
    let len = output_len(arbitrary.len(), structured.len());
    convolve_pieces(arbitrary, pieces(structured), len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{math::min_plus_convolution::min_plus_convolution_naive, tools::Xorshift};

    #[test]
    fn test_piecewise_linear() {
        let mut rng = Xorshift::default();
        for _ in 0..64 {
            let a_len = rng.random(0..=11);
            let b_len = rng.random(0..=11);
            let arbitrary: Vec<_> = rng.random_iter(-8_i64..=8).take(a_len).collect();
            let slope = rng.random(-8_i64..=8);
            let intercept = rng.random(-8_i64..=8);
            let linear: Vec<_> = (0..b_len)
                .map(|index| slope * index as i64 + intercept)
                .collect();
            assert_eq!(
                min_plus_convolution_linear(&arbitrary, &linear),
                min_plus_convolution_naive(&arbitrary, &linear)
            );
            assert_eq!(
                min_plus_convolution_piecewise_linear(&arbitrary, &linear),
                min_plus_convolution_naive(&arbitrary, &linear)
            );
            if a_len == 0 || b_len == 0 {
                continue;
            }
            let piece_count = rng.random(1..=b_len.min(5));
            let slopes: Vec<_> = rng.random_iter(-8_i64..=8).take(piece_count).collect();
            let mut piecewise = Vec::with_capacity(b_len);
            let mut value = rng.random(-8_i64..=8);
            for index in 0..b_len {
                piecewise.push(value);
                value += slopes[index * piece_count / b_len];
            }
            assert_eq!(
                min_plus_convolution_piecewise_linear(&arbitrary, &piecewise),
                min_plus_convolution_naive(&arbitrary, &piecewise)
            );
        }
    }
}

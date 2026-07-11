use super::{DoublyLinkedList, Zero};
use std::{
    cmp::{Ordering, Reverse},
    ops::{Add, Neg},
};

/// Returns the maximum total score of exactly `k` nonempty disjoint segments for each `k`.
pub fn maximum_scoring_segment_sets<T>(scores: &[T]) -> Vec<T>
where
    T: Clone + Ord + Zero + Add<Output = T> + Neg<Output = T>,
{
    let n = scores.len();
    let zero = T::zero();
    let mut positive_sum = zero.clone();
    let mut nonnegative_count = 0;
    let mut negative_scores = Vec::with_capacity(n);
    let mut blocks: Vec<T> = Vec::with_capacity(n);

    for score in scores {
        let ordering = score.cmp(&zero);
        match ordering {
            Ordering::Greater => {
                positive_sum = positive_sum + score.clone();
                nonnegative_count += 1;
            }
            Ordering::Equal => {
                nonnegative_count += 1;
                continue;
            }
            Ordering::Less => negative_scores.push(Reverse(score.clone())),
        }

        if let Some(last) = blocks.last_mut()
            && ((*last).cmp(&zero) == Ordering::Greater) == (ordering == Ordering::Greater)
        {
            *last = last.clone() + score.clone();
        } else {
            blocks.push(score.clone());
        }
    }

    if blocks.first().is_some_and(|score| score < &zero) {
        blocks.remove(0);
    }
    if blocks.last().is_some_and(|score| score < &zero) {
        blocks.pop();
    }

    let mut result = vec![zero.clone(); n + 1];
    let segment_count = blocks.len().div_ceil(2);

    for value in &mut result[segment_count..=nonnegative_count] {
        value.clone_from(&positive_sum);
    }

    negative_scores.sort_unstable();
    let mut score_with_negatives = positive_sum.clone();
    for (offset, Reverse(score)) in negative_scores.into_iter().enumerate() {
        score_with_negatives = score_with_negatives + score;
        result[nonnegative_count + offset + 1].clone_from(&score_with_negatives);
    }

    if segment_count <= 1 {
        return result;
    }

    let block_count = blocks.len();
    let mut links = DoublyLinkedList::new(block_count);
    for index in 1..block_count {
        links.link(index - 1, index);
    }

    let absolute = |value: &T| {
        if value < &zero {
            -value.clone()
        } else {
            value.clone()
        }
    };
    let mut weights: Vec<_> = blocks.iter().map(|block| Some(absolute(block))).collect();
    let mut candidates: Vec<_> = (0..block_count).collect();

    let mut losses = Vec::with_capacity(segment_count);
    for _ in 1..segment_count {
        let (index, left, right) = loop {
            let index = candidates
                .pop()
                .expect("a live block sequence must have a local minimum");
            if weights[index].is_none() {
                continue;
            }
            let left = links.prev(index);
            let right = links.next(index);
            if (left == usize::MAX || weights[index] <= weights[left])
                && (right == usize::MAX || weights[index] <= weights[right])
            {
                break (index, left, right);
            }
        };
        losses.push(weights[index].take().expect("a local minimum must be live"));

        if left == usize::MAX {
            links.detach(index);
            let (_, next) = links.detach(right);
            weights[right] = None;
            candidates.push(next);
        } else if right == usize::MAX {
            let (prev, _) = links.detach(left);
            links.detach(index);
            weights[left] = None;
            candidates.push(prev);
        } else {
            blocks[index] = blocks[left].clone() + blocks[index].clone() + blocks[right].clone();
            weights[index] = Some(absolute(&blocks[index]));
            let (prev, _) = links.detach(left);
            let (_, next) = links.detach(right);
            weights[left] = None;
            weights[right] = None;
            candidates.extend(
                [prev, index, next]
                    .into_iter()
                    .filter(|&index| index != usize::MAX),
            );
        }
    }

    losses.extend(weights.into_iter().flatten());
    losses.sort_unstable();

    let mut current_score = positive_sum;
    for (result, loss) in result[..segment_count].iter_mut().rev().zip(losses) {
        current_score = current_score + -loss;
        result.clone_from(&current_score);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{rand, tools::Xorshift};

    fn naive(scores: &[i32]) -> Vec<i32> {
        let n = scores.len();
        let mut result = vec![i32::MIN; n + 1];
        result[0] = 0;
        for mask in 1usize..1 << n {
            let mut score = 0;
            let mut selected_count = 0;
            let mut run_count = 0;
            for (index, &value) in scores.iter().enumerate() {
                if mask >> index & 1 == 1 {
                    score += value;
                    selected_count += 1;
                    if index == 0 || mask >> (index - 1) & 1 == 0 {
                        run_count += 1;
                    }
                }
            }
            for value in &mut result[run_count..=selected_count] {
                *value = (*value).max(score);
            }
        }
        result
    }

    #[test]
    fn test_maximum_scoring_segment_sets() {
        for n in 0..=6u32 {
            for mut encoded in 0..5usize.pow(n) {
                let mut scores = Vec::with_capacity(n as usize);
                for _ in 0..n {
                    scores.push((encoded % 5) as i32 - 2);
                    encoded /= 5;
                }
                assert_eq!(maximum_scoring_segment_sets(&scores), naive(&scores));
            }
        }

        const Q: usize = 200;
        const N: usize = 15;
        const A: i32 = 1000;
        let mut rng = Xorshift::default();
        for _ in 0..Q {
            rand!(rng, n: 7..=N, scores: [-A..=A; n]);
            assert_eq!(maximum_scoring_segment_sets(&scores), naive(&scores));
        }
    }
}

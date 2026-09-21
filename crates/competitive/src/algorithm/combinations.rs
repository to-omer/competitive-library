use std::{collections::BTreeSet, mem::swap};

pub trait SliceCombinationsExt<T> {
    fn for_each_product<F>(&self, r: usize, f: F)
    where
        F: FnMut(&[T]);
    fn for_each_permutations<F>(&self, r: usize, f: F)
    where
        F: FnMut(&[T]);
    fn for_each_combinations<F>(&self, r: usize, f: F)
    where
        F: FnMut(&[T]);
    fn for_each_combinations_with_replacement<F>(&self, r: usize, f: F)
    where
        F: FnMut(&[T]);
    fn next_permutation(&mut self) -> bool
    where
        T: Ord;
    fn prev_permutation(&mut self) -> bool
    where
        T: Ord;
    fn next_combination(&mut self, r: usize) -> bool
    where
        T: Ord;
    fn prev_combination(&mut self, r: usize) -> bool
    where
        T: Ord;

    fn apply_permutation(&mut self, permutation: &[usize]);
}

impl<T> SliceCombinationsExt<T> for [T]
where
    T: Clone,
{
    /// choose `r` elements from `n` independently
    ///
    /// # Example
    ///
    /// ```
    /// # use competitive::algorithm::SliceCombinationsExt;
    /// let n = vec![1, 2, 3, 4];
    /// let mut p = Vec::new();
    /// let mut q = Vec::new();
    /// n.for_each_product(2, |v| p.push(v.to_vec()));
    /// for x in n.iter().cloned() {
    ///     for y in n.iter().cloned() {
    ///         q.push(vec![x, y]);
    ///     }
    /// }
    /// assert_eq!(p, q);
    /// ```
    fn for_each_product<F>(&self, r: usize, mut f: F)
    where
        F: FnMut(&[T]),
    {
        fn product_inner<T, F>(n: &[T], mut r: usize, buf: &mut Vec<T>, f: &mut F)
        where
            T: Clone,
            F: FnMut(&[T]),
        {
            if r == 0 {
                f(buf.as_slice());
            } else {
                r -= 1;
                for a in n.iter().cloned() {
                    buf.push(a);
                    product_inner(n, r, buf, f);
                    buf.pop();
                }
            }
        }

        let mut v = Vec::with_capacity(r);
        product_inner(self, r, &mut v, &mut f);
    }

    /// choose `r` elements from `n` independently
    ///
    /// # Example
    ///
    /// ```
    /// # use competitive::algorithm::SliceCombinationsExt;
    /// let n = vec![1, 2, 3, 4];
    /// let mut p = Vec::new();
    /// let mut q = Vec::new();
    /// n.for_each_product(2, |v| p.push(v.to_vec()));
    /// for x in n.iter().cloned() {
    ///     for y in n.iter().cloned() {
    ///         q.push(vec![x, y]);
    ///     }
    /// }
    /// assert_eq!(p, q);
    /// ```
    fn for_each_permutations<F>(&self, r: usize, mut f: F)
    where
        F: FnMut(&[T]),
    {
        fn permutations_inner<T, F>(
            n: &[T],
            mut r: usize,
            rem: &mut BTreeSet<usize>,
            buf: &mut Vec<T>,
            f: &mut F,
        ) where
            T: Clone,
            F: FnMut(&[T]),
        {
            if r == 0 {
                f(buf.as_slice());
            } else {
                r -= 1;
                for i in rem.iter().cloned().collect::<Vec<_>>() {
                    buf.push(n[i].clone());
                    rem.remove(&i);
                    permutations_inner(n, r, rem, buf, f);
                    rem.insert(i);
                    buf.pop();
                }
            }
        }

        if r <= self.len() {
            let mut v = Vec::with_capacity(r);
            let mut rem: BTreeSet<usize> = (0..self.len()).collect();
            permutations_inner(self, r, &mut rem, &mut v, &mut f);
        }
    }

    /// choose distinct `r` elements from `n` in any order
    ///
    /// # Example
    ///
    /// ```
    /// # use competitive::algorithm::SliceCombinationsExt;
    /// let n = vec![1, 2, 3, 4];
    /// let mut p = Vec::new();
    /// let mut q = Vec::new();
    /// n.for_each_permutations(2, |v| p.push(v.to_vec()));
    /// for (i, x) in n.iter().cloned().enumerate() {
    ///     for (j, y) in n.iter().cloned().enumerate() {
    ///         if i != j {
    ///             q.push(vec![x, y]);
    ///         }
    ///     }
    /// }
    /// assert_eq!(p, q);
    /// ```
    fn for_each_combinations<F>(&self, r: usize, mut f: F)
    where
        F: FnMut(&[T]),
    {
        fn combinations_inner<T, F>(
            n: &[T],
            mut r: usize,
            start: usize,
            buf: &mut Vec<T>,
            f: &mut F,
        ) where
            T: Clone,
            F: FnMut(&[T]),
        {
            if r == 0 {
                f(buf.as_slice());
            } else {
                r -= 1;
                for i in start..n.len() - r {
                    buf.push(n[i].clone());
                    combinations_inner(n, r, i + 1, buf, f);
                    buf.pop();
                }
            }
        }

        if r <= self.len() {
            let mut v = Vec::with_capacity(r);
            combinations_inner(self, r, 0, &mut v, &mut f);
        }
    }

    /// choose `r` elements from `n` in sorted order
    ///
    /// # Example
    ///
    /// ```
    /// # use competitive::algorithm::SliceCombinationsExt;
    /// let n = vec![1, 2, 3, 4];
    /// let mut p = Vec::new();
    /// let mut q = Vec::new();
    /// n.for_each_combinations_with_replacement(2, |v| p.push(v.to_vec()));
    /// for (i, x) in n.iter().cloned().enumerate() {
    ///     for y in n[i..].iter().cloned() {
    ///         q.push(vec![x, y]);
    ///     }
    /// }
    /// assert_eq!(p, q);
    /// ```
    fn for_each_combinations_with_replacement<F>(&self, r: usize, mut f: F)
    where
        F: FnMut(&[T]),
    {
        fn combinations_with_replacement_inner<T, F>(
            n: &[T],
            mut r: usize,
            start: usize,
            buf: &mut Vec<T>,
            f: &mut F,
        ) where
            T: Clone,
            F: FnMut(&[T]),
        {
            if r == 0 {
                f(buf.as_slice());
            } else {
                r -= 1;
                for i in start..n.len() {
                    buf.push(n[i].clone());
                    combinations_with_replacement_inner(n, r, i, buf, f);
                    buf.pop();
                }
            }
        }

        let mut v = Vec::with_capacity(r);
        combinations_with_replacement_inner(self, r, 0, &mut v, &mut f);
    }

    /// Permute the elements into next permutation in lexicographical order.
    /// Return whether such a next permutation exists.
    fn next_permutation(&mut self) -> bool
    where
        T: Ord,
    {
        if self.len() < 2 {
            return false;
        }
        let mut target = self.len() - 2;
        while target > 0 && self[target] > self[target + 1] {
            target -= 1;
        }
        if target == 0 && self[target] > self[target + 1] {
            return false;
        }
        let mut next = self.len() - 1;
        while next > target && self[next] < self[target] {
            next -= 1;
        }
        self.swap(next, target);
        self[target + 1..].reverse();
        true
    }

    /// Permute the elements into previous permutation in lexicographical order.
    /// Return whether such a previous permutation exists.
    fn prev_permutation(&mut self) -> bool
    where
        T: Ord,
    {
        if self.len() < 2 {
            return false;
        }
        let mut target = self.len() - 2;
        while target > 0 && self[target] < self[target + 1] {
            target -= 1;
        }
        if target == 0 && self[target] < self[target + 1] {
            return false;
        }
        self[target + 1..].reverse();
        let mut next = self.len() - 1;
        while next > target && self[next - 1] < self[target] {
            next -= 1;
        }
        self.swap(target, next);
        true
    }

    /// Permute the elements into next combination choosing r elements in lexicographical order.
    /// Return whether such a next combination exists.
    fn next_combination(&mut self, r: usize) -> bool
    where
        T: Ord,
    {
        assert!(r <= self.len());
        let (a, b) = self.split_at_mut(r);
        next_combination_inner(a, b)
    }

    /// Permute the elements into previous combination choosing r elements in lexicographical order.
    /// Return whether such a previous combination exists.
    fn prev_combination(&mut self, r: usize) -> bool
    where
        T: Ord,
    {
        assert!(r <= self.len());
        let (a, b) = self.split_at_mut(r);
        next_combination_inner(b, a)
    }

    /// Apply a permutation to the elements.
    /// self[i] <- self[p[i]] for each i
    fn apply_permutation(&mut self, p: &[usize]) {
        assert_eq!(self.len(), p.len());
        let mut visited = vec![false; self.len()];
        for mut current in 0..self.len() {
            if visited[current] {
                continue;
            }
            loop {
                visited[current] = true;
                let next = p[current];
                if visited[next] {
                    break;
                }
                self.swap(current, next);
                current = next;
            }
        }
    }
}

fn rotate_distinct<'a, T>(mut a: &'a mut [T], mut b: &'a mut [T]) {
    while !a.is_empty() && !b.is_empty() {
        if a.len() >= b.len() {
            let (l, r) = a.split_at_mut(b.len());
            l.swap_with_slice(b);
            a = r;
        } else {
            let (l, r) = b.split_at_mut(a.len());
            l.swap_with_slice(a);
            a = l;
            b = r;
        }
    }
}

fn next_combination_inner<T>(a: &mut [T], b: &mut [T]) -> bool
where
    T: Ord,
{
    if a.is_empty() || b.is_empty() {
        return false;
    }
    let mut target = a.len() - 1;
    let last_elem = b.last().unwrap();
    while target > 0 && &a[target] >= last_elem {
        target -= 1;
    }
    if target == 0 && &a[target] >= last_elem {
        rotate_distinct(a, b);
        return false;
    }
    let mut next = 0;
    while a[target] >= b[next] {
        next += 1;
    }
    swap(&mut a[target], &mut b[next]);
    rotate_distinct(&mut a[target + 1..], &mut b[next + 1..]);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn test_enumeration() {
        let mut rng = Xorshift::default();
        for (n, r) in (1usize..=6).flat_map(|n| (0..=6).map(move |r| (n, r))) {
            let values: Vec<_> = rng.random_iter(-10..=10).take(n).collect();
            let mut product = Vec::new();
            let mut permutations = Vec::new();
            let mut combinations = Vec::new();
            let mut replacement = Vec::new();
            for mut code in 0..n.pow(r as u32) {
                let mut indices = vec![0; r];
                for i in indices.iter_mut().rev() {
                    *i = code % n;
                    code /= n;
                }
                let row: Vec<_> = indices.iter().map(|&i| values[i]).collect();
                product.push(row.clone());
                if (0..r).all(|i| !indices[..i].contains(&indices[i])) {
                    permutations.push(row.clone());
                }
                if indices.windows(2).all(|w| w[0] < w[1]) {
                    combinations.push(row.clone());
                }
                if indices.is_sorted() {
                    replacement.push(row);
                }
            }
            let mut actual = Vec::new();
            values.for_each_product(r, |row| actual.push(row.to_vec()));
            assert_eq!(actual, product);
            actual.clear();
            values.for_each_permutations(r, |row| actual.push(row.to_vec()));
            assert_eq!(actual, permutations);
            actual.clear();
            values.for_each_combinations(r, |row| actual.push(row.to_vec()));
            assert_eq!(actual, combinations);
            actual.clear();
            values.for_each_combinations_with_replacement(r, |row| actual.push(row.to_vec()));
            assert_eq!(actual, replacement);
        }
    }

    #[test]
    fn test_next_prev() {
        let mut rng = Xorshift::default();
        for n in 1..=7usize {
            let mut values: Vec<_> = (0..n)
                .map(|i| i as i32 * 100 + rng.random(0..100))
                .collect();
            values.sort();
            values.dedup();
            let n = values.len();
            let mut permutations = Vec::new();
            values.for_each_permutations(n, |row| permutations.push(row.to_vec()));
            let mut p = values.clone();
            for (i, expected) in permutations.iter().enumerate() {
                assert_eq!(&p, expected);
                if i + 1 < permutations.len() {
                    assert!(p.next_permutation());
                    assert!(p.prev_permutation());
                    assert_eq!(&p, expected);
                }
                assert_eq!(p.next_permutation(), i + 1 < permutations.len());
            }
            for r in 0..=n {
                let mut combinations = Vec::new();
                values.for_each_combinations(r, |row| combinations.push(row.to_vec()));
                p = values.clone();
                for (i, expected) in combinations.iter().enumerate() {
                    assert_eq!(&p[..r], expected);
                    if i + 1 < combinations.len() {
                        assert!(p.next_combination(r));
                        assert!(p.prev_combination(r));
                        assert_eq!(&p[..r], expected);
                    }
                    assert_eq!(p.next_combination(r), i + 1 < combinations.len());
                }
            }
        }
    }

    #[test]
    fn test_apply_permutation() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            let n = rng.random(1..100);
            let a: Vec<_> = rng.random_iter(0..1_000).take(n).collect();
            let mut p: Vec<usize> = (0..n).collect();
            rng.shuffle(&mut p);
            let expected: Vec<_> = p.iter().map(|&i| a[i]).collect();
            let mut result = a.to_vec();
            result.apply_permutation(&p);
            assert_eq!(expected, result);
        }
    }
}

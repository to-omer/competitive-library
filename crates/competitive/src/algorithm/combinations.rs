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
    fn test_for_each_product() {
        for n in 1..=6 {
            let values: Vec<i32> = (0..n).collect();
            for r in 0..=6 {
                let mut result = vec![];
                values
                    .as_slice()
                    .for_each_product(r, |cur| result.push(cur.to_vec()));
                let mut expected = vec![];
                let mut current = vec![0; r];
                'outer: loop {
                    expected.push(current.clone());
                    for i in (0..r).rev() {
                        if current[i] + 1 < n {
                            current[i] += 1;
                            for c in &mut current[i + 1..] {
                                *c = 0;
                            }
                            continue 'outer;
                        }
                    }
                    break;
                }
                assert_eq!(result, expected);
            }
        }
    }

    #[test]
    fn test_for_each_permutations_small_cases() {
        for n in 1..=6 {
            let values: Vec<i32> = (0..n).collect();
            for r in 0..=6 {
                let mut result = vec![];
                values
                    .as_slice()
                    .for_each_permutations(r, |cur| result.push(cur.to_vec()));
                let mut expected = vec![];
                let mut current = vec![0; r];
                'outer: loop {
                    let ok = {
                        let mut current = current.clone();
                        current.sort_unstable();
                        current.dedup();
                        current.len() == r
                    };
                    if ok {
                        expected.push(current.clone());
                    }
                    for i in (0..r).rev() {
                        if current[i] + 1 < n {
                            current[i] += 1;
                            for c in &mut current[i + 1..] {
                                *c = 0;
                            }
                            continue 'outer;
                        }
                    }
                    break;
                }
                assert_eq!(result, expected);
            }
        }
    }

    #[test]
    fn test_for_each_combinations_small_cases() {
        for n in 1..=6 {
            let values: Vec<i32> = (0..n).collect();
            for r in 0..=6 {
                let mut result = vec![];
                values
                    .as_slice()
                    .for_each_combinations(r, |cur| result.push(cur.to_vec()));
                let mut expected = vec![];
                let mut current = vec![0; r];
                'outer: loop {
                    let ok = {
                        let mut current = current.clone();
                        current.dedup();
                        current.len() == r && current.is_sorted()
                    };
                    if ok {
                        expected.push(current.clone());
                    }
                    for i in (0..r).rev() {
                        if current[i] + 1 < n {
                            current[i] += 1;
                            for c in &mut current[i + 1..] {
                                *c = 0;
                            }
                            continue 'outer;
                        }
                    }
                    break;
                }
                assert_eq!(result, expected);
            }
        }
    }

    #[test]
    fn test_for_each_combinations_with_replacement_small_cases() {
        for n in 1..=6 {
            let values: Vec<i32> = (0..n).collect();
            for r in 0..=6 {
                let mut result = vec![];
                values
                    .as_slice()
                    .for_each_combinations_with_replacement(r, |cur| result.push(cur.to_vec()));
                let mut expected = vec![];
                let mut current = vec![0; r];
                'outer: loop {
                    let ok = {
                        let current = current.clone();
                        current.is_sorted()
                    };
                    if ok {
                        expected.push(current.clone());
                    }
                    for i in (0..r).rev() {
                        if current[i] + 1 < n {
                            current[i] += 1;
                            for c in &mut current[i + 1..] {
                                *c = 0;
                            }
                            continue 'outer;
                        }
                    }
                    break;
                }
                assert_eq!(result, expected);
            }
        }
    }

    #[test]
    fn test_next_prev_permutation() {
        for n in 1..=7 {
            let mut p: Vec<_> = (0..n).collect();
            let mut a = vec![];
            p.for_each_permutations(n, |p| a.push(p.to_vec()));
            let mut b = vec![];
            loop {
                b.push(p.to_vec());
                if !p.next_permutation() {
                    break;
                }
                assert!(p.prev_permutation());
                assert_eq!(b.last().as_ref().unwrap().as_slice(), &p);
                assert!(p.next_permutation());
            }
            assert_eq!(a, b);
        }
    }

    #[test]
    fn test_next_prev_combination() {
        for n in 1..=7 {
            for r in 0..=n {
                let mut p: Vec<_> = (0..n).collect();
                let mut a = vec![];
                p.for_each_combinations(r, |p| a.push(p.to_vec()));
                let mut b = vec![];
                loop {
                    b.push(p[..r].to_vec());
                    if !p.next_combination(r) {
                        break;
                    }
                    assert!(p.prev_combination(r));
                    assert_eq!(b.last().as_ref().unwrap().as_slice(), &p[..r]);
                    assert!(p.next_combination(r));
                }
                assert_eq!(a, b);
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

use super::UnionFind;
use std::mem::{swap, take};

/// Enumerates a finite monoid defined by word equations using the HLT strategy.
#[derive(Debug, Clone)]
pub struct ToddCoxeter {
    relations: Vec<(Vec<usize>, Vec<usize>)>,
    table: Vec<Vec<usize>>,
    uf: UnionFind,
}

impl ToddCoxeter {
    /// Generators are numbered `0..alphabet_size`; the empty word is the identity.
    pub fn new(alphabet_size: usize) -> Self {
        Self {
            relations: Vec::new(),
            table: vec![vec![usize::MAX; alphabet_size]],
            uf: UnionFind::new(1),
        }
    }

    /// Adds an equation valid in every left and right context, not a directed rewrite.
    pub fn add_relation<I, J>(&mut self, lhs: I, rhs: J)
    where
        I: IntoIterator<Item = usize>,
        J: IntoIterator<Item = usize>,
    {
        let lhs: Vec<_> = lhs.into_iter().collect();
        let rhs: Vec<_> = rhs.into_iter().collect();
        assert!(lhs.iter().chain(&rhs).all(|&c| c < self.table[0].len()));
        self.relations.push((lhs, rhs));
    }

    /// Returns the right generator transitions `table[state][generator]`.
    /// State 0 is the identity; states follow breadth-first order with ascending generators.
    ///
    /// The presented monoid must be finite.
    ///
    /// With P created states, K generators, and L the sum of `1 + lhs.len() + rhs.len()`
    /// over the equations, time is O(P(1 + K + L) alpha(P)) and space O(P(1 + K) + L).
    /// P can greatly exceed the final number of states and depends on equation order.
    ///
    /// ```
    /// use competitive::algorithm::ToddCoxeter;
    /// let mut tc = ToddCoxeter::new(2);
    /// tc.add_relation([0, 0], []);
    /// tc.add_relation([1, 1, 1], []);
    /// tc.add_relation([0, 1, 0, 1], []);
    /// let table = tc.solve();
    /// assert_eq!(table.len(), 6);
    /// assert_eq!([1, 0, 1].into_iter().fold(0, |q, c| table[q][c]), table[0][0]);
    /// ```
    pub fn solve(mut self) -> Vec<Vec<usize>> {
        let relations = take(&mut self.relations);
        let mut stack = Vec::new();
        let alphabet_size = self.table[0].len();
        let mut state = 0;
        while state < self.table.len() {
            if self.uf.find_root(state) == state {
                for (lhs, rhs) in &relations {
                    let (lc, lp) = lhs
                        .split_last()
                        .map_or((None, &[][..]), |(&c, p)| (Some(c), p));
                    let (rc, rp) = rhs
                        .split_last()
                        .map_or((None, &[][..]), |(&c, p)| (Some(c), p));
                    let left = self.follow(state, lp);
                    let right = self.follow(state, rp);
                    // Both prefixes must exist before reading either last edge (e.g. a = aa).
                    let x = lc.map_or(left, |c| self.table[left][c]);
                    let y = rc.map_or(right, |c| self.table[right][c]);
                    if x == usize::MAX {
                        let y = if y == usize::MAX {
                            self.define(right, rc.unwrap())
                        } else {
                            y
                        };
                        self.table[left][lc.unwrap()] = y;
                    } else if y == usize::MAX {
                        self.table[right][rc.unwrap()] = x;
                    } else {
                        self.merge(x, y, &mut stack);
                    }
                }
                for c in 0..alphabet_size {
                    self.define(state, c);
                }
            }
            state += 1;
        }

        let identity = self.uf.find_root(0);
        let mut order = vec![identity];
        let mut index = vec![usize::MAX; self.table.len()];
        index[identity] = 0;
        let mut table = Vec::new();
        let mut i = 0;
        while i < order.len() {
            let mut row = Vec::with_capacity(alphabet_size);
            for &target in &self.table[order[i]] {
                let target = self.uf.find_root(target);
                if index[target] == usize::MAX {
                    index[target] = order.len();
                    order.push(target);
                }
                row.push(index[target]);
            }
            table.push(row);
            i += 1;
        }
        table
    }

    fn define(&mut self, state: usize, c: usize) -> usize {
        let state = self.uf.find_root(state);
        if self.table[state][c] == usize::MAX {
            assert!(self.table.len() < i32::MAX as usize);
            self.table[state][c] = self.table.len();
            self.table.push(vec![usize::MAX; self.table[0].len()]);
            self.uf.push();
        }
        self.uf.find_root(self.table[state][c])
    }

    fn follow(&mut self, state: usize, word: &[usize]) -> usize {
        let mut state = self.uf.find_root(state);
        for &c in word {
            state = self.define(state, c);
        }
        state
    }

    fn merge(&mut self, x: usize, y: usize, stack: &mut Vec<(usize, usize)>) {
        stack.push((x, y));
        while let Some((x, y)) = stack.pop() {
            let mut x = self.uf.find_root(x);
            let mut y = self.uf.find_root(y);
            if !self.uf.unite(x, y) {
                continue;
            }
            if self.uf.find_root(x) == y {
                swap(&mut x, &mut y);
            }
            for c in 0..self.table[0].len() {
                let a = self.table[x][c];
                let b = self.table[y][c];
                if a == usize::MAX {
                    self.table[x][c] = b;
                } else if b != usize::MAX {
                    stack.push((a, b));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        math::Fps998244353,
        num::montgomery::MInt998244353 as M,
        tools::{Xorshift, testutil::exhaustive_sequences},
    };
    use std::{collections::HashMap, iter::repeat_n};

    fn transformation_table(
        degree: usize,
        generators: &[Vec<usize>],
    ) -> (Vec<Vec<usize>>, Vec<Vec<usize>>) {
        let identity: Vec<_> = (0..degree).collect();
        let mut index = HashMap::from([(identity.clone(), 0)]);
        let mut elements = vec![identity];
        let mut words = vec![vec![]];
        let mut table = Vec::new();
        let mut state = 0;
        while state < elements.len() {
            let mut row = Vec::new();
            for (c, generator) in generators.iter().enumerate() {
                let next: Vec<_> = elements[state].iter().map(|&x| generator[x]).collect();
                let target = *index.entry(next.clone()).or_insert_with(|| {
                    let target = elements.len();
                    elements.push(next);
                    let mut word = words[state].clone();
                    word.push(c);
                    words.push(word);
                    target
                });
                row.push(target);
            }
            table.push(row);
            state += 1;
        }
        (table, words)
    }

    #[test]
    fn test_todd_coxeter_cyclic_monoid() {
        for period_end in 1..=128 {
            for cycle_start in 0..period_end {
                let mut tc = ToddCoxeter::new(1);
                let mut lhs = vec![0; cycle_start];
                let mut rhs = vec![0; period_end];
                if (cycle_start + period_end) % 2 == 0 {
                    swap(&mut lhs, &mut rhs);
                }
                tc.add_relation(lhs, rhs);
                let table = tc.solve();
                let expected: Vec<_> = (0..period_end)
                    .map(|i| {
                        let target = if i + 1 == period_end {
                            cycle_start
                        } else {
                            i + 1
                        };
                        vec![target]
                    })
                    .collect();
                assert_eq!(table, expected);
            }
        }
    }

    #[test]
    fn test_todd_coxeter_transformation_monoid() {
        let mut rng = Xorshift::default();
        let maps: Vec<_> = exhaustive_sequences(0..3, 3..=3).collect();
        let small = exhaustive_sequences(0..maps.len(), 0..=2)
            .map(|ids| (3, ids.into_iter().map(|i| maps[i].clone()).collect()));
        let full = (2..=4).map(|n| {
            let cycle = (0..n).map(|i| (i + 1) % n).collect();
            let mut transposition: Vec<_> = (0..n).collect();
            transposition.swap(0, 1);
            let mut projection: Vec<_> = (0..n).collect();
            projection[1] = 0;
            (n, vec![cycle, transposition, projection])
        });
        for (degree, generators) in small.chain(full) {
            let (expected, words) = transformation_table(degree, &generators);
            let mut relations = Vec::new();
            for (state, row) in expected.iter().enumerate() {
                for (c, &target) in row.iter().enumerate() {
                    let mut word = words[state].clone();
                    word.push(c);
                    relations.push((word, words[target].clone()));
                }
            }
            // Redundant equations exercise overlapping prefixes and different scan orders.
            for word in exhaustive_sequences(0..generators.len(), 0..=3) {
                let target = word.iter().fold(0, |q, &c| expected[q][c]);
                relations.push((word, words[target].clone()));
            }
            rng.shuffle(&mut relations);
            let mut tc = ToddCoxeter::new(generators.len());
            for (mut lhs, mut rhs) in relations {
                if rng.gen_bool(0.5) {
                    swap(&mut lhs, &mut rhs);
                }
                tc.add_relation(lhs, rhs);
            }
            assert_eq!(tc.solve(), expected);
        }
    }

    #[test]
    fn test_todd_coxeter_symmetric_group() {
        for n in 1..=6 {
            let generators: Vec<_> = (0..n - 1)
                .map(|i| {
                    let mut permutation: Vec<_> = (0..n).collect();
                    permutation.swap(i, i + 1);
                    permutation
                })
                .collect();
            let (expected, _) = transformation_table(n, &generators);
            let mut tc = ToddCoxeter::new(generators.len());
            for i in 0..generators.len() {
                tc.add_relation([i, i], []);
                for j in i + 1..generators.len() {
                    if j == i + 1 {
                        tc.add_relation([i, j, i], [j, i, j]);
                    } else {
                        tc.add_relation([i, j], [j, i]);
                    }
                }
            }
            assert_eq!(tc.solve(), expected);
        }
    }

    #[test]
    fn test_todd_coxeter_trivial_monoid() {
        assert_eq!(ToddCoxeter::new(0).solve(), vec![vec![]]);
        let mut tc = ToddCoxeter::new(1);
        tc.add_relation(repeat_n(0, 4096), []);
        tc.add_relation([0], []);
        assert_eq!(tc.solve(), vec![vec![0]]);
    }

    #[test]
    fn test_todd_coxeter_counting() {
        let generators = [
            vec![vec![0], vec![0]],
            vec![vec![1, 0, 2], vec![1, 2, 0]],
            vec![vec![1, 0, 3, 2], vec![1, 2, 0, 3]],
            vec![vec![1, 0, 2, 3], vec![0, 2, 3, 1]],
            vec![vec![1, 0, 3, 2, 4], vec![0, 2, 4, 3, 1]],
        ];
        for (i, generators) in generators.iter().enumerate() {
            let (expected, _) = transformation_table(generators[0].len(), generators);
            let mut tc = ToddCoxeter::new(2);
            tc.add_relation([0, 0], []);
            tc.add_relation([1, 1, 1], []);
            tc.add_relation((0..=i).flat_map(|_| [0, 1]), []);
            let table = tc.solve();
            assert_eq!(table, expected);

            let target = [0, 1, 0].into_iter().fold(0, |q, c| table[q][c]);
            let mut dp = vec![M::new(0); table.len()];
            dp[0] = M::new(1);
            let mut terms = Vec::new();
            for n in 0..1000 {
                if n < 2 * table.len() {
                    terms.push(dp[target]);
                }
                let mut next = vec![M::new(0); table.len()];
                for (row, &count) in table.iter().zip(&dp) {
                    for &q in row {
                        next[q] += count;
                    }
                }
                dp = next;
            }
            assert_eq!(Fps998244353::kth_term(terms, 1000), dp[target]);
            if table.len() == 1 {
                assert_eq!(dp[0], M::new(2).pow(1000));
            }
        }
    }
}

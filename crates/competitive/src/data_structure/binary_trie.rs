use super::{AbelianMonoid, LazyMapMonoid};
use std::{
    mem::replace,
    ops::{Bound, RangeBounds},
};

struct Node<M>
where
    M: LazyMapMonoid,
{
    child: [usize; 2],
    parent: usize,
    agg: M::Agg,
    lazy: M::Act,
}

impl<M> Node<M>
where
    M: LazyMapMonoid,
{
    fn new(parent: usize) -> Self {
        Self {
            child: [usize::MAX; 2],
            parent,
            agg: M::agg_unit(),
            lazy: M::act_unit(),
        }
    }
}

pub struct BinaryTrie<M>
where
    M: LazyMapMonoid,
{
    bit_len: usize,
    max_key: u64,
    len: usize,
    xor_mask: u64,
    nodes: Vec<Node<M>>,
}

impl<M> BinaryTrie<M>
where
    M: LazyMapMonoid,
{
    pub fn new(bit_len: usize) -> Self {
        Self::with_capacity(bit_len, 0)
    }

    pub fn with_capacity(bit_len: usize, capacity: usize) -> Self {
        assert!(bit_len <= 64);
        let max_key = if bit_len == 64 {
            u64::MAX
        } else {
            (1u64 << bit_len) - 1
        };
        let mut nodes = Vec::with_capacity(
            capacity
                .saturating_mul(bit_len.saturating_add(1))
                .saturating_add(1),
        );
        nodes.push(Node::new(usize::MAX));
        Self {
            bit_len,
            max_key,
            len: 0,
            xor_mask: 0,
            nodes,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.len = 0;
        self.xor_mask = 0;
        self.nodes.clear();
        self.nodes.push(Node::new(usize::MAX));
    }

    pub fn set(&mut self, key: u64, value: M::Agg) {
        self.modify_or_insert(key, |x| *x = value);
    }

    pub fn modify_or_insert(&mut self, key: u64, f: impl FnOnce(&mut M::Agg)) {
        assert!(key <= self.max_key);
        if self.bit_len == 0 {
            if self.is_empty() {
                self.len = 1;
            }
            f(&mut self.nodes[0].agg);
            return;
        }

        let key = key ^ self.xor_mask;
        let mut inserted = false;
        let mut node = 0;
        for d in (0..self.bit_len).rev() {
            self.push_at(node, d + 1);
            let bit = ((key >> d) & 1) as usize;
            if self.nodes[node].child[bit] == usize::MAX {
                inserted = true;
                let next = self.nodes.len();
                self.nodes[node].child[bit] = next;
                self.nodes.push(Node::new(node));
            }
            node = self.nodes[node].child[bit];
        }

        if inserted {
            self.len += 1;
        }
        self.nodes[node].lazy = M::act_unit();
        f(&mut self.nodes[node].agg);
        self.recalc_up(node);
    }

    pub fn get(&mut self, key: u64) -> Option<M::Agg> {
        assert!(key <= self.max_key);
        if self.is_empty() {
            return None;
        }
        if self.bit_len == 0 {
            return Some(self.nodes[0].agg.clone());
        }

        let key = key ^ self.xor_mask;
        let mut node = 0;
        for d in (0..self.bit_len).rev() {
            let bit = ((key >> d) & 1) as usize;
            let next = self.nodes[node].child[bit];
            if next == usize::MAX {
                return None;
            }
            self.push_at(node, d + 1);
            node = next;
        }
        Some(self.nodes[node].agg.clone())
    }

    pub fn update<R>(&mut self, range: R, act: M::Act)
    where
        R: RangeBounds<u64>,
    {
        let Some(range) = self.range_to_bounds(range) else {
            return;
        };
        if self.is_empty() {
            return;
        }

        let (ql, qr) = range;
        if ql == 0 && qr == self.max_key {
            self.apply_at(0, self.bit_len, &act);
            return;
        }

        let mut l = ql;
        loop {
            let depth = (l.trailing_zeros() as usize)
                .min(self.bit_len)
                .min(63 - (qr - l + 1).leading_zeros() as usize);
            let r = l | ((1u64 << depth) - 1);

            let mut node = 0;
            for d in (depth..self.bit_len).rev() {
                self.push_at(node, d + 1);
                node = self.nodes[node].child[(((l ^ self.xor_mask) >> d) & 1) as usize];
                if node == usize::MAX {
                    break;
                }
            }
            if node != usize::MAX {
                self.apply_at(node, depth, &act);
                self.recalc_up(node);
            }
            if r == qr {
                break;
            }
            l = r + 1;
        }
    }

    pub fn fold<R>(&mut self, range: R) -> M::Agg
    where
        R: RangeBounds<u64>,
    {
        let Some(range) = self.range_to_bounds(range) else {
            return M::agg_unit();
        };

        let (ql, qr) = range;
        if ql == 0 && qr == self.max_key {
            return self.nodes[0].agg.clone();
        }

        let mut res = M::agg_unit();
        let mut l = ql;
        loop {
            let depth = (l.trailing_zeros() as usize)
                .min(self.bit_len)
                .min(63 - (qr - l + 1).leading_zeros() as usize);
            let r = l | ((1u64 << depth) - 1);

            let mut node = 0;
            for d in (depth..self.bit_len).rev() {
                self.push_at(node, d + 1);
                node = self.nodes[node].child[(((l ^ self.xor_mask) >> d) & 1) as usize];
                if node == usize::MAX {
                    break;
                }
            }
            if node != usize::MAX {
                res = M::agg_operate(&res, &self.nodes[node].agg);
            }
            if r == qr {
                break;
            }
            l = r + 1;
        }
        res
    }

    fn apply_at(&mut self, node: usize, depth: usize, act: &M::Act) {
        if M::is_act_unit(act) {
            return;
        }
        if let Some(agg) = M::act_agg(&self.nodes[node].agg, act) {
            self.nodes[node].agg = agg;
            if depth > 0 {
                M::act_operate_assign(&mut self.nodes[node].lazy, act);
            }
        } else if depth == 0 {
            panic!("act failed on leaf");
        } else {
            self.push_at(node, depth);
            for child in self.nodes[node].child {
                if child != usize::MAX {
                    self.apply_at(child, depth - 1, act);
                }
            }
            self.recalc_at(node);
        }
    }

    fn push_at(&mut self, node: usize, depth: usize) {
        let act = replace(&mut self.nodes[node].lazy, M::act_unit());
        if M::is_act_unit(&act) {
            return;
        }
        let child = self.nodes[node].child;
        for child in child {
            if child != usize::MAX {
                self.apply_at(child, depth - 1, &act);
            }
        }
    }

    fn recalc_at(&mut self, node: usize) {
        let mut agg = M::agg_unit();
        for child in self.nodes[node].child {
            if child != usize::MAX {
                agg = M::agg_operate(&agg, &self.nodes[child].agg);
            }
        }
        self.nodes[node].agg = agg;
    }

    fn recalc_up(&mut self, mut node: usize) {
        while self.nodes[node].parent != usize::MAX {
            node = self.nodes[node].parent;
            self.recalc_at(node);
        }
    }

    fn range_to_bounds<R>(&self, range: R) -> Option<(u64, u64)>
    where
        R: RangeBounds<u64>,
    {
        let start = match range.start_bound() {
            Bound::Included(&x) => {
                assert!(x <= self.max_key || (self.bit_len < 64 && x == self.max_key + 1));
                if x <= self.max_key { Some(x) } else { None }
            }
            Bound::Excluded(&x) => {
                assert!(x <= self.max_key);
                (x < self.max_key).then_some(x + 1)
            }
            Bound::Unbounded => Some(0),
        };
        let end = match range.end_bound() {
            Bound::Included(&x) => {
                assert!(x <= self.max_key);
                Some(x)
            }
            Bound::Excluded(&x) => {
                if x == 0 {
                    None
                } else {
                    assert!(self.bit_len == 64 || x <= self.max_key + 1);
                    Some((x - 1).min(self.max_key))
                }
            }
            Bound::Unbounded => Some(self.max_key),
        };
        if let (Some(start), Some(end)) = (start, end) {
            (start <= end).then_some((start, end))
        } else {
            None
        }
    }
}

impl<M> BinaryTrie<M>
where
    M: LazyMapMonoid,
    M::AggMonoid: AbelianMonoid,
{
    pub fn xor_all(&mut self, mask: u64) {
        assert!(mask <= self.max_key);
        self.xor_mask ^= mask;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{
            AdditiveOperation, Associative, FlattenAct, LazyMapMonoid, Magma, RangeSumRangeAdd,
            Unital,
        },
        tools::Xorshift,
    };
    use std::{
        collections::BTreeMap,
        marker::PhantomData,
        ops::{Bound, RangeBounds},
    };

    #[test]
    fn binary_trie_range_sum_randomized() {
        const A: i64 = 100;
        const Q: usize = 4_000;
        let mut rng = Xorshift::default();

        for bit_len in [0, 1, 6, 64] {
            let mut trie = BinaryTrie::<RangeSumRangeAdd<i64>>::new(bit_len);
            let mut map = BTreeMap::new();
            let universe = (bit_len < 64).then(|| 1u64 << bit_len);
            let max_key = universe.map_or(u64::MAX, |n| n - 1);

            for _ in 0..Q {
                let key = match universe {
                    Some(n) => rng.random(0..n),
                    None => rng.random(..),
                };
                let mut x = match universe {
                    Some(_) => rng.random(0..=max_key),
                    None => rng.random(..),
                };
                let mut y = match universe {
                    Some(_) => rng.random(0..=max_key),
                    None => rng.random(..),
                };
                if x > y {
                    std::mem::swap(&mut x, &mut y);
                }
                let range = (
                    match rng.random(0..3) {
                        0 => Bound::Excluded(x),
                        1 => Bound::Included(x),
                        _ => Bound::Unbounded,
                    },
                    match rng.random(0..3) {
                        0 => Bound::Excluded(y),
                        1 => Bound::Included(y),
                        _ => Bound::Unbounded,
                    },
                );
                match rng.random(0..10) {
                    0 => {
                        let value = (rng.random(-A..=A), rng.random(0i64..=5));
                        trie.set(key, value);
                        map.insert(key, value);
                    }
                    1 => {
                        let dx = rng.random(-A..=A);
                        let dy = rng.random(0i64..=3);
                        trie.modify_or_insert(key, |value| {
                            value.0 += dx;
                            value.1 += dy;
                        });
                        let value = map.entry(key).or_insert((0, 0));
                        value.0 += dx;
                        value.1 += dy;
                    }
                    2 => {
                        assert_eq!(trie.get(key), map.get(&key).copied());
                    }
                    3 => {
                        let add = rng.random(-A..=A);
                        trie.update(range, add);
                        for (key, value) in map.iter_mut() {
                            if range.contains(key) {
                                value.0 += add * value.1;
                            }
                        }
                    }
                    4 => {
                        assert_eq!(
                            trie.fold(range),
                            map.iter()
                                .filter(|(key, _)| range.contains(key))
                                .fold((0, 0), |(sx, sy), (_, &(x, y))| (sx + x, sy + y))
                        );
                    }
                    5 => {
                        let add = rng.random(-A..=A);
                        trie.update(.., add);
                        for value in map.values_mut() {
                            value.0 += add * value.1;
                        }
                    }
                    6 => {
                        assert_eq!(
                            trie.fold(..),
                            map.values()
                                .fold((0, 0), |(sx, sy), &(x, y)| (sx + x, sy + y))
                        );
                    }
                    7 => {
                        let add = rng.random(-A..=A);
                        trie.update(..=max_key, add);
                        for value in map.values_mut() {
                            value.0 += add * value.1;
                        }
                    }
                    8 => {
                        assert_eq!(
                            trie.fold(max_key..=max_key),
                            map.get(&max_key).copied().unwrap_or((0, 0))
                        );
                    }
                    _ => {
                        let mask = match universe {
                            Some(n) => rng.random(0..n),
                            None => rng.random(..),
                        };
                        trie.xor_all(mask);
                        map = map
                            .into_iter()
                            .map(|(key, value)| (key ^ mask, value))
                            .collect();
                    }
                }
                assert_eq!(trie.len(), map.len());
                assert_eq!(trie.is_empty(), map.is_empty());
            }

            trie.clear();
            map.clear();
            assert_eq!(trie.fold(..), (0, 0));
            assert!(trie.is_empty());
        }
    }

    struct Concat;

    impl Magma for Concat {
        type T = Vec<i32>;

        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            let mut res = x.clone();
            res.extend(y);
            res
        }
    }

    impl Associative for Concat {}

    impl Unital for Concat {
        fn unit() -> Self::T {
            Vec::new()
        }
    }

    struct DescendAdd {
        _marker: PhantomData<fn()>,
    }

    impl LazyMapMonoid for DescendAdd {
        type Key = i32;
        type Agg = Vec<i32>;
        type Act = i32;
        type AggMonoid = Concat;
        type ActMonoid = AdditiveOperation<i32>;
        type KeyAct = FlattenAct<AdditiveOperation<i32>>;

        fn single_agg(key: &Self::Key) -> Self::Agg {
            vec![*key]
        }

        fn act_agg(x: &Self::Agg, a: &Self::Act) -> Option<Self::Agg> {
            Self::is_act_unit(a)
                .then_some(x.clone())
                .or_else(|| (x.len() <= 1).then(|| x.iter().map(|x| x + a).collect()))
        }
    }

    #[test]
    fn binary_trie_non_commutative_descending_lazy_randomized() {
        const B: usize = 5;
        const Q: usize = 2_000;
        let mut rng = Xorshift::default();
        let mut trie = BinaryTrie::<DescendAdd>::new(B);
        let mut map = BTreeMap::<u64, Vec<i32>>::new();
        let universe = 1u64 << B;

        for _ in 0..Q {
            let key = rng.random(0..universe);
            let l = rng.random(0..=universe);
            let r = rng.random(l..=universe);
            match rng.random(0..5) {
                0 => {
                    let value = vec![rng.random(-100..=100)];
                    trie.set(key, value.clone());
                    map.insert(key, value);
                }
                1 => {
                    let value = rng.random(-100..=100);
                    trie.modify_or_insert(key, |bucket| {
                        if bucket.is_empty() {
                            bucket.push(value);
                        } else {
                            bucket[0] += value;
                        }
                    });
                    map.entry(key)
                        .and_modify(|bucket| bucket[0] += value)
                        .or_insert_with(|| vec![value]);
                }
                2 => {
                    let add = rng.random(-100..=100);
                    trie.update(l..r, add);
                    for (_, bucket) in map.range_mut(l..r) {
                        for value in bucket {
                            *value += add;
                        }
                    }
                }
                3 => {
                    assert_eq!(trie.get(key), map.get(&key).cloned());
                }
                _ => {
                    let expected = map
                        .range(l..r)
                        .flat_map(|(_, value)| value.iter().copied())
                        .collect::<Vec<_>>();
                    assert_eq!(trie.fold(l..r), expected);
                }
            }
            assert_eq!(trie.len(), map.len());
        }
    }
}

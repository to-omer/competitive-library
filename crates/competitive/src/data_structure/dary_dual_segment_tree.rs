#[cfg(target_arch = "x86_64")]
use super::prefetch_block;
use super::{RangeBoundsExt, SimdBackend, simd_backend};
use super::{SimdMonoidAct, simd_monoid, simd_monoid_act::simd_act_block};

#[codesnip::entry("DaryDualSegmentTree")]
pub use self::dary_dual_tree::{DaryDualKernel, DaryDualSegmentTree};

#[codesnip::entry("DaryDualSegmentTree", include("avx_helper", "discrete_steps"))]
mod dary_dual_tree {
    #![allow(unsafe_op_in_unsafe_fn)] // Construction selects a supported ISA for block operations.
    #[cfg(target_arch = "x86_64")]
    use super::prefetch_block;
    use super::{RangeBoundsExt, SimdBackend, simd_backend};
    use std::{
        fmt::{self, Debug, Formatter},
        ops::{Range, RangeBounds},
    };

    /// SIMD blocks for range actions and point access, without aggregates.
    ///
    /// B is a power of two greater than one. `compose(old,later)` preserves action
    /// order. Leaves retain all supplied keys; branch initializes B identity tags.
    /// Branches contain only child tags. Editing a block
    /// first applies carry to every valid lane, then action to full, then clears the
    /// tags of the boundary children in edges. Those children were already edited.
    ///
    /// # Safety
    /// ISA is 0 (portable), 1 (AVX2), or 2 (AVX-512F/DQ/CD/BW/VL), supported by the CPU.
    /// Kernels check any additional ISA requirements and support portable execution
    /// on other architectures. Unsafe methods receive len <= B and valid ranges/lanes.
    pub trait DaryDualKernel {
        const B: usize;
        type Key;
        type Value: Copy;
        type Output;
        type Action;
        type Tag: Copy;
        type Leaf;
        type Branch;
        fn encode(key: Self::Key) -> Self::Value;
        fn decode(value: Self::Value) -> Self::Output;
        fn prepare(action: Self::Action) -> Self::Tag;
        fn unit() -> Self::Tag;
        fn compose(old: Self::Tag, later: Self::Tag) -> Self::Tag;
        fn apply(value: Self::Value, tag: Self::Tag) -> Self::Value;
        fn leaf(values: impl Iterator<Item = Self::Value>) -> Self::Leaf;
        fn branch() -> Self::Branch;
        fn get(node: &Self::Leaf, lane: usize) -> Self::Value;
        fn tag(node: &Self::Branch, lane: usize) -> Self::Tag;
        /// # Safety
        /// ISA, len, and range must satisfy the block contract.
        unsafe fn edit_leaf<const ISA: u8>(
            node: &mut Self::Leaf,
            len: usize,
            range: Range<usize>,
            action: Self::Tag,
            carry: Self::Tag,
        );
        /// # Safety
        /// ISA, len, and lane must satisfy the block contract.
        unsafe fn set_leaf<const ISA: u8>(
            node: &mut Self::Leaf,
            len: usize,
            lane: usize,
            value: Self::Value,
            carry: Self::Tag,
        );
        /// # Safety
        /// ISA, len, full, and edges must satisfy the block contract.
        unsafe fn edit_branch<const ISA: u8>(
            node: &mut Self::Branch,
            len: usize,
            full: Range<usize>,
            edges: &[Option<usize>; 2],
            action: Self::Tag,
            carry: Self::Tag,
        );
    }

    pub struct DaryDualSegmentTree<K>
    where
        K: DaryDualKernel,
    {
        leaves: Vec<K::Leaf>,
        branches: Vec<Vec<K::Branch>>,
        len: usize,
        backend: SimdBackend,
    }
    impl<K> Clone for DaryDualSegmentTree<K>
    where
        K: DaryDualKernel<Leaf: Clone, Branch: Clone>,
    {
        fn clone(&self) -> Self {
            Self {
                leaves: self.leaves.clone(),
                branches: self.branches.clone(),
                len: self.len,
                backend: self.backend,
            }
        }
    }
    impl<K> Debug for DaryDualSegmentTree<K>
    where
        K: DaryDualKernel<Leaf: Debug, Branch: Debug>,
    {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            f.debug_struct("DaryDualSegmentTree")
                .field("leaves", &self.leaves)
                .field("branches", &self.branches)
                .field("len", &self.len)
                .field("backend", &self.backend)
                .finish()
        }
    }
    impl<K> DaryDualSegmentTree<K>
    where
        K: DaryDualKernel,
    {
        pub fn from_keys(keys: impl ExactSizeIterator<Item = K::Key>) -> Self {
            assert!(K::B >= 2 && K::B.is_power_of_two());
            let len = keys.len();
            let mut keys = keys.map(K::encode).fuse();
            let mut consumed = 0;
            let leaves: Vec<_> = (0..len.div_ceil(K::B))
                .map(|_| K::leaf(keys.by_ref().take(K::B).inspect(|_| consumed += 1)))
                .collect();
            // ExactSizeIterator is safe to implement; its length cannot justify unsafe reads.
            assert!(
                consumed == len && keys.next().is_none(),
                "iterator length mismatch"
            );
            let mut branches = Vec::new();
            let mut count = leaves.len();
            while count > 1 {
                count = count.div_ceil(K::B);
                branches.push((0..count).map(|_| K::branch()).collect());
            }
            Self {
                leaves,
                branches,
                len,
                backend: simd_backend(),
            }
        }

        /// Hints the cache for a future access; index need not be in bounds.
        pub fn prefetch(&self, index: usize) {
            #[cfg(target_arch = "x86_64")]
            {
                let block = index / K::B;
                if let Some(leaf) = self.leaves.get(block) {
                    prefetch_block(leaf);
                }
                if let Some(level) = self.branches.first()
                    && let Some(branch) = level.get(block / K::B)
                {
                    prefetch_block(branch);
                }
            }
            let _ = index;
        }

        pub fn len(&self) -> usize {
            self.len
        }
        pub fn is_empty(&self) -> bool {
            self.len == 0
        }
        pub fn get(&self, mut index: usize) -> K::Output {
            assert!(index < self.len);
            let mut value = K::get(&self.leaves[index / K::B], index % K::B);
            index /= K::B;
            for level in &self.branches {
                value = K::apply(value, K::tag(&level[index / K::B], index % K::B));
                index /= K::B;
            }
            K::decode(value)
        }
        pub fn update<R>(&mut self, range: R, action: K::Action)
        where
            R: RangeBounds<usize>,
        {
            let range = range.to_range_bounded(0, self.len).expect("invalid range");
            if range.is_empty() {
                return;
            }
            let action = K::prepare(action);
            // SAFETY: range is valid and construction selects a supported ISA.
            unsafe {
                match self.backend {
                    SimdBackend::Scalar => self.update_node::<0>(
                        self.branches.len(),
                        0,
                        self.len,
                        range,
                        action,
                        K::unit(),
                    ),
                    SimdBackend::Avx2 => self.update_node::<1>(
                        self.branches.len(),
                        0,
                        self.len,
                        range,
                        action,
                        K::unit(),
                    ),
                    SimdBackend::Avx512 => self.update_node::<2>(
                        self.branches.len(),
                        0,
                        self.len,
                        range,
                        action,
                        K::unit(),
                    ),
                }
            }
        }
        pub fn set(&mut self, index: usize, key: K::Key) {
            assert!(index < self.len);
            let value = K::encode(key);
            // SAFETY: index is valid and construction selects a supported ISA.
            unsafe {
                match self.backend {
                    SimdBackend::Scalar => self.set_node::<0>(
                        self.branches.len(),
                        0,
                        self.len,
                        index,
                        value,
                        K::unit(),
                    ),
                    SimdBackend::Avx2 => self.set_node::<1>(
                        self.branches.len(),
                        0,
                        self.len,
                        index,
                        value,
                        K::unit(),
                    ),
                    SimdBackend::Avx512 => self.set_node::<2>(
                        self.branches.len(),
                        0,
                        self.len,
                        index,
                        value,
                        K::unit(),
                    ),
                }
            }
        }
        unsafe fn set_node<const ISA: u8>(
            &mut self,
            level: usize,
            index: usize,
            len: usize,
            position: usize,
            value: K::Value,
            carry: K::Tag,
        ) {
            if level == 0 {
                K::set_leaf::<ISA>(&mut self.leaves[index], len, position, value, carry);
                return;
            }
            let shift = level * K::B.trailing_zeros() as usize;
            let width = 1 << shift;
            let lane = position >> shift;
            let incoming = K::compose(K::tag(&self.branches[level - 1][index], lane), carry);
            self.set_node::<ISA>(
                level - 1,
                index * K::B + lane,
                width.min(len - lane * width),
                position - lane * width,
                value,
                incoming,
            );
            K::edit_branch::<ISA>(
                &mut self.branches[level - 1][index],
                len.div_ceil(width),
                0..0,
                &[Some(lane), None],
                K::unit(),
                carry,
            );
        }
        unsafe fn update_node<const ISA: u8>(
            &mut self,
            level: usize,
            index: usize,
            len: usize,
            range: Range<usize>,
            action: K::Tag,
            carry: K::Tag,
        ) {
            if level == 0 {
                K::edit_leaf::<ISA>(&mut self.leaves[index], len, range, action, carry);
                return;
            }
            let shift = level * K::B.trailing_zeros() as usize;
            let width = 1 << shift;
            let first = range.start >> shift;
            let last = (range.end - 1) >> shift;
            let mut full = first..last + 1;
            let mut edges = [None; 2];
            let first_offset = first * width;
            let first_len = width.min(len - first_offset);
            if range.start != first_offset || range.end - first_offset < first_len {
                let incoming = K::compose(K::tag(&self.branches[level - 1][index], first), carry);
                self.update_node::<ISA>(
                    level - 1,
                    index * K::B + first,
                    first_len,
                    range.start - first_offset..first_len.min(range.end - first_offset),
                    action,
                    incoming,
                );
                full.start += 1;
                edges[0] = Some(first);
            }
            let last_offset = last * width;
            let last_len = width.min(len - last_offset);
            if last != first && range.end - last_offset < last_len {
                let incoming = K::compose(K::tag(&self.branches[level - 1][index], last), carry);
                self.update_node::<ISA>(
                    level - 1,
                    index * K::B + last,
                    last_len,
                    0..range.end - last_offset,
                    action,
                    incoming,
                );
                full.end -= 1;
                edges[1] = Some(last);
            }
            K::edit_branch::<ISA>(
                &mut self.branches[level - 1][index],
                len.div_ceil(width),
                full,
                &edges,
                action,
                carry,
            );
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::{
            data_structure::DaryAffinePointKernel, num::mint_basic::MInt998244353 as M,
            tools::Xorshift,
        };
        #[test]
        fn test_affine_point() {
            let mut rng = Xorshift::default();
            let backends = vec![SimdBackend::Scalar];
            #[cfg(target_arch = "x86_64")]
            let backends = {
                let mut backends = backends;
                if is_x86_feature_detected!("avx2") {
                    backends.push(SimdBackend::Avx2);
                }
                if crate::tools::avx512_supported() {
                    backends.push(SimdBackend::Avx512);
                }
                backends
            };
            for len in [0usize, 1, 15, 16, 17, 255, 256, 257, 4095, 4096, 4097] {
                for &backend in &backends {
                    let mut expected: Vec<_> = (0..len).map(|_| M::from(rng.rand64())).collect();
                    let mut tree = DaryDualSegmentTree::<DaryAffinePointKernel>::from_keys(
                        expected.iter().copied(),
                    );
                    tree.backend = backend;
                    for turn in 0..1000 {
                        let (left, right) = match turn % 8 {
                            0 => (0, len),
                            1 => (len, len),
                            2 => (len / 2, len / 2),
                            3 => (0, len.min(16)),
                            4 => (len.saturating_sub(17), len),
                            _ => {
                                let l = rng.rand(len as u64 + 1) as usize;
                                (l, l + rng.rand((len - l + 1) as u64) as usize)
                            }
                        };
                        let action = match turn % 6 {
                            0 => (M::from(0), M::from(0)),
                            1 => (M::from(1), M::from(0)),
                            2 => (M::from(0), M::from(rng.rand64())),
                            _ => (M::from(rng.rand64()), M::from(rng.rand64())),
                        };
                        tree.update(left..right, action);
                        for v in &mut expected[left..right] {
                            *v = *v * action.0 + action.1;
                        }
                        if len != 0 {
                            let index = match turn % 8 {
                                0 => 0,
                                1 => len - 1,
                                2 => len / 2,
                                _ => rng.rand(len as u64) as usize,
                            };
                            assert_eq!(tree.get(index), expected[index]);
                            if turn % 7 == 0 {
                                let v = M::from(rng.rand64());
                                tree.set(index, v);
                                expected[index] = v;
                            }
                        }
                    }
                    for (i, &v) in expected.iter().enumerate() {
                        assert_eq!(tree.get(i), v);
                    }
                }
            }
        }
        #[test]
        fn test_large_branching() {
            struct SparseBool;
            impl DaryDualKernel for SparseBool {
                const B: usize = 1 << (usize::BITS - 1);
                type Key = ();
                type Value = bool;
                type Output = bool;
                type Action = bool;
                type Tag = bool;
                type Leaf = Vec<(Range<usize>, bool)>;
                type Branch = std::collections::BTreeSet<usize>;
                fn encode(_: ()) -> bool {
                    false
                }
                fn decode(x: bool) -> bool {
                    x
                }
                fn prepare(x: bool) -> bool {
                    x
                }
                fn unit() -> bool {
                    false
                }
                fn compose(old: bool, later: bool) -> bool {
                    old | later
                }
                fn apply(value: bool, tag: bool) -> bool {
                    value | tag
                }
                fn leaf(values: impl Iterator<Item = bool>) -> Self::Leaf {
                    values
                        .enumerate()
                        .filter(|(_, v)| *v)
                        .map(|(i, _)| (i..i + 1, true))
                        .collect()
                }
                fn branch() -> Self::Branch {
                    std::collections::BTreeSet::new()
                }
                fn get(node: &Self::Leaf, lane: usize) -> bool {
                    node.iter()
                        .rev()
                        .find(|(r, _)| r.contains(&lane))
                        .is_some_and(|(_, x)| *x)
                }
                fn tag(node: &Self::Branch, lane: usize) -> bool {
                    node.contains(&lane)
                }
                unsafe fn edit_leaf<const ISA: u8>(
                    node: &mut Self::Leaf,
                    len: usize,
                    r: Range<usize>,
                    action: bool,
                    carry: bool,
                ) {
                    assert!(r.start <= r.end && r.end <= len && len <= Self::B);
                    if carry {
                        node.push((0..len, true));
                    }
                    if action {
                        node.push((r, true));
                    }
                }
                unsafe fn set_leaf<const ISA: u8>(
                    node: &mut Self::Leaf,
                    len: usize,
                    i: usize,
                    value: bool,
                    carry: bool,
                ) {
                    assert!(i < len && len <= Self::B);
                    if carry {
                        node.push((0..len, true));
                    }
                    node.push((i..i + 1, value));
                }
                unsafe fn edit_branch<const ISA: u8>(
                    node: &mut Self::Branch,
                    len: usize,
                    r: Range<usize>,
                    edges: &[Option<usize>; 2],
                    action: bool,
                    carry: bool,
                ) {
                    assert!(r.start <= r.end && r.end <= len && len <= Self::B);
                    if carry {
                        node.extend(0..len);
                    }
                    if action {
                        node.extend(r);
                    }
                    for i in edges.iter().flatten() {
                        node.remove(i);
                    }
                }
            }

            let n = SparseBool::B + 2;
            // The uniform initial state from from_keys(repeat_n((), n)), without
            // iterating through 2^(usize::BITS - 1) keys in debug builds.
            let mut tree = DaryDualSegmentTree::<SparseBool> {
                leaves: vec![Vec::new(), Vec::new()],
                branches: vec![vec![std::collections::BTreeSet::new()]],
                len: n,
                backend: SimdBackend::Scalar,
            };
            tree.update(SparseBool::B..SparseBool::B + 1, true);
            assert!(!tree.get(SparseBool::B - 1));
            assert!(tree.get(SparseBool::B));
            assert!(!tree.get(SparseBool::B + 1));
            tree.update(..SparseBool::B + 1, true);
            assert!(tree.get(0));
            assert!(tree.get(SparseBool::B - 1));
            assert!(!tree.get(SparseBool::B + 1));
            tree.set(SparseBool::B, ());
            assert!(!tree.get(SparseBool::B));
            assert!(tree.get(SparseBool::B - 1));
        }
    }
}

#[codesnip::entry(
    "_SimdDualAdapter",
    when("SimdMonoidAct", "DaryDualSegmentTree"),
    include("_SimdMonoidActBlock")
)]
mod dual_monoid_act {
    use super::DaryDualKernel;
    use super::{
        SimdMonoidAct,
        simd_act_block::{apply_selected, edit_keys, leaf},
        simd_monoid::LANES,
    };
    use std::ops::Range;

    impl<A> DaryDualKernel for A
    where
        A: SimdMonoidAct,
    {
        const B: usize = LANES;
        type Key = A::Key;
        type Value = A::Key;
        type Output = A::Key;
        type Action = A::Act;
        type Tag = A::Act;
        type Leaf = [A::Key; LANES];
        type Branch = [A::Act; LANES];

        fn encode(key: A::Key) -> A::Key {
            key
        }
        fn decode(value: A::Key) -> A::Key {
            value
        }
        fn prepare(action: A::Act) -> A::Act {
            action
        }
        fn unit() -> A::Act {
            A::unit()
        }
        fn compose(old: A::Act, later: A::Act) -> A::Act {
            A::operate(&old, &later)
        }
        fn apply(value: A::Key, tag: A::Act) -> A::Key {
            A::act(&value, &tag)
        }
        fn leaf(values: impl Iterator<Item = A::Key>) -> Self::Leaf {
            leaf(values)
        }
        fn branch() -> Self::Branch {
            [A::unit(); LANES]
        }
        fn get(node: &Self::Leaf, lane: usize) -> A::Key {
            node[lane]
        }
        fn tag(node: &Self::Branch, lane: usize) -> A::Act {
            node[lane]
        }
        unsafe fn edit_leaf<const ISA: u8>(
            node: &mut Self::Leaf,
            len: usize,
            range: Range<usize>,
            action: A::Act,
            carry: A::Act,
        ) {
            unsafe { edit_keys::<A, ISA>(&mut node[..len], range, action, carry) };
        }
        unsafe fn set_leaf<const ISA: u8>(
            node: &mut Self::Leaf,
            len: usize,
            lane: usize,
            value: A::Key,
            carry: A::Act,
        ) {
            unsafe { A::act_lanes::<ISA>(&mut node[..len], carry) };
            node[lane] = value;
        }
        unsafe fn edit_branch<const ISA: u8>(
            node: &mut Self::Branch,
            len: usize,
            full: Range<usize>,
            edges: &[Option<usize>; 2],
            action: A::Act,
            carry: A::Act,
        ) {
            unsafe {
                if full.is_empty() {
                    A::compose_lanes::<ISA>(&mut node[..len], carry);
                } else {
                    let combined = A::operate(&carry, &action);
                    apply_selected(&mut node[..len], full, carry, combined, |tags, tag| {
                        A::compose_lanes::<ISA>(tags, tag);
                    });
                }
            }
            for &lane in edges.iter().flatten() {
                node[lane] = A::unit();
            }
        }
    }
}

#[cfg(target_arch = "x86_64")]
use super::prefetch_block;
use super::{RangeBoundsExt, SimdBackend, simd_backend};
use super::{SimdAggregateAct, SimdMonoid, simd_monoid, simd_monoid_act::simd_act_block};

#[codesnip::entry("DaryLazySegmentTree")]
pub use self::dary_lazy_tree::{DaryLazyKernel, DaryLazyPointKernel, DaryLazySegmentTree};

#[codesnip::entry("DaryLazySegmentTree", include("discrete_steps", "avx_helper"))]
mod dary_lazy_tree {
    #![allow(unsafe_op_in_unsafe_fn)] // Backend selection guards the kernel calls.
    #[cfg(target_arch = "x86_64")]
    use super::prefetch_block;
    use super::{RangeBoundsExt, SimdBackend, simd_backend};
    use std::{
        fmt::{self, Debug, Formatter},
        ops::{Range, RangeBounds},
    };

    pub struct DaryLazySegmentTree<K>
    where
        K: DaryLazyKernel,
    {
        leaves: Vec<K::Leaf>,
        branches: Vec<Vec<K::Branch>>,
        total: K::Summary,
        len: usize,
        backend: SimdBackend,
        state: K::State,
    }

    impl<K> Clone for DaryLazySegmentTree<K>
    where
        K: DaryLazyKernel,
        K::Leaf: Clone,
        K::Branch: Clone,
        K::State: Clone,
    {
        fn clone(&self) -> Self {
            Self {
                leaves: self.leaves.clone(),
                branches: self.branches.clone(),
                total: self.total,
                len: self.len,
                backend: self.backend,
                state: self.state.clone(),
            }
        }
    }
    impl<K> Debug for DaryLazySegmentTree<K>
    where
        K: DaryLazyKernel,
        K::Leaf: Debug,
        K::Branch: Debug,
        K::Summary: Debug,
        K::State: Debug,
    {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            f.debug_struct("DaryLazySegmentTree")
                .field("leaves", &self.leaves)
                .field("branches", &self.branches)
                .field("total", &self.total)
                .field("len", &self.len)
                .field("backend", &self.backend)
                .field("state", &self.state)
                .finish()
        }
    }

    /// Block operations for an ordered d-ary lazy segment tree.
    ///
    /// `combine(left, right)` preserves key order; `compose(old, later)` preserves
    /// action order. `Change` repairs a summary after its incoming carry has already
    /// been applied. A boundary repair consumes the child's stored tag even when the
    /// summary does not change. `None` from `constant` means the old keys are needed.
    ///
    /// B is a power of two greater than one. Leaves retain all supplied keys;
    /// blocks contain at most B valid lanes.
    /// `len` is the number of keys in the
    /// subtree, `width` the power-of-two nominal span of a child; the last child may be shorter.
    /// Updating a block materializes its carry on every valid lane, updates `full`,
    /// then repairs `edges`. Child updates happen before their parent block is edited.
    /// State belongs to one tree; Prepared must not borrow it or leave dangling tags.
    /// prepare can inspect branch storage before the update to reclaim records that
    /// no stored tag uses. finish runs after all edits and consumes Prepared.
    ///
    /// # Safety
    /// Unsafe methods receive a nonempty valid subtree and in-bounds lane ranges.
    /// Empty fold ranges must return unit.
    /// ISA is 0 (portable), 1 (AVX2), or 2 (AVX-512F/DQ/CD/BW/VL), selected on a
    /// supporting CPU. Additional instruction sets require the kernel's own check.
    /// Implementations must support the portable path on non-x86 targets.
    pub trait DaryLazyKernel {
        const B: usize;
        /// Bottom-up folds avoid repeated descent; constant-tag shortcuts benefit from top-down folds.
        const FOLD_UPWARD: bool = true;
        /// Descends both boundary paths together before repairing their parents.
        const PAIRED_UPDATES: bool = false;
        type Key;
        type EncodedKey;
        type Output;
        type Action;
        type Summary: Copy;
        type Tag: Copy;
        type Prepared;
        type Change: Copy;
        type Leaf;
        type Branch;
        type State;

        fn new_state(len: usize) -> Self::State;
        fn encode(key: Self::Key) -> Self::EncodedKey;
        fn decode(value: Self::Summary) -> Self::Output;
        fn unit() -> Self::Summary;
        fn tag_unit() -> Self::Tag;
        fn combine(left: Self::Summary, right: Self::Summary) -> Self::Summary;
        fn compose(old: Self::Tag, later: Self::Tag) -> Self::Tag;
        fn prepare(
            state: &mut Self::State,
            action: Self::Action,
            range: Range<usize>,
            len: usize,
            branches: &[Vec<Self::Branch>],
        ) -> Self::Prepared;
        fn finish(_state: &mut Self::State, _action: Self::Prepared) {}
        fn map(
            state: &Self::State,
            value: Self::Summary,
            tag: Self::Tag,
            len: usize,
        ) -> Self::Summary;
        fn repair(value: Self::Summary, change: Self::Change) -> Self::Summary;
        fn constant(_state: &Self::State, _tag: Self::Tag, _len: usize) -> Option<Self::Summary> {
            None
        }
        fn leaf(keys: impl Iterator<Item = Self::EncodedKey>) -> Self::Leaf;
        fn branch(values: &[Self::Summary]) -> Self::Branch;
        /// # Safety
        /// The ISA and range must satisfy the trait's block contract.
        unsafe fn fold_leaf<const ISA: u8>(
            state: &Self::State,
            node: &Self::Leaf,
            range: Range<usize>,
        ) -> Self::Summary;
        /// # Safety
        /// The ISA and range must satisfy the trait's block contract.
        unsafe fn fold_branch<const ISA: u8>(
            state: &Self::State,
            node: &Self::Branch,
            range: Range<usize>,
        ) -> Self::Summary;
        fn tag(node: &Self::Branch, lane: usize) -> Self::Tag;
        /// # Safety
        /// The ISA, valid length, and range must satisfy the trait's block contract.
        unsafe fn edit_leaf<const ISA: u8>(
            state: &mut Self::State,
            node: &mut Self::Leaf,
            len: usize,
            range: Range<usize>,
            action: &Self::Prepared,
            carry: Self::Tag,
        ) -> Self::Change;
        /// # Safety
        /// The ISA, spans, full lanes, and completed edges must satisfy the trait's block contract.
        #[allow(clippy::too_many_arguments)] // Geometry, boundary repairs, and actions form one fused edit.
        unsafe fn edit_branch<const ISA: u8>(
            state: &mut Self::State,
            node: &mut Self::Branch,
            len: usize,
            width: usize,
            full: Range<usize>,
            edges: &[Option<(usize, Self::Change)>; 2],
            action: &Self::Prepared,
            carry: Self::Tag,
        ) -> Self::Change;
    }

    /// Point access and replacement for a lazy kernel that retains individual keys.
    pub trait DaryLazyPointKernel: DaryLazyKernel {
        /// Returns the stored key before applying ancestor tags.
        fn key(node: &Self::Leaf, lane: usize) -> Self::Key;
        fn apply_key(key: Self::Key, tag: Self::Tag) -> Self::Key;
        /// Prepares a replacement for a singleton edit that descends to its leaf.
        fn prepare_set(key: Self::Key) -> Self::Prepared;
    }

    impl<K> DaryLazySegmentTree<K>
    where
        K: DaryLazyKernel,
    {
        pub fn from_keys(keys: impl ExactSizeIterator<Item = K::Key>) -> Self {
            assert!(K::B >= 2 && K::B.is_power_of_two());
            let state = K::new_state(keys.len());
            let backend = simd_backend();
            // SAFETY: the selected backend is supported on this CPU.
            unsafe {
                match backend {
                    SimdBackend::Scalar => Self::build::<0>(keys, state, backend),
                    SimdBackend::Avx2 => Self::build::<1>(keys, state, backend),
                    SimdBackend::Avx512 => Self::build::<2>(keys, state, backend),
                }
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

        pub fn update<R>(&mut self, range: R, action: K::Action)
        where
            R: RangeBounds<usize>,
        {
            let range = range.to_range_bounded(0, self.len).expect("invalid range");
            if range.is_empty() {
                return;
            }
            let action = K::prepare(
                &mut self.state,
                action,
                range.clone(),
                self.len,
                &self.branches,
            );
            self.edit::<false>(range, &action);
            K::finish(&mut self.state, action);
        }

        pub fn fold<R>(&self, range: R) -> K::Output
        where
            R: RangeBounds<usize>,
        {
            let range = range.to_range_bounded(0, self.len).expect("invalid range");
            K::decode(self.fold_range(range))
        }

        pub fn fold_all(&self) -> K::Output {
            K::decode(self.total)
        }

        pub fn get(&self, mut index: usize) -> K::Key
        where
            K: DaryLazyPointKernel,
        {
            assert!(index < self.len);
            let mut key = K::key(&self.leaves[index / K::B], index % K::B);
            index /= K::B;
            for level in &self.branches {
                let tag = K::tag(&level[index / K::B], index % K::B);
                key = K::apply_key(key, tag);
                index /= K::B;
            }
            key
        }

        pub fn set(&mut self, index: usize, key: K::Key)
        where
            K: DaryLazyPointKernel,
        {
            assert!(index < self.len);
            let action = K::prepare_set(key);
            self.edit::<true>(index..index + 1, &action);
            K::finish(&mut self.state, action);
        }

        fn edit<const TO_LEAF: bool>(&mut self, range: Range<usize>, action: &K::Prepared) {
            // SAFETY: construction selects a supported backend for every descendant.
            let change = unsafe {
                match self.backend {
                    SimdBackend::Scalar => self.update_node::<0, TO_LEAF>(
                        self.branches.len(),
                        0,
                        self.len,
                        range,
                        action,
                        K::tag_unit(),
                    ),
                    SimdBackend::Avx2 => self.update_node::<1, TO_LEAF>(
                        self.branches.len(),
                        0,
                        self.len,
                        range,
                        action,
                        K::tag_unit(),
                    ),
                    SimdBackend::Avx512 => self.update_node::<2, TO_LEAF>(
                        self.branches.len(),
                        0,
                        self.len,
                        range,
                        action,
                        K::tag_unit(),
                    ),
                }
            };
            self.total = K::repair(self.total, change);
        }

        unsafe fn build<const ISA: u8>(
            keys: impl ExactSizeIterator<Item = K::Key>,
            state: K::State,
            backend: SimdBackend,
        ) -> Self {
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
            let mut values: Vec<_> = leaves
                .iter()
                .enumerate()
                .map(|(i, node)| unsafe {
                    K::fold_leaf::<ISA>(&state, node, 0..K::B.min(len - i * K::B))
                })
                .collect();
            let mut branches = Vec::new();
            while values.len() > 1 {
                let nodes: Vec<_> = values.chunks(K::B).map(K::branch).collect();
                values = nodes
                    .iter()
                    .enumerate()
                    .map(|(i, node)| unsafe {
                        K::fold_branch::<ISA>(&state, node, 0..K::B.min(values.len() - i * K::B))
                    })
                    .collect();
                branches.push(nodes);
            }
            Self {
                leaves,
                branches,
                total: values.first().copied().unwrap_or_else(K::unit),
                len,
                backend,
                state,
            }
        }

        fn fold_range(&self, range: Range<usize>) -> K::Summary {
            if range.is_empty() {
                K::unit()
            } else if range.start == 0 && range.end == self.len {
                self.total
            } else {
                // SAFETY: construction selects a supported backend.
                unsafe {
                    match self.backend {
                        SimdBackend::Scalar => self.fold_with::<0>(range),
                        SimdBackend::Avx2 => self.fold_with::<1>(range),
                        SimdBackend::Avx512 => self.fold_with::<2>(range),
                    }
                }
            }
        }

        unsafe fn update_node<const ISA: u8, const TO_LEAF: bool>(
            &mut self,
            level: usize,
            index: usize,
            len: usize,
            range: Range<usize>,
            action: &K::Prepared,
            carry: K::Tag,
        ) -> K::Change {
            if level == 0 {
                return K::edit_leaf::<ISA>(
                    &mut self.state,
                    &mut self.leaves[index],
                    len,
                    range,
                    action,
                    carry,
                );
            }
            let shift = level * K::B.trailing_zeros() as usize;
            let width = 1 << shift;
            let first = range.start >> shift;
            let last = (range.end - 1) >> shift;
            let first_offset = first * width;
            let first_len = width.min(len - first_offset);
            let last_offset = last * width;
            let last_len = width.min(len - last_offset);
            let partial_left =
                TO_LEAF || range.start != first_offset || range.end - first_offset < first_len;
            let partial_right = last != first && range.end - last_offset < last_len;
            let full = first + partial_left as usize..last + 1 - partial_right as usize;
            let mut edges = [None; 2];
            if K::PAIRED_UPDATES && partial_left && partial_right {
                let left_carry = K::compose(K::tag(&self.branches[level - 1][index], first), carry);
                let right_carry = K::compose(K::tag(&self.branches[level - 1][index], last), carry);
                let (left, right) = self.update_pair::<ISA>(
                    level - 1,
                    (
                        index * K::B + first,
                        first_len,
                        range.start - first_offset,
                        left_carry,
                    ),
                    (
                        index * K::B + last,
                        last_len,
                        range.end - last_offset,
                        right_carry,
                    ),
                    action,
                );
                edges = [Some((first, left)), Some((last, right))];
            } else {
                if partial_left {
                    let incoming =
                        K::compose(K::tag(&self.branches[level - 1][index], first), carry);
                    let change = self.update_node::<ISA, TO_LEAF>(
                        level - 1,
                        index * K::B + first,
                        first_len,
                        range.start - first_offset..first_len.min(range.end - first_offset),
                        action,
                        incoming,
                    );
                    edges[0] = Some((first, change));
                }
                if partial_right {
                    let incoming =
                        K::compose(K::tag(&self.branches[level - 1][index], last), carry);
                    let change = self.update_node::<ISA, TO_LEAF>(
                        level - 1,
                        index * K::B + last,
                        last_len,
                        0..range.end - last_offset,
                        action,
                        incoming,
                    );
                    edges[1] = Some((last, change));
                }
            }
            K::edit_branch::<ISA>(
                &mut self.state,
                &mut self.branches[level - 1][index],
                len,
                width,
                full,
                &edges,
                action,
                carry,
            )
        }

        // The left range is a suffix and the right range is a prefix of distinct subtrees.
        unsafe fn update_pair<const ISA: u8>(
            &mut self,
            level: usize,
            (left, left_len, start, left_carry): (usize, usize, usize, K::Tag),
            (right, right_len, end, right_carry): (usize, usize, usize, K::Tag),
            action: &K::Prepared,
        ) -> (K::Change, K::Change) {
            if level == 0 {
                let a = K::edit_leaf::<ISA>(
                    &mut self.state,
                    &mut self.leaves[left],
                    left_len,
                    start..left_len,
                    action,
                    left_carry,
                );
                let b = K::edit_leaf::<ISA>(
                    &mut self.state,
                    &mut self.leaves[right],
                    right_len,
                    0..end,
                    action,
                    right_carry,
                );
                return (a, b);
            }
            let shift = level * K::B.trailing_zeros() as usize;
            let width = 1usize << shift;
            let l = start >> shift;
            let r = (end - 1) >> shift;
            let left_offset = l * width;
            let right_offset = r * width;
            let a_len = width.min(left_len - left_offset);
            let b_len = width.min(right_len - right_offset);
            let partial_left = start != left_offset;
            let partial_right = end - right_offset < b_len;
            let mut left_edges = [None; 2];
            let mut right_edges = [None; 2];
            if partial_left && partial_right {
                let lc = K::compose(K::tag(&self.branches[level - 1][left], l), left_carry);
                let rc = K::compose(K::tag(&self.branches[level - 1][right], r), right_carry);
                let (a, b) = self.update_pair::<ISA>(
                    level - 1,
                    (left * K::B + l, a_len, start - left_offset, lc),
                    (right * K::B + r, b_len, end - right_offset, rc),
                    action,
                );
                left_edges[0] = Some((l, a));
                right_edges[1] = Some((r, b));
            } else {
                if partial_left {
                    let lc = K::compose(K::tag(&self.branches[level - 1][left], l), left_carry);
                    let a = self.update_node::<ISA, false>(
                        level - 1,
                        left * K::B + l,
                        a_len,
                        start - left_offset..a_len,
                        action,
                        lc,
                    );
                    left_edges[0] = Some((l, a));
                }
                if partial_right {
                    let rc = K::compose(K::tag(&self.branches[level - 1][right], r), right_carry);
                    let b = self.update_node::<ISA, false>(
                        level - 1,
                        right * K::B + r,
                        b_len,
                        0..end - right_offset,
                        action,
                        rc,
                    );
                    right_edges[1] = Some((r, b));
                }
            }
            let a = K::edit_branch::<ISA>(
                &mut self.state,
                &mut self.branches[level - 1][left],
                left_len,
                width,
                l + partial_left as usize..((left_len - 1) >> shift) + 1,
                &left_edges,
                action,
                left_carry,
            );
            let b = K::edit_branch::<ISA>(
                &mut self.state,
                &mut self.branches[level - 1][right],
                right_len,
                width,
                0..r + 1 - partial_right as usize,
                &right_edges,
                action,
                right_carry,
            );
            (a, b)
        }

        unsafe fn fold_with<const ISA: u8>(&self, range: Range<usize>) -> K::Summary {
            if K::FOLD_UPWARD {
                self.fold_upward::<ISA>(range)
            } else {
                self.fold_node::<ISA>(self.branches.len(), 0, self.len, range, K::tag_unit())
            }
        }

        unsafe fn fold_upward<const ISA: u8>(&self, range: Range<usize>) -> K::Summary {
            let mut left = range.start / K::B;
            let mut right = (range.end - 1) / K::B;
            let l = range.start % K::B;
            let r = (range.end - 1) % K::B + 1;
            if left == right {
                let value = K::fold_leaf::<ISA>(&self.state, &self.leaves[left], l..r);
                return self.lift(0, left, value, range.len());
            }
            let mut suffix = K::fold_leaf::<ISA>(&self.state, &self.leaves[left], l..K::B);
            let mut prefix = K::fold_leaf::<ISA>(&self.state, &self.leaves[right], 0..r);
            let mut suffix_len = K::B - l;
            let mut prefix_len = r;
            let mut width = K::B;
            for (at, level) in self.branches.iter().enumerate() {
                let l = left % K::B;
                let r = right % K::B;
                left /= K::B;
                right /= K::B;
                suffix = K::map(&self.state, suffix, K::tag(&level[left], l), suffix_len);
                prefix = K::map(&self.state, prefix, K::tag(&level[right], r), prefix_len);
                if left == right {
                    let middle = K::fold_branch::<ISA>(&self.state, &level[left], l + 1..r);
                    let value = K::combine(K::combine(suffix, middle), prefix);
                    return self.lift(at + 1, left, value, range.len());
                }
                suffix = K::combine(
                    suffix,
                    K::fold_branch::<ISA>(&self.state, &level[left], l + 1..K::B),
                );
                prefix = K::combine(
                    K::fold_branch::<ISA>(&self.state, &level[right], 0..r),
                    prefix,
                );
                suffix_len += (K::B - l - 1) * width;
                prefix_len += r * width;
                width *= K::B;
            }
            unreachable!()
        }
        fn lift(
            &self,
            start: usize,
            mut index: usize,
            mut value: K::Summary,
            len: usize,
        ) -> K::Summary {
            for level in &self.branches[start..] {
                value = K::map(
                    &self.state,
                    value,
                    K::tag(&level[index / K::B], index % K::B),
                    len,
                );
                index /= K::B;
            }
            value
        }

        unsafe fn fold_node<const ISA: u8>(
            &self,
            level: usize,
            index: usize,
            len: usize,
            range: Range<usize>,
            carry: K::Tag,
        ) -> K::Summary {
            if let Some(value) = K::constant(&self.state, carry, range.len()) {
                return value;
            }
            if level == 0 {
                let count = range.len();
                return K::map(
                    &self.state,
                    K::fold_leaf::<ISA>(&self.state, &self.leaves[index], range),
                    carry,
                    count,
                );
            }
            let shift = level * K::B.trailing_zeros() as usize;
            let width = 1 << shift;
            let first = range.start >> shift;
            let last = (range.end - 1) >> shift;
            let node = &self.branches[level - 1][index];
            let first_offset = first * width;
            let first_len = width.min(len - first_offset);
            if first == last {
                if range.start == first_offset && range.end == first_offset + first_len {
                    return K::map(
                        &self.state,
                        K::fold_branch::<ISA>(&self.state, node, first..first + 1),
                        carry,
                        first_len,
                    );
                }
                return self.fold_node::<ISA>(
                    level - 1,
                    index * K::B + first,
                    first_len,
                    range.start - first_offset..range.end - first_offset,
                    K::compose(K::tag(node, first), carry),
                );
            }
            let mut result = K::unit();
            let mut full = first..last + 1;
            if range.start != first_offset {
                result = self.fold_node::<ISA>(
                    level - 1,
                    index * K::B + first,
                    first_len,
                    range.start - first_offset..first_len,
                    K::compose(K::tag(node, first), carry),
                );
                full.start += 1;
            }
            let last_offset = last * width;
            let last_end = last_offset + width.min(len - last_offset);
            let partial_right = range.end != last_end;
            if partial_right {
                full.end -= 1;
            }
            if !full.is_empty() {
                let full_end = if partial_right { last_offset } else { last_end };
                let count = full_end - full.start * width;
                result = K::combine(
                    result,
                    K::map(
                        &self.state,
                        K::fold_branch::<ISA>(&self.state, node, full),
                        carry,
                        count,
                    ),
                );
            }
            if partial_right {
                let right = self.fold_node::<ISA>(
                    level - 1,
                    index * K::B + last,
                    last_end - last_offset,
                    0..range.end - last_offset,
                    K::compose(K::tag(node, last), carry),
                );
                result = K::combine(result, right);
            }
            result
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::{
            data_structure::{DaryAffineSumKernel, DaryAssignCompositeKernel},
            num::{
                mint_basic::MInt998244353 as M,
                montgomery::{Modulo998244353 as Mod, MontgomeryReduction32},
            },
        };
        const B: usize = 16;
        use crate::tools::{Xorshift, testutil::sequence_cases};

        fn backends() -> Vec<SimdBackend> {
            let result = vec![SimdBackend::Scalar];
            #[cfg(target_arch = "x86_64")]
            {
                let mut result = result;
                if is_x86_feature_detected!("avx2") {
                    result.push(SimdBackend::Avx2);
                }
                if crate::tools::avx512_supported() {
                    result.push(SimdBackend::Avx512);
                }
                result
            }
            #[cfg(not(target_arch = "x86_64"))]
            result
        }

        #[test]
        fn test_affine_sum() {
            for backend in backends() {
                let mut rng = Xorshift::default();
                let cases: Vec<_> =
                    sequence_cases(&mut rng, 0u64..=Mod::MOD as u64 - 1, 0..=4097).collect();
                let p = Mod::MOD as u64;
                for mut values in cases {
                    let n = values.len();
                    let mut tree = DaryLazySegmentTree::<DaryAffineSumKernel>::from_keys(
                        values.iter().copied().map(M::from),
                    );
                    tree.backend = backend;
                    assert_eq!(
                        tree.fold_all().inner() as u64,
                        values.iter().sum::<u64>() % p
                    );
                    for (i, &value) in values.iter().enumerate() {
                        assert_eq!(tree.fold(i..i + 1).inner() as u64, value);
                    }
                    for range in [0..n / 2, n / 3..n] {
                        assert_eq!(
                            tree.fold(range.clone()).inner() as u64,
                            values[range].iter().sum::<u64>() % p
                        );
                    }
                    for step in 0..6 * 6 {
                        let range = match step % 6 {
                            0 => 0..n,
                            1 => 0..n.min(1),
                            2 => n.saturating_sub(1)..n,
                            3 => n / 2..n / 2,
                            _ => {
                                let l = rng.random(0..=n);
                                let r = rng.random(0..=n);
                                l.min(r)..l.max(r)
                            }
                        };
                        let (a, b) = match step / 6 % 6 {
                            0 => (0, rng.random(0..p)),
                            1 => (1, 0),
                            2 => (1, rng.random(0..p)),
                            3 => (p - 1, p - 1),
                            _ => (rng.random(0..p), rng.random(0..p)),
                        };
                        tree.update(range.clone(), (M::from(a), M::from(b)));
                        for value in &mut values[range] {
                            *value = (a * *value + b) % p;
                        }
                        assert_eq!(tree.len(), n);
                        assert_eq!(tree.is_empty(), n == 0);
                        assert_eq!(
                            tree.fold_all().inner() as u64,
                            values.iter().sum::<u64>() % p
                        );
                        for _ in 0..8 {
                            let l = rng.random(0..=n);
                            let r = rng.random(0..=n);
                            let range = l.min(r)..l.max(r);
                            assert_eq!(
                                tree.fold(range.clone()).inner() as u64,
                                values[range].iter().sum::<u64>() % p
                            );
                        }
                    }
                }
                let mut values = vec![M::from(0); B * B + 1];
                values[0] = M::from(1);
                values[1] = M::from(p - 1);
                let mut tree =
                    DaryLazySegmentTree::<DaryAffineSumKernel>::from_keys(values.into_iter());
                tree.backend = backend;
                tree.update(.., (M::from(2), M::from(0)));
                tree.update(..B, (M::from(3), M::from(0)));
                assert_eq!(tree.fold_all(), M::from(0));
                assert_eq!(tree.fold(..1), M::from(6));
            }
        }

        #[test]
        fn test_assign_composite() {
            for backend in backends() {
                let mut rng = Xorshift::default();
                let p = Mod::MOD as u64;
                let cases: Vec<_> = sequence_cases(&mut rng, 0u64..=p - 1, 0..=4097).collect();
                for values in cases {
                    let n = values.len();
                    let mut values: Vec<_> =
                        values.into_iter().map(|a| (a, rng.random(0..p))).collect();
                    let mut tree = DaryLazySegmentTree::<DaryAssignCompositeKernel>::from_keys(
                        values.iter().map(|&(a, b)| (M::from(a), M::from(b))),
                    );
                    tree.backend = backend;
                    for (i, &(a, b)) in values.iter().enumerate() {
                        assert_eq!(tree.fold(i..i + 1), (M::from(a), M::from(b)));
                    }
                    let (a, b) = tree.fold_all();
                    for x in [0, 1] {
                        let expected = values.iter().fold(x, |x, &(a, b)| (a * x + b) % p);
                        assert_eq!((a.inner() as u64 * x + b.inner() as u64) % p, expected);
                    }
                    for range in [0..n / 2, n / 3..n] {
                        let (a, b) = tree.fold(range.clone());
                        for x in [0, 1] {
                            let expected = values[range.clone()]
                                .iter()
                                .fold(x, |x, &(a, b)| (a * x + b) % p);
                            assert_eq!((a.inner() as u64 * x + b.inner() as u64) % p, expected);
                        }
                    }
                    for step in 0..6 * 6 {
                        let range = match step % 6 {
                            0 => 0..n,
                            1 => 0..n.min(1),
                            2 => n.saturating_sub(1)..n,
                            3 => n / 2..n / 2,
                            _ => {
                                let l = rng.random(0..=n);
                                let r = rng.random(0..=n);
                                l.min(r)..l.max(r)
                            }
                        };
                        let action = match step / 6 % 6 {
                            0 => (0, rng.random(0..p)),
                            1 => (1, 0),
                            2 => (1, rng.random(0..p)),
                            3 => (p - 1, p - 1),
                            _ => (rng.random(0..p), rng.random(0..p)),
                        };
                        tree.update(range.clone(), (M::from(action.0), M::from(action.1)));
                        values[range].fill(action);
                        assert_eq!(tree.len(), n);
                        assert_eq!(tree.is_empty(), n == 0);
                        for query in 0..8 {
                            let l = rng.random(0..=n);
                            let r = rng.random(0..=n);
                            let range = if query == 0 { 0..n } else { l.min(r)..l.max(r) };
                            let (a, b) = tree.fold(range.clone());
                            if query == 0 {
                                assert_eq!((a, b), tree.fold_all());
                            }
                            for x in [0, 1] {
                                let expected = values[range.clone()]
                                    .iter()
                                    .fold(x, |x, &(a, b)| (a * x + b) % p);
                                assert_eq!((a.inner() as u64 * x + b.inner() as u64) % p, expected);
                            }
                        }
                    }
                }
            }
            for backend in backends() {
                let mut rng = Xorshift::default();
                let p = Mod::MOD as u64;
                for n in [B + 1, B * B + 1, B * B * B + 1] {
                    let mut values = vec![(1u64, 0u64); n];
                    let mut tree = DaryLazySegmentTree::<DaryAssignCompositeKernel>::from_keys(
                        values.iter().map(|&(a, b)| (M::from(a), M::from(b))),
                    );
                    tree.backend = backend;
                    // Distinct tags at every level keep stale descendants alive during reclamation.
                    let mut width = B;
                    let mut serial = 1u64;
                    while width < n {
                        for left in (0..n).step_by(width) {
                            let range = left..n.min(left + width);
                            let action = (serial, serial + 1);
                            tree.update(range.clone(), (M::from(action.0), M::from(action.1)));
                            values[range].fill(action);
                            serial += 1;
                        }
                        width *= B;
                    }
                    let copy = tree.clone();
                    let copied = values.clone();
                    for step in 0..4 * n {
                        let left = rng.random(0..n.div_ceil(B)) * B;
                        let range = left..n.min(left + B);
                        let action = match step % 4 {
                            0 => (0, rng.random(0..p)),
                            1 => (1, rng.random(0..p)),
                            _ => (rng.random(0..p), rng.random(0..p)),
                        };
                        tree.update(range.clone(), (M::from(action.0), M::from(action.1)));
                        values[range.clone()].fill(action);
                        let range = if step.is_multiple_of(B) { 0..n } else { range };
                        let (a, b) = tree.fold(range.clone());
                        for x in [0, 1] {
                            let expected = values[range.clone()]
                                .iter()
                                .fold(x, |x, &(a, b)| (a * x + b) % p);
                            assert_eq!((a.inner() as u64 * x + b.inner() as u64) % p, expected);
                        }
                    }
                    for range in [0..n, 0..1, n / 2..n / 2 + 1, n - 1..n] {
                        let (a, b) = copy.fold(range.clone());
                        if range == (0..n) {
                            assert_eq!((a, b), copy.fold_all());
                        }
                        for x in [0, 1] {
                            let expected = copied[range.clone()]
                                .iter()
                                .fold(x, |x, &(a, b)| (a * x + b) % p);
                            assert_eq!((a.inner() as u64 * x + b.inner() as u64) % p, expected);
                        }
                    }
                }
            }
        }
        #[test]
        fn test_large_branching() {
            struct Count<const UP: bool>;
            impl<const UP: bool> DaryLazyKernel for Count<UP> {
                const B: usize = 1 << (usize::BITS - 1);
                const FOLD_UPWARD: bool = UP;
                type Key = ();
                type EncodedKey = ();
                type Output = u128;
                type Action = u64;
                type Summary = u128;
                type Tag = u64;
                type Prepared = u64;
                type Change = u128;
                type Leaf = Vec<(Range<usize>, u64)>;
                type Branch = Vec<(u128, u64)>;
                type State = ();
                fn new_state(_: usize) {}
                fn encode(_: ()) {}
                fn decode(value: u128) -> u128 {
                    value
                }
                fn unit() -> u128 {
                    0
                }
                fn tag_unit() -> u64 {
                    0
                }
                fn combine(left: u128, right: u128) -> u128 {
                    left + right
                }
                fn compose(old: u64, later: u64) -> u64 {
                    old + later
                }
                fn prepare(
                    _: &mut (),
                    action: u64,
                    _: Range<usize>,
                    _: usize,
                    _: &[Vec<Self::Branch>],
                ) -> u64 {
                    action
                }
                fn map(_: &(), value: u128, tag: u64, len: usize) -> u128 {
                    value + tag as u128 * len as u128
                }
                fn repair(value: u128, change: u128) -> u128 {
                    value + change
                }
                fn leaf(values: impl Iterator<Item = ()>) -> Self::Leaf {
                    values.for_each(drop);
                    Vec::new()
                }
                fn branch(values: &[u128]) -> Self::Branch {
                    values.iter().map(|&v| (v, 0)).collect()
                }
                unsafe fn fold_leaf<const ISA: u8>(
                    _: &(),
                    node: &Self::Leaf,
                    range: Range<usize>,
                ) -> u128 {
                    range.len() as u128
                        + node
                            .iter()
                            .map(|(r, value)| {
                                range
                                    .end
                                    .min(r.end)
                                    .saturating_sub(range.start.max(r.start))
                                    as u128
                                    * *value as u128
                            })
                            .sum::<u128>()
                }
                unsafe fn fold_branch<const ISA: u8>(
                    _: &(),
                    node: &Self::Branch,
                    range: Range<usize>,
                ) -> u128 {
                    node[range].iter().map(|&(value, _)| value).sum()
                }
                fn tag(node: &Self::Branch, lane: usize) -> u64 {
                    node[lane].1
                }
                unsafe fn edit_leaf<const ISA: u8>(
                    _: &mut (),
                    node: &mut Self::Leaf,
                    len: usize,
                    range: Range<usize>,
                    action: &u64,
                    carry: u64,
                ) -> u128 {
                    node.push((0..len, carry));
                    let change = range.len() as u128 * *action as u128;
                    node.push((range, *action));
                    change
                }
                unsafe fn edit_branch<const ISA: u8>(
                    _: &mut (),
                    node: &mut Self::Branch,
                    len: usize,
                    width: usize,
                    full: Range<usize>,
                    edges: &[Option<(usize, u128)>; 2],
                    action: &u64,
                    carry: u64,
                ) -> u128 {
                    let mut delta = 0;
                    for (lane, (value, tag)) in node.iter_mut().enumerate() {
                        let count = width.min(len - lane * width) as u128;
                        *value += count * carry as u128;
                        *tag += carry;
                        if full.contains(&lane) {
                            let change = count * *action as u128;
                            *value += change;
                            *tag += *action;
                            delta += change;
                        }
                    }
                    for &(lane, change) in edges.iter().flatten() {
                        node[lane].0 += change;
                        node[lane].1 = 0;
                        delta += change;
                    }
                    delta
                }
            }
            fn check<const UP: bool>() {
                let b = Count::<UP>::B;
                let n = b + 2;
                // This is the uniform state constructed from n unit keys; its
                // sparse storage keeps the largest valid B practical in debug tests.
                let mut tree = DaryLazySegmentTree::<Count<UP>> {
                    leaves: vec![Vec::new(), Vec::new()],
                    branches: vec![vec![vec![(b as u128, 0), (2, 0)]]],
                    total: n as u128,
                    len: n,
                    backend: SimdBackend::Scalar,
                    state: (),
                };
                assert_eq!(tree.fold(1..n), (n - 1) as u128);
                tree.update(b..b + 1, 3);
                assert_eq!(tree.fold_all(), n as u128 + 3);
                assert_eq!(tree.fold(b..b + 1), 4);
                assert_eq!(tree.fold(b + 1..n), 1);
                tree.update(..b + 1, 2);
                assert_eq!(tree.fold(..1), 3);
                assert_eq!(tree.fold(b..b + 1), 6);
                assert_eq!(tree.fold(b + 1..n), 1);
                assert_eq!(tree.fold(1..n), 3 * b as u128 + 4);
                assert_eq!(tree.fold(1..b + 1), 3 * b as u128 + 3);
                assert_eq!(tree.fold_all(), 3 * b as u128 + 7);
            }
            check::<false>();
            check::<true>();
        }
    }
}

#[codesnip::entry(
    "_SimdLazyAdapter",
    when("SimdMonoidAct", "DaryLazySegmentTree"),
    include("_SimdMonoidActBlock")
)]
mod lazy_monoid_act {
    use super::{DaryLazyKernel, DaryLazyPointKernel};
    use super::{
        SimdAggregateAct, SimdMonoid,
        simd_act_block::{apply_selected, edit_keys, leaf},
        simd_monoid::LANES,
    };
    use std::ops::Range;

    #[derive(Clone, Debug)]
    pub struct Branch<S, T> {
        values: [S; LANES],
        tags: [T; LANES],
    }

    pub enum Edit<K, T> {
        Set(K),
        Apply(T),
    }

    impl<M, A> DaryLazyKernel for (M, A)
    where
        M: SimdMonoid,
        A: SimdAggregateAct<M>,
    {
        const B: usize = LANES;
        const FOLD_UPWARD: bool = !A::HAS_CONSTANT_TAGS;
        type Key = A::Key;
        type EncodedKey = A::Key;
        type Output = M::T;
        type Action = A::Act;
        type Summary = M::T;
        type Tag = A::Act;
        type Prepared = Edit<A::Key, A::Act>;
        type Change = M::T;
        type Leaf = [A::Key; LANES];
        type Branch = Branch<M::T, A::Act>;
        type State = ();

        fn new_state(_len: usize) {}
        fn encode(key: A::Key) -> A::Key {
            key
        }
        fn decode(value: M::T) -> M::T {
            value
        }
        fn unit() -> M::T {
            M::unit()
        }
        fn tag_unit() -> A::Act {
            A::unit()
        }
        fn combine(left: M::T, right: M::T) -> M::T {
            M::operate(&left, &right)
        }
        fn compose(old: A::Act, later: A::Act) -> A::Act {
            A::operate(&old, &later)
        }
        fn prepare(
            _state: &mut (),
            action: A::Act,
            _range: Range<usize>,
            _len: usize,
            _branches: &[Vec<Self::Branch>],
        ) -> Self::Prepared {
            Edit::Apply(action)
        }
        fn map(_state: &(), value: M::T, tag: A::Act, len: usize) -> M::T {
            A::act_aggregate(value, tag, len)
        }
        fn constant(_state: &(), tag: A::Act, len: usize) -> Option<M::T> {
            A::constant_aggregate(tag, len)
        }
        fn repair(_value: M::T, change: M::T) -> M::T {
            change
        }
        fn leaf(keys: impl Iterator<Item = A::Key>) -> Self::Leaf {
            leaf(keys)
        }
        fn branch(values: &[M::T]) -> Self::Branch {
            let mut block = [M::unit(); LANES];
            block[..values.len()].copy_from_slice(values);
            Branch {
                values: block,
                tags: [A::unit(); LANES],
            }
        }
        unsafe fn fold_leaf<const ISA: u8>(
            _state: &(),
            node: &Self::Leaf,
            range: Range<usize>,
        ) -> M::T {
            let keys = &node[range];
            let mut values = [M::unit(); LANES];
            for (value, &key) in values.iter_mut().zip(keys) {
                *value = A::lift(key);
            }
            unsafe { M::reduce_lanes::<ISA>(&values[..keys.len()]) }
        }
        unsafe fn fold_branch<const ISA: u8>(
            _state: &(),
            node: &Self::Branch,
            range: Range<usize>,
        ) -> M::T {
            unsafe { M::reduce_lanes::<ISA>(&node.values[range]) }
        }
        fn tag(node: &Self::Branch, lane: usize) -> A::Act {
            node.tags[lane]
        }
        unsafe fn edit_leaf<const ISA: u8>(
            state: &mut (),
            node: &mut Self::Leaf,
            len: usize,
            range: Range<usize>,
            action: &Self::Prepared,
            carry: A::Act,
        ) -> M::T {
            let keys = &mut node[..len];
            match *action {
                Edit::Set(key) => {
                    unsafe { A::act_lanes::<ISA>(keys, carry) };
                    keys[range.start] = key;
                }
                Edit::Apply(tag) => unsafe { edit_keys::<A, ISA>(keys, range, tag, carry) },
            }
            unsafe { Self::fold_leaf::<ISA>(state, node, 0..len) }
        }
        unsafe fn edit_branch<const ISA: u8>(
            _state: &mut (),
            node: &mut Self::Branch,
            len: usize,
            width: usize,
            full: Range<usize>,
            edges: &[Option<(usize, M::T)>; 2],
            action: &Self::Prepared,
            carry: A::Act,
        ) -> M::T {
            let whole = len / width;
            let tail = len % width;
            let count = whole + usize::from(tail != 0);
            let (full, inside) = match *action {
                Edit::Apply(tag) if !full.is_empty() => (full, A::operate(&carry, &tag)),
                _ => (0..0, carry),
            };
            let equal_span = full.start.min(whole)..full.end.min(whole);
            unsafe {
                apply_selected(
                    &mut node.values[..whole],
                    equal_span,
                    carry,
                    inside,
                    |values, tag| {
                        A::act_aggregate_lanes::<ISA>(values, tag, width);
                    },
                );
                if tail != 0 {
                    let tag = if full.contains(&whole) { inside } else { carry };
                    node.values[whole] = A::act_aggregate(node.values[whole], tag, tail);
                }
                apply_selected(&mut node.tags[..count], full, carry, inside, |tags, tag| {
                    A::compose_lanes::<ISA>(tags, tag);
                });
            }
            for &(lane, change) in edges.iter().flatten() {
                node.values[lane] = change;
                node.tags[lane] = A::unit();
            }
            unsafe { M::reduce_lanes::<ISA>(&node.values[..count]) }
        }
    }

    impl<M, A> DaryLazyPointKernel for (M, A)
    where
        M: SimdMonoid,
        A: SimdAggregateAct<M>,
    {
        fn key(node: &Self::Leaf, lane: usize) -> A::Key {
            node[lane]
        }
        fn apply_key(key: A::Key, tag: A::Act) -> A::Key {
            A::act(&key, &tag)
        }
        fn prepare_set(key: A::Key) -> Self::Prepared {
            Edit::Set(key)
        }
    }
}

use std::ops::Range;
#[cfg(verus_keep_ghost)]
use vstd::{
    prelude::*,
    std_specs::cmp::{PartialOrdIs, PartialOrdSpec},
};

#[cfg_attr(verus_keep_ghost, verus_verify)]
#[cfg_attr(verus_keep_ghost, verifier::allow(autoderive_clone_without_spec))]
#[derive(Debug, Clone)]
pub struct CartesianTree {
    pub root: usize,
    pub parents: Vec<usize>,
    pub children: Vec<[usize; 2]>,
}

#[cfg_attr(verus_keep_ghost, verus_verify)]
impl CartesianTree {
    #[cfg_attr(verus_keep_ghost, verifier::rlimit(40))]
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        ensures
            result.parents.len() == a.len(), result.children.len() == a.len(),
            result.valid_parents(),
            a.len() == 0 ==> result.root == !0usize,
            a.len() > 0 ==> exists|ranges: Seq<(usize, usize)>| #[trigger] result.valid_ranges(ranges)
                && (Self::lawful_order(a@) ==> Self::range_minima(a@, ranges)
                    && Self::range_boundaries(a@, ranges)),
    ))]
    pub fn new<T>(a: &[T]) -> Self
    where
        T: PartialOrd,
    {
        let mut parents = vec![!0; a.len()];
        let mut children = vec![[!0; 2]; a.len()];
        let mut root = !0;
        #[cfg(verus_keep_ghost)]
        proof_decl! {
            let ghost mut ranges = Seq::<(usize, usize)>::empty();
            let ghost mut spine = Seq::<usize>::empty();
        }
        #[cfg_attr(verus_keep_ghost, verus_spec(iter =>
            invariant
                parents.len() == a.len(), children.len() == a.len(),
                Self::prefix_tree(parents@, children@, root, ranges, spine, iter.index() as nat),
                Self::lawful_order(a@) ==> Self::range_minima(a@, ranges),
                Self::lawful_order(a@) ==> Self::range_boundaries(a@, ranges),
        ))]
        for i in 0..a.len() {
            #[cfg(verus_keep_ghost)]
            proof_decl! { let ghost mut cut = spine.len(); }
            let mut prev = !0;
            let mut parent = i.wrapping_sub(1);
            #[cfg(verus_keep_ghost)]
            proof! {
                let maximum = usize::MAX;
                assert(maximum <= !0usize) by(bit_vector);
                assert(!0usize == usize::MAX);
                if i > 0 { assert(parent == i - 1); }
            }
            #[cfg_attr(verus_keep_ghost, verus_spec(
                invariant
                    i < a.len(), parents.len() == a.len(), children.len() == a.len(),
                    Self::prefix_tree(parents@, children@, root, ranges, spine, i as nat),
                    Self::lawful_order(a@) ==> Self::range_minima(a@, ranges),
                    Self::lawful_order(a@) ==> Self::range_boundaries(a@, ranges),
                    Self::lawful_order(a@) && prev != !0usize ==>
                        !Self::not_greater(a@[prev as int], a@[i as int]),
                    cut <= spine.len(),
                    parent == if cut == 0 { !0usize } else { spine[cut as int - 1] },
                    prev == if cut == spine.len() { !0usize } else { spine[cut as int] },
                decreases cut,
            ))]
            while parent != !0 && a[i] < a[parent] {
                #[cfg(verus_keep_ghost)]
                proof! { assert(Self::spine_link(parents@, children@, spine, cut as int - 1)); }
                prev = parent;
                parent = parents[parent];
                #[cfg(verus_keep_ghost)]
                proof! { cut = (cut - 1) as nat; }
            }
            #[cfg(verus_keep_ghost)]
            proof_decl! {
                let ghost old_ranges = ranges;
                let ghost old_spine = spine;
                let ghost old_children = children@;
                let ghost old_parents = parents@;
                let ghost old_root = root;
                let ghost retained = spine.take(cut as int);
            }
            parents[i] = parent;
            if parent != !0 {
                children[parent][1] = i;
            } else {
                root = i;
            }
            if prev != !0 {
                parents[prev] = i;
                children[i][0] = prev;
            }
            #[cfg(verus_keep_ghost)]
            proof! {
                broadcast use vstd::seq_lib::lemma_seq_contains_after_push;
                Self::spine_prefix(old_spine, cut, i as nat);
                let left = if parent == !0usize { 0usize } else { (parent + 1) as usize };
                ranges = Seq::new(i as nat, |node: int| (
                    old_ranges[node].0,
                    if retained.contains(node as usize) { (i + 1) as usize } else { old_ranges[node].1 },
                )).push((left, (i + 1) as usize));
                spine = retained.push(i);
                assert(Self::prefix_tree(old_parents, old_children, old_root, old_ranges, old_spine, i as nat));
                assert forall|node: int| 0 <= node < children.len() implies
                    (#[trigger] children@[node])[0] == if node == i { prev } else { old_children[node][0] }
                    && children@[node][1] == if node == parent { i } else { old_children[node][1] }
                    && (#[trigger] parents@[node]) == if node == i { parent }
                        else if node == prev { i } else { old_parents[node] } by {}
                if parent != !0usize {
                    assert(cut > 0);
                    assert(retained.contains(parent));
                }
                if prev != !0usize {
                    assert(cut < old_spine.len());
                    assert(!retained.contains(prev));
                }
                assert forall|node: int| 0 <= node < i + 1 implies
                    #[trigger] Self::node_range_valid(children@, ranges, node) by {
                    if node < i {
                        assert(Self::node_range_valid(old_children, old_ranges, node));
                        let left_child = old_children[node][0];
                        if left_child != !0usize {
                            assert(old_ranges[left_child as int].1 == node);
                            assert(!retained.contains(left_child));
                        }
                        let right_child = old_children[node][1];
                        if node != parent && right_child != !0usize {
                            Self::preserved_right_path(
                                old_parents, old_children, old_root, old_ranges, old_spine, cut, node,
                            );
                        }
                    } else if prev != !0usize {
                        assert(old_ranges[prev as int].1 == i);
                        if parent != !0usize {
                            assert(Self::spine_link(old_parents, old_children, old_spine, cut as int - 1));
                            assert(Self::node_range_valid(old_children, old_ranges, parent as int));
                        } else {
                            assert(old_root == prev);
                        }
                        assert(old_ranges[prev as int].0 == left);
                    } else {
                        assert(left == i);
                    }
                }
                if cut > 0 {
                    assert(retained.contains(old_root));
                }
                assert(Self::range_structure(root, children@, ranges));
                if prev != !0usize {
                    assert(Self::node_range_valid(children@, ranges, i as int));
                    assert(old_ranges[prev as int] == ranges[prev as int]);
                    assert(old_ranges[prev as int] == (left, i));
                }
                if Self::lawful_order(a@) {
                    assert forall|node: int| 0 <= node < i + 1 implies
                        #[trigger] Self::node_minimum(a@, ranges, node) by {
                        if node < i {
                            assert(Self::node_minimum(a@, old_ranges, node));
                            if retained.contains(node as usize) {
                                assert(node <= parent < i);
                                assert(Self::node_range_valid(old_children, old_ranges, node));
                                assert(old_ranges[node].0 <= parent < old_ranges[node].1);
                                assert(Self::not_greater(a@[node], a@[parent as int]));
                                assert(Self::not_greater(a@[parent as int], a@[i as int]));
                                assert(Self::not_greater(a@[node], a@[i as int]));
                                Self::append_minimum(a@, node, old_ranges[node].0 as int, i as int);
                            }
                        } else if prev != !0usize {
                            assert(Self::node_minimum(a@, old_ranges, prev as int));
                            assert(old_ranges[prev as int] == (left, i));
                            assert(!Self::not_greater(a@[prev as int], a@[i as int]));
                            assert(Self::node_range_valid(old_children, old_ranges, prev as int));
                            Self::append_minimum(a@, prev as int, left as int, i as int);
                        } else {
                            assert(left == i);
                            assert(Self::not_greater(a@[i as int], a@[i as int]));
                        }
                    }
                    assert(Self::range_minima(a@, ranges));
                    assert forall|node: int| 0 <= node < i + 1 implies
                        #[trigger] Self::node_boundaries(a@, ranges, node) by {
                        if node < i {
                            assert(Self::node_boundaries(a@, old_ranges, node));
                            assert(Self::node_range_valid(old_children, old_ranges, node));
                            if old_ranges[node].1 == i && !retained.contains(node as usize) {
                                assert(old_spine.contains(node as usize));
                                assert(prev <= node < i);
                                assert(Self::node_range_valid(old_children, old_ranges, prev as int));
                                assert(Self::node_minimum(a@, ranges, i as int));
                                assert(left <= node < i);
                                assert(Self::not_greater(a@[i as int], a@[node]));
                                assert(!Self::not_greater(a@[node], a@[i as int]));
                            }
                        } else if left > 0 {
                            assert(parent != !0usize && left == parent + 1);
                            assert(Self::not_greater(a@[parent as int], a@[i as int]));
                        }
                    }
                    assert(Self::range_boundaries(a@, ranges));
                }
                assert forall|node: int| 0 <= node < i + 1 implies
                    #[trigger] Self::node_parent_valid(parents@, children@, root, (i + 1) as nat, node) by {
                    if node < i {
                        assert(Self::node_parent_valid(old_parents, old_children, old_root, i as nat, node));
                        if parent != !0usize {
                            assert(Self::spine_link(old_parents, old_children, old_spine, cut as int - 1));
                        }
                        if prev != !0usize {
                            assert(Self::spine_link(old_parents, old_children, old_spine, cut as int));
                        }
                    }
                }
                assert forall|index: int| 0 <= index < spine.len() implies
                    #[trigger] Self::spine_link(parents@, children@, spine, index) by {
                    if index < cut {
                        assert(Self::spine_link(old_parents, old_children, old_spine, index));
                        assert(retained.contains(old_spine[index]));
                        if index + 1 < cut {
                            assert(Self::spine_ordered(old_spine, index, cut as int - 1));
                            assert(old_spine[index] != parent);
                        } else {
                            assert(old_spine[index] == parent);
                        }
                    } else {
                        assert(spine[index] == i);
                    }
                }
                assert forall|node: int| #![trigger parents@[node]] #![trigger children@[node]]
                    i + 1 <= node < parents.len() implies parents@[node] == !0usize
                        && children@[node][0] == !0usize && children@[node][1] == !0usize by {
                    assert(old_parents[node] == !0usize);
                    assert(old_children[node][0] == !0usize && old_children[node][1] == !0usize);
                }
                assert forall|node: int| 0 <= node < i + 1 implies
                    (#[trigger] ranges[node]).1 == i + 1 <==> spine.contains(node as usize) by {
                    if node < i {
                        assert(Self::node_range_valid(old_children, old_ranges, node));
                    }
                }
                assert forall|left: int, right: int| 0 <= left < right < spine.len() implies
                    #[trigger] Self::spine_ordered(spine, left, right) by {
                    if right < cut {
                        assert(Self::spine_ordered(old_spine, left, right));
                        assert(old_spine[left] < old_spine[right]);
                    } else {
                        assert(retained.contains(spine[left]));
                    }
                }
                assert(ranges.len() == i + 1 && spine.len() == cut + 1 <= i + 1);
                assert(spine[0] == root && spine.last() == i);
                assert forall|index: int| 0 <= index < spine.len() implies
                    #[trigger] spine[index] < i + 1 by {
                    if index < cut {
                        assert(retained.contains(spine[index]));
                    }
                }
                assert(Self::prefix_tree(parents@, children@, root, ranges, spine, (i + 1) as nat)) by {}
            }
        }
        #[cfg(verus_keep_ghost)]
        proof! {
            let result = Self { root, parents, children };
            #[verifier::proof_note("For every input length and any PartialOrd comparison outcomes, CartesianTree::new creates reciprocal parent/child links; the root alone has the absent-parent marker and every child is on the correct side of its parent")]
            assert(result.valid_parents());
            if a.len() > 0 {
                #[verifier::proof_note("For every nonempty input length and any PartialOrd comparison outcomes, CartesianTree::new constructs valid subtree intervals: the root spans the full input and each node's children split its interval around its own index")]
                assert(result.valid_ranges(ranges));
                if Self::lawful_order(a@) {
                    #[verifier::proof_note("For every nonempty input whose stable comparison model defines a total preorder on its values, every constructed Cartesian subtree is rooted at the leftmost minimum of its interval")]
                    assert(Self::range_minima(a@, ranges));
                    #[verifier::proof_note("Under the same total-preorder assumption, every subtree interval is maximal for its leftmost minimum: a preceding boundary value is no greater than the root, and a following boundary value is strictly smaller than the root")]
                    assert(Self::range_boundaries(a@, ranges));
                }
            }
        }
        Self {
            root,
            parents,
            children,
        }
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(
        requires exists|ranges: Seq<(usize, usize)>| #![auto]
            self.valid_ranges(ranges) && Self::callbacks_defined(ranges, f),
    ))]
    pub fn with_ranges(&self, mut f: impl FnMut(usize, Range<usize>)) {
        #[cfg(verus_keep_ghost)]
        proof_decl! {
            let ghost ranges = choose|ranges: Seq<(usize, usize)>| #![auto]
                self.valid_ranges(ranges) && Self::callbacks_defined(ranges, f);
            let ghost mut visited = Seq::<usize>::empty();
        }
        let mut stack = vec![(self.root, 0, self.parents.len())];
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost mut before = stack@; }
        #[cfg_attr(verus_keep_ghost, verus_spec(
            invariant
                self.valid_ranges(ranges), Self::callbacks_defined(ranges, f),
                self.walk_partition(ranges, stack@, visited), before == stack@,
                visited.len() <= self.parents.len(),
            ensures stack.len() == 0,
            decreases self.parents.len() - visited.len(),
        ))]
        while let Some((v, l, r)) = stack.pop() {
            #[cfg(verus_keep_ghost)]
            proof_decl! { let ghost seen = visited; }
            #[cfg(verus_keep_ghost)]
            proof! {
                assert(Self::node_range_valid(self.children@, ranges, v as int));
            }
            f(v, l..r);
            #[cfg(verus_keep_ghost)]
            proof! {
                assert(!visited.contains(v));
                visited = visited.push(v);
            }
            if self.children[v][1] != !0 {
                #[cfg(verus_keep_ghost)]
                proof! {
                    assert(Self::node_range_valid(self.children@, ranges, self.children@[v as int][1] as int));
                }
                stack.push((self.children[v][1], v + 1, r));
            }
            if self.children[v][0] != !0 {
                #[cfg(verus_keep_ghost)]
                proof! { assert(Self::node_range_valid(self.children@, ranges, self.children@[v as int][0] as int)); }
                stack.push((self.children[v][0], l, v));
            }
            #[cfg(verus_keep_ghost)]
            proof! {
                broadcast use vstd::seq_lib::lemma_seq_contains_after_push;
                assert forall|i: int| 0 <= i < stack.len() implies
                    if i < before.len() - 1 {
                        #[trigger] stack@[i] == before[i]
                    } else {
                        l <= (#[trigger] stack@[i]).1 < stack@[i].2 <= r
                            && (stack@[i].2 <= v || v < stack@[i].1)
                    } by {}
                assert forall|node: usize| #[trigger] visited.contains(node) implies
                    node < self.parents.len() && !Self::pending(stack@, node as int) by {
                    if Self::pending(stack@, node as int) {
                        let i = choose|i: int| 0 <= i < stack.len()
                            && (#[trigger] stack@[i]).1 <= node < stack@[i].2;
                        if seen.contains(node) {
                            if i < before.len() - 1 {
                                assert(before[i].1 <= node < before[i].2);
                            } else {
                                assert(before[before.len() - 1].1 <= node < before[before.len() - 1].2);
                            }
                            assert(Self::pending(before, node as int));
                        } else if i < before.len() - 1 {
                            assert(before[before.len() - 1].2 <= before[i].1);
                        }
                    }
                }
                assert forall|node: usize| node < self.parents.len() implies
                    visited.contains(node) || #[trigger] Self::pending(stack@, node as int) by {
                    if !visited.contains(node) {
                        assert(Self::pending(before, node as int));
                        let i = choose|i: int| 0 <= i < before.len()
                            && (#[trigger] before[i]).1 <= node < before[i].2;
                        if i + 1 < before.len() {
                            assert(stack@[i] == before[i]);
                        } else if node < v {
                            assert(stack@[stack.len() - 1].1 <= node < stack@[stack.len() - 1].2);
                        } else {
                            let right = before.len() - 1;
                            assert(stack@[right].1 <= node < stack@[right].2);
                        }
                    }
                }
                assert(visited.to_set().subset_of(Set::<usize>::range(0, self.parents.len())));
                visited.unique_seq_to_set();
                vstd::set_lib::lemma_len_subset(
                    visited.to_set(), Set::<usize>::range(0, self.parents.len()),
                );
            }
            #[cfg(verus_keep_ghost)]
            proof! { before = stack@; }
        }
        #[cfg(verus_keep_ghost)]
        proof! {
            assert forall|node: usize| node < self.parents.len() implies
                #[trigger] visited.contains(node) by {
                assert(!Self::pending(stack@, node as int));
            }
            #[verifier::proof_note("On normal return, with_ranges has invoked its callback exactly once per node with that node's subtree interval, for every nonempty tree whose children split those intervals and whose callback precondition holds for every reported interval")]
            assert(visited.no_duplicates()
                && forall|node: usize| node < self.parents.len() <==> #[trigger] visited.contains(node));
        }
    }
}

#[cfg(verus_keep_ghost)]
verus! {
    impl CartesianTree {
        pub open spec fn valid_ranges(&self, ranges: Seq<(usize, usize)>) -> bool {
            &&& self.parents.len() == ranges.len() && self.children.len() == ranges.len()
            &&& Self::range_structure(self.root, self.children@, ranges)
        }

        pub open spec fn range_structure(
            root: usize, children: Seq<[usize; 2]>, ranges: Seq<(usize, usize)>,
        ) -> bool {
            let n = ranges.len();
            &&& 0 < n <= usize::MAX && root < n && n <= children.len()
            &&& ranges[root as int] == (0usize, n as usize)
            &&& forall|node: int| 0 <= node < n ==>
                #[trigger] Self::node_range_valid(children, ranges, node)
        }

        pub open spec fn node_range_valid(
            children: Seq<[usize; 2]>, ranges: Seq<(usize, usize)>, node: int,
        ) -> bool {
            let n = ranges.len();
            let (l, r) = ranges[node];
            &&& l <= node < r <= n
            &&& if children[node][0] == !0usize {
                l == node
            } else {
                children[node][0] < n
                    && ranges[children[node][0] as int] == (l, node as usize)
            }
            &&& if children[node][1] == !0usize {
                r == node + 1
            } else {
                children[node][1] < n
                    && ranges[children[node][1] as int] == ((node + 1) as usize, r)
            }
        }

        pub open spec fn callbacks_defined<F>(ranges: Seq<(usize, usize)>, callback: F) -> bool
            where F: FnMut(usize, Range<usize>),
        {
            forall|node: usize| node < ranges.len() ==>
                #[trigger] call_requires(callback, (node, Range {
                    start: ranges[node as int].0, end: ranges[node as int].1,
                }))
        }

        pub proof fn unique_ranges(&self, first: Seq<(usize, usize)>, second: Seq<(usize, usize)>)
            requires self.valid_ranges(first), self.valid_ranges(second),
            ensures first == second,
        {
            assert forall|node: int| 0 <= node < first.len() implies
                #[trigger] first[node] == second[node] by {
                self.subtree_ranges_unique(first, second, self.root as int, node);
            }
            #[verifier::proof_note("For every nonempty Cartesian tree with valid subtree intervals, its root and child links determine exactly one interval for every node; this uniqueness needs no assumptions about the input ordering")]
            assert(first =~= second);
        }

        pub open spec fn valid_parents(&self) -> bool {
            let n = self.parents.len();
            &&& self.children.len() == n
            &&& if n == 0 { self.root == !0usize } else { self.root < n }
            &&& forall|node: int| 0 <= node < n ==>
                #[trigger] Self::node_parent_valid(self.parents@, self.children@, self.root, n as nat, node)
        }

        pub open spec fn node_parent_valid(
            parents: Seq<usize>, children: Seq<[usize; 2]>, root: usize, n: nat, node: int,
        ) -> bool {
            let parent = parents[node];
            let left = children[node][0];
            let right = children[node][1];
            &&& if node == root { parent == !0usize } else {
                parent < n && (node < parent && children[parent as int][0] == node
                    || parent < node && children[parent as int][1] == node)
            }
            &&& left != !0usize ==> left < node && parents[left as int] == node
            &&& right != !0usize ==> node < right < n && parents[right as int] == node
        }

        pub open spec fn not_greater<T>(left: T, right: T) -> bool
            where T: PartialOrd,
        {
            !right.is_lt(&left)
        }

        pub open spec fn lawful_order<T>(values: Seq<T>) -> bool
            where T: PartialOrd,
        {
            &&& T::obeys_partial_cmp_spec()
            &&& forall|i: int| 0 <= i < values.len() ==>
                #[trigger] Self::not_greater(values[i], values[i])
            &&& forall|i: int, j: int| 0 <= i < values.len() && 0 <= j < values.len() ==>
                #[trigger] Self::not_greater(values[i], values[j])
                    || #[trigger] Self::not_greater(values[j], values[i])
            &&& forall|i: int, j: int, k: int|
                0 <= i < values.len() && 0 <= j < values.len() && 0 <= k < values.len()
                    && #[trigger] Self::not_greater(values[i], values[j])
                    && #[trigger] Self::not_greater(values[j], values[k]) ==>
                        Self::not_greater(values[i], values[k])
        }

        pub open spec fn range_minima<T>(values: Seq<T>, ranges: Seq<(usize, usize)>) -> bool
            where T: PartialOrd,
        {
            ranges.len() <= values.len() && forall|node: int| 0 <= node < ranges.len() ==>
                #[trigger] Self::node_minimum(values, ranges, node)
        }

        pub open spec fn node_minimum<T>(
            values: Seq<T>, ranges: Seq<(usize, usize)>, node: int,
        ) -> bool
            where T: PartialOrd,
        {
            Self::interval_minimum(values, node, ranges[node].0 as int, ranges[node].1 as int)
        }

        pub open spec fn range_boundaries<T>(values: Seq<T>, ranges: Seq<(usize, usize)>) -> bool
            where T: PartialOrd,
        {
            ranges.len() <= values.len() && forall|node: int| 0 <= node < ranges.len() ==>
                #[trigger] Self::node_boundaries(values, ranges, node)
        }

        pub open spec fn node_boundaries<T>(
            values: Seq<T>, ranges: Seq<(usize, usize)>, node: int,
        ) -> bool
            where T: PartialOrd,
        {
            let (left, right) = ranges[node];
            &&& left > 0 ==> Self::not_greater(values[left as int - 1], values[node])
            &&& right < ranges.len() ==> !Self::not_greater(values[node], values[right as int])
        }

        pub open spec fn interval_minimum<T>(values: Seq<T>, node: int, left: int, right: int) -> bool
            where T: PartialOrd,
        {
            forall|index: int| left <= index < right ==>
                #[trigger] Self::not_greater(values[node], values[index])
                    && (index < node ==> !Self::not_greater(values[index], values[node]))
        }

        proof fn subtree_ranges_unique(
            &self, first: Seq<(usize, usize)>, second: Seq<(usize, usize)>, root: int, node: int,
        )
            requires self.valid_ranges(first), self.valid_ranges(second),
                0 <= root < first.len(), first[root] == second[root],
                first[root].0 <= node < first[root].1,
            ensures first[node] == second[node],
            decreases first[root].1 - first[root].0,
        {
            assert(Self::node_range_valid(self.children@, first, root));
            assert(Self::node_range_valid(self.children@, second, root));
            if node < root {
                let child = self.children@[root][0] as int;
                self.subtree_ranges_unique(first, second, child, node);
            } else if root < node {
                let child = self.children@[root][1] as int;
                self.subtree_ranges_unique(first, second, child, node);
            }
        }

        proof fn append_minimum<T>(values: Seq<T>, node: int, left: int, right: int)
            where T: PartialOrd,
            requires
                Self::lawful_order(values), 0 <= left <= node < right < values.len(),
                Self::interval_minimum(values, node, left, right),
            ensures
                Self::not_greater(values[node], values[right]) ==>
                    Self::interval_minimum(values, node, left, right + 1),
                !Self::not_greater(values[node], values[right]) ==>
                    Self::interval_minimum(values, right, left, right + 1),
        {
            if Self::not_greater(values[node], values[right]) {
                assert forall|index: int| left <= index < right + 1 implies
                    #[trigger] Self::not_greater(values[node], values[index])
                        && (index < node ==> !Self::not_greater(values[index], values[node])) by {}
            } else {
                assert(Self::not_greater(values[right], values[node]));
                assert forall|index: int| left <= index < right + 1 implies
                    #[trigger] Self::not_greater(values[right], values[index])
                        && (index < right ==> !Self::not_greater(values[index], values[right])) by {
                    if index < right {
                        assert(Self::not_greater(values[node], values[index]));
                        assert(Self::not_greater(values[right], values[index]));
                        assert(!Self::not_greater(values[index], values[right]));
                    } else {
                        assert(Self::not_greater(values[right], values[right]));
                    }
                }
            }
        }

        spec fn pending(stack: Seq<(usize, usize, usize)>, node: int) -> bool {
            exists|i: int| 0 <= i < stack.len()
                && (#[trigger] stack[i]).1 <= node < stack[i].2
        }

        spec fn walk_partition(
            &self, ranges: Seq<(usize, usize)>, stack: Seq<(usize, usize, usize)>,
            visited: Seq<usize>,
        ) -> bool {
            &&& visited.no_duplicates()
            &&& forall|node: usize| #[trigger] visited.contains(node) ==>
                node < self.parents.len() && !Self::pending(stack, node as int)
            &&& forall|node: usize| node < self.parents.len() ==>
                visited.contains(node) || #[trigger] Self::pending(stack, node as int)
            &&& forall|i: int| 0 <= i < stack.len() ==> {
                let (node, l, r) = #[trigger] stack[i];
                node < ranges.len() && ranges[node as int] == (l, r)
            }
            &&& forall|i: int, j: int| 0 <= i < j < stack.len() ==>
                (#[trigger] stack[j]).2 <= (#[trigger] stack[i]).1
        }

        spec fn prefix_tree(
            parents: Seq<usize>, children: Seq<[usize; 2]>, root: usize,
            ranges: Seq<(usize, usize)>, spine: Seq<usize>, processed: nat,
        ) -> bool {
            &&& parents.len() == children.len() && processed <= parents.len()
            &&& ranges.len() == processed && spine.len() <= processed
            &&& forall|node: int| #![trigger parents[node]] #![trigger children[node]]
                processed <= node < parents.len() ==> parents[node] == !0usize
                    && children[node][0] == !0usize && children[node][1] == !0usize
            &&& if processed == 0 {
                root == !0usize && spine.len() == 0
            } else {
                &&& Self::range_structure(root, children, ranges)
                &&& forall|node: int| 0 <= node < processed ==>
                    #[trigger] Self::node_parent_valid(parents, children, root, processed, node)
                &&& spine.len() > 0 && spine[0] == root && spine.last() == processed - 1
                &&& forall|index: int| 0 <= index < spine.len() ==> #[trigger] spine[index] < processed
                &&& forall|index: int| 0 <= index < spine.len() ==>
                    #[trigger] Self::spine_link(parents, children, spine, index)
                &&& forall|left: int, right: int| 0 <= left < right < spine.len() ==>
                    #[trigger] Self::spine_ordered(spine, left, right)
                &&& forall|node: int| 0 <= node < processed ==>
                    ((#[trigger] ranges[node]).1 == processed <==> spine.contains(node as usize))
            }
        }

        spec fn spine_link(
            parents: Seq<usize>, children: Seq<[usize; 2]>, spine: Seq<usize>, index: int,
        ) -> bool {
            let node = spine[index];
            &&& parents[node as int] == if index == 0 { !0usize } else { spine[index - 1] }
            &&& children[node as int][1] == if index + 1 == spine.len() {
                !0usize
            } else { spine[index + 1] }
        }

        spec fn spine_ordered(spine: Seq<usize>, left: int, right: int) -> bool {
            spine[left] < spine[right]
        }

        proof fn spine_prefix(spine: Seq<usize>, cut: nat, processed: nat)
            requires
                cut <= spine.len(),
                forall|index: int| 0 <= index < spine.len() ==> #[trigger] spine[index] < processed,
                forall|left: int, right: int| 0 <= left < right < spine.len() ==>
                    #[trigger] Self::spine_ordered(spine, left, right),
            ensures
                forall|index: int| 0 <= index < spine.len() ==>
                    (#[trigger] spine.take(cut as int).contains(spine[index])) == (index < cut),
                forall|node: usize| #[trigger] spine.take(cut as int).contains(node) ==>
                    spine.contains(node) && node < processed && cut > 0 && node <= spine[cut as int - 1],
                forall|node: usize| #[trigger] spine.contains(node)
                    && !spine.take(cut as int).contains(node) ==>
                    cut < spine.len() && spine[cut as int] <= node < processed,
        {
            let retained = spine.take(cut as int);
            assert forall|index: int| 0 <= index < spine.len() implies
                (#[trigger] retained.contains(spine[index])) == (index < cut) by {
                if index < cut {
                    assert(retained[index] == spine[index]);
                } else if retained.contains(spine[index]) {
                    let k = choose|k: int| 0 <= k < retained.len() && retained[k] == spine[index];
                    assert(spine[k] == spine[index]);
                    assert(Self::spine_ordered(spine, k, index));
                }
            }
            assert forall|node: usize| #[trigger] retained.contains(node) implies
                spine.contains(node) && node < processed && cut > 0 && node <= spine[cut as int - 1] by {
                let index = choose|index: int| 0 <= index < retained.len() && retained[index] == node;
                assert(spine[index] == node);
                if index + 1 < cut {
                    assert(Self::spine_ordered(spine, index, cut as int - 1));
                }
            }
            assert forall|node: usize| #[trigger] spine.contains(node) && !retained.contains(node) implies
                cut < spine.len() && spine[cut as int] <= node < processed by {
                let index = choose|index: int| 0 <= index < spine.len() && spine[index] == node;
                assert(index >= cut);
                if index > cut {
                    assert(Self::spine_ordered(spine, cut as int, index));
                }
            }
        }

        proof fn preserved_right_path(
            parents: Seq<usize>, children: Seq<[usize; 2]>, root: usize,
            ranges: Seq<(usize, usize)>, spine: Seq<usize>, cut: nat, node: int,
        )
            requires
                Self::prefix_tree(parents, children, root, ranges, spine, ranges.len()),
                cut <= spine.len(), 0 <= node < ranges.len(),
                node != if cut == 0 { !0usize } else { spine[cut as int - 1] },
                children[node][1] != !0usize,
            ensures
                spine.take(cut as int).contains(node as usize)
                    == spine.take(cut as int).contains(children[node][1]),
        {
            let retained = spine.take(cut as int);
            let right = children[node][1];
            Self::spine_prefix(spine, cut, ranges.len());
            assert(Self::node_range_valid(children, ranges, node));
            if spine.contains(node as usize) {
                let index = choose|index: int| 0 <= index < spine.len() && spine[index] == node;
                assert(Self::spine_link(parents, children, spine, index));
                assert(index + 1 < spine.len());
                assert(spine[index + 1] == right);
                assert(index + 1 != cut);
            } else {
                assert(!spine.contains(right));
            }
        }
    }
}

#[cfg(verus_keep_ghost)]
mod verification {
    use super::*;
    use crate::tools::PartialIgnoredOrd;

    verus! {
        pub fn validated_ranges(tree: &CartesianTree, Ghost(ranges): Ghost<Seq<(usize, usize)>>)
            requires tree.valid_ranges(ranges),
        {
            assert forall|node: int| 0 <= node < ranges.len() implies
                (#[trigger] ranges[node]).0 <= node < ranges[node].1 by {
                assert(CartesianTree::node_range_valid(tree.children@, ranges, node));
            }
            let visit = |node: usize, range: Range<usize>|
                requires range.start <= node < range.end,
            {
                #[verifier::proof_note("A callback that requires its node to lie in a nonempty subtree interval can be passed to with_ranges for every tree with valid subtree intervals")]
                assert(range.start <= node < range.end);
            };
            assert(CartesianTree::callbacks_defined(ranges, visit));
            tree.with_ranges(visit);
        }

        pub fn singleton_ranges() {
            let tree = CartesianTree {
                root: 0,
                parents: vec![!0usize],
                children: vec![[!0usize; 2]],
            };
            let ghost ranges = seq![(0usize, 1usize)];
            #[verifier::proof_note("The subtree-interval precondition is inhabited: a singleton Cartesian tree satisfies it and can be traversed with a callback whose precondition checks the reported interval")]
            assert(tree.valid_ranges(ranges));
            validated_ranges(&tree, Ghost(ranges));
        }

        pub fn constructed_ranges<T>(a: &[T])
            where T: PartialOrd,
            requires a.len() > 0,
        {
            let tree = CartesianTree::new(a);
            let ghost ranges = choose|ranges: Seq<(usize, usize)>| tree.valid_ranges(ranges);
            #[verifier::proof_note("For every nonempty input, the actual CartesianTree constructor establishes the subtree-interval precondition needed to traverse every node with with_ranges")]
            assert(tree.valid_ranges(ranges));
            validated_ranges(&tree, Ghost(ranges));
        }

        pub fn integer_minima(values: &[i64])
            requires values.len() > 0,
        {
            assert(CartesianTree::lawful_order(values@));
            let tree = CartesianTree::new(values);
            let ghost ranges = choose|ranges: Seq<(usize, usize)>| #[trigger] tree.valid_ranges(ranges)
                && CartesianTree::range_minima(values@, ranges)
                && CartesianTree::range_boundaries(values@, ranges);
            assert forall|node: int, index: int| 0 <= node < values.len()
                && (#[trigger] ranges[node]).0 <= index < ranges[node].1 implies
                values@[node] <= (#[trigger] values@[index])
                    && (index < node ==> values@[node] < values@[index]) by {
                assert(CartesianTree::node_minimum(values@, ranges, node));
                assert(CartesianTree::not_greater(values@[node], values@[index]));
            }
            assert forall|node: int| 0 <= node < values.len() implies
                ((#[trigger] ranges[node]).0 > 0 ==>
                    values@[ranges[node].0 as int - 1] <= values@[node])
                && (ranges[node].1 < values.len() ==>
                    values@[ranges[node].1 as int] < values@[node]) by {
                assert(CartesianTree::node_boundaries(values@, ranges, node));
            }
            #[verifier::proof_note("For every nonempty i64 input, the ordering precondition holds and the actual constructor produces maximal intervals for each node's leftmost minimum, including repeated and extreme values")]
            assert(tree.valid_ranges(ranges) && CartesianTree::range_minima(values@, ranges)
                && CartesianTree::range_boundaries(values@, ranges));
        }

        fn key_minima<U>(values: &[PartialIgnoredOrd<i64, U>]) {
            assert(CartesianTree::lawful_order(values@));
            let tree = CartesianTree::new(values);
            if values.len() > 0 {
                let ghost ranges = choose|ranges: Seq<(usize, usize)>| #[trigger] tree.valid_ranges(ranges)
                    && CartesianTree::range_minima(values@, ranges)
                    && CartesianTree::range_boundaries(values@, ranges);
                assert(CartesianTree::node_minimum(values@, ranges, tree.root as int));
                assert forall|i: int| 0 <= i < values.len() implies
                    values@[tree.root as int].0 <= (#[trigger] values@[i]).0
                        && (i < tree.root ==> values@[tree.root as int].0 < values@[i].0) by {
                    assert(CartesianTree::not_greater(values@[tree.root as int], values@[i]));
                }
                #[verifier::proof_note("For arbitrary i64 keys with unconstrained payloads, constructing a Cartesian tree chooses the leftmost minimum key as the root and establishes the subtree-minimum guarantees without inspecting payloads")]
                assert(tree.valid_ranges(ranges) && CartesianTree::range_minima(values@, ranges));
            }
        }

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::MinOperation,
        crecurse,
        data_structure::SegmentTree,
        tools::{Xorshift, testutil::exhaustive_sequences},
    };

    #[test]
    fn test_cartesian_tree() {
        const Q: usize = 1000;
        const N: usize = 100;
        const A: i64 = 100;
        let mut rng = Xorshift::default();
        let arrays = exhaustive_sequences(-1..=1, 0..=6).chain((0..Q).map(|_| {
            let n = rng.random(1..=N);
            rng.random_iter(0..A).take(n).collect()
        }));
        for a in arrays {
            let n = a.len();
            let mut seg = SegmentTree::<MinOperation<_>>::from_vec(
                a.iter().enumerate().map(|(i, &a)| (a, i)).collect(),
            );
            let mut parents = vec![!0; n];
            let mut root = !0;
            let mut children = vec![[!0; 2]; n];
            let mut ranges = vec![];
            crecurse!(
                unsafe fn dfs(l: usize, r: usize, p: usize, ci: usize) {
                    if l >= r {
                        return;
                    }
                    let m = seg.fold(l..r).1;
                    ranges.push((m, l..r));
                    if p == !0 {
                        root = m;
                    } else {
                        parents[m] = p;
                        children[p][ci] = m;
                    }
                    seg.set(m, (i64::MAX, usize::MAX));
                    dfs!(l, m, m, 0);
                    dfs!(m + 1, r, m, 1);
                }
            )(0, n, !0, !0);

            let ct = CartesianTree::new(&a);
            assert_eq!(ct.root, root);
            assert_eq!(ct.parents, parents);
            assert_eq!(ct.children, children);
            let mut ct_ranges = vec![];
            if !a.is_empty() {
                ct.with_ranges(|v, range| {
                    ct_ranges.push((v, range));
                });
            }
            assert_eq!(ct_ranges, ranges);
        }
    }
}

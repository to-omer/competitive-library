use std::ops::Index;
#[cfg(verus_keep_ghost)]
use verification::*;
#[cfg(verus_keep_ghost)]
use vstd::{prelude::*, std_specs::cmp::PartialEqSpec};

#[cfg_attr(verus_keep_ghost, verus_verify)]
#[cfg_attr(verus_keep_ghost, verifier::allow(autoderive_clone_without_spec))]
#[derive(Clone, Debug)]
pub struct SuffixArray {
    sa: Vec<usize>,
}

#[cfg_attr(verus_keep_ghost, verus_verify)]
impl SuffixArray {
    #[cfg_attr(verus_keep_ghost, verifier::external)]
    pub fn new<T>(text: &[T]) -> Self
    where
        T: Ord,
    {
        let n = text.len();
        let mut ord: Vec<_> = (0..n).collect();
        ord.sort_unstable_by_key(|&i| &text[i]);
        let mut s = vec![0usize; n + 1];
        let mut upper = 0usize;
        for (k, &i) in ord.iter().enumerate() {
            if k == 0 || text[ord[k - 1]] != text[i] {
                upper += 1;
            }
            s[i] = upper;
        }
        s[n] = 0;
        let sa = sa_is(&s, upper);
        Self { sa }
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(result => ensures result@ == self@))]
    pub fn into_inner(self) -> Vec<usize> {
        self.sa
    }

    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires self.valid_for(text@),
        ensures result.0.len() == text.len(), inverse_ranks(self@, result.1@),
            forall|i: int| 0 <= i < result.0.len() ==>
                common_prefix(text@, self@[i] as int, self@[i + 1] as int, result.0@[i] as int),
    ))]
    pub fn lcp_array_with_rank<T>(&self, text: &[T]) -> (Vec<usize>, Vec<usize>)
    where
        T: PartialEq,
    {
        #[cfg(verus_keep_ghost)]
        proof_decl! {
            let ghost keys = choose|keys: Seq<int>| #![auto] key_model(text@, keys) && sorted_suffixes(keys, self@);
        }
        let n = self.sa.len() - 1;
        #[cfg(verus_keep_ghost)]
        proof! { empty_suffix_first(keys, self@); }
        let mut rank = vec![0usize; n + 1];
        #[cfg_attr(verus_keep_ghost, verus_spec(cursor =>
            invariant n == text.len(), permutation(self@, n as nat),
                rank.len() == n + 1,
                forall|p: int| 0 <= p < cursor.index() ==> #[trigger] rank@[self@[p] as int] == p,
        ))]
        for i in 0..=n {
            rank[self.sa[i]] = i;
        }
        #[cfg(verus_keep_ghost)]
        proof! {
            assert forall|p: int| 0 <= p <= n implies
                (#[trigger] rank@[p]) < self@.len() && self@[rank@[p] as int] == p by {
                assert(self@.contains(p as usize));
                let r = self@.lemma_contains_to_index(p as usize);
                assert(rank@[p] == r);
            }
            assert(inverse_ranks(self@, rank@));
        }

        let mut h = 0usize;
        let mut lcp_array = vec![0usize; n];
        #[cfg(verus_keep_ghost)]
        proof_decl! {
            let ghost mut successes = 0int;
            let ghost mut failures = 0int;
        }
        #[cfg_attr(verus_keep_ghost, verus_spec(cursor =>
            invariant n == text.len(), n < usize::MAX, T::obeys_eq_spec(),
                key_model(text@, keys), permutation(self@, n as nat), sorted_suffixes(keys, self@),
                inverse_ranks(self@, rank@), self@[0] == n, lcp_array.len() == n,
                h <= n - cursor.index(),
                0 <= successes <= cursor.index() + h,
                0 <= failures <= cursor.index(),
                cursor.index() < n ==> matched(keys, cursor.index(),
                    self@[rank@[cursor.index()] as int - 1] as int, h as int),
                forall|p: int| 0 <= p < cursor.index() ==> #[trigger]
                    longest(keys, self@[rank@[p] as int - 1] as int, p,
                        lcp_array@[rank@[p] as int - 1] as int),
        ))]
        for i in 0..n {
            #[cfg(verus_keep_ghost)]
            proof! { assert(rank@[i as int] > 0); }
            let r = rank[i] - 1;
            let j = self.sa[r];
            #[cfg_attr(verus_keep_ghost, verus_spec(
                invariant n == text.len(), n < usize::MAX, T::obeys_eq_spec(),
                    key_model(text@, keys), i < n, j <= n,
                    matched(keys, i as int, j as int, h as int),
                    0 <= successes <= i + h,
                decreases n - i - h,
            ))]
            while i + h < n && j + h < n && text[i + h] == text[j + h] {
                #[cfg(verus_keep_ghost)]
                proof! {
                    successes = successes + 1;
                    assert(same_at(keys, i as int, j as int, h as int));
                    assert forall|k: int| 0 <= k < h + 1 implies
                        #[trigger] same_at(keys, i as int, j as int, k) by {
                        if k < h { assert(same_at(keys, i as int, j as int, k)); }
                    }
                }
                h += 1;
            }
            #[cfg(verus_keep_ghost)]
            proof! {
                if i + h < n && j + h < n { failures = failures + 1; }
                matched_symmetric(keys, i as int, j as int, h as int);
                assert(longest(keys, j as int, i as int, h as int));
                if i + 1 < n { shift_lower_bound(keys, self@, rank@, i as int, h as int); }
            }
            lcp_array[r] = h;
            h = h.saturating_sub(1);
        }
        #[cfg(verus_keep_ghost)]
        proof! {
            assert forall|r: int| 0 <= r < n implies
                #[trigger] common_prefix(text@, self@[r] as int, self@[r + 1] as int, lcp_array@[r] as int) by {
                let i = self@[r + 1] as int;
                assert(i < n && rank@[i] == r + 1);
                assert(longest(keys, self@[rank@[i] as int - 1] as int, i,
                    lcp_array@[rank@[i] as int - 1] as int));
                assert(longest(keys, self@[r] as int, i, lcp_array@[r] as int));
                assert forall|k: int| 0 <= k < lcp_array@[r] implies
                    #[trigger] text@[self@[r] as int + k].eq_spec(&text@[i + k]) by {
                    assert(same_at(keys, self@[r] as int, i, k));
                }
            }
            #[verifier::proof_note("Given a permutation of all suffix starts sorted lexicographically under keys consistent with text equality, LCP construction returns the exact longest common prefix of every adjacent suffix pair and the exact inverse rank permutation, for arbitrary text lengths including zero")]
            assert(inverse_ranks(self@, rank@));
            #[verifier::proof_note("LCP construction makes at most 2n equality comparisons for text length n; the bound excludes the internal cost of equality and allocation, and termination assumes these called operations terminate")]
            assert(successes + failures <= 2 * n);
        }
        (lcp_array, rank)
    }

    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires self.valid_for(text@),
        ensures result.len() == text.len(),
            forall|i: int| 0 <= i < result.len() ==>
                common_prefix(text@, self@[i] as int, self@[i + 1] as int, result@[i] as int),
    ))]
    pub fn lcp_array<T>(&self, text: &[T]) -> Vec<usize>
    where
        T: PartialEq,
    {
        self.lcp_array_with_rank(text).0
    }
}

#[cfg_attr(verus_keep_ghost, verus_verify)]
impl Index<usize> for SuffixArray {
    type Output = usize;
    #[cfg_attr(verus_keep_ghost, verus_spec(result => ensures *result == self@[index as int]))]
    fn index(&self, index: usize) -> &Self::Output {
        &self.sa[index]
    }
}

#[cfg(verus_keep_ghost)]
verus! {
    impl View for SuffixArray {
        type V = Seq<usize>;
        closed spec fn view(&self) -> Seq<usize> { self.sa@ }
    }

    impl vstd::std_specs::core::IndexSpecImpl<usize> for SuffixArray {
        open spec fn index_req(&self, index: &usize) -> bool { *index < self@.len() }
    }

    impl SuffixArray {
        pub closed spec fn valid_for<T>(&self, text: Seq<T>) -> bool
            where T: PartialEq,
        {
            &&& T::obeys_eq_spec()
            &&& permutation(self@, text.len())
            &&& exists|keys: Seq<int>| key_model(text, keys) && sorted_suffixes(keys, self@)
        }
    }
}

fn induced_sort(s: &[usize], upper: usize, is_s: &[bool], lms: &[usize]) -> Vec<usize> {
    let n = s.len();
    let mut sa = vec![!0usize; n];

    let mut bucket_size = vec![0usize; upper + 1];
    for &c in s {
        bucket_size[c] += 1;
    }

    let mut bucket_tail = vec![0usize; upper + 1];
    {
        let mut sum = 0usize;
        for i in 0..=upper {
            sum += bucket_size[i];
            bucket_tail[i] = sum;
        }
    }
    for &pos in lms.iter().rev() {
        let c = s[pos];
        bucket_tail[c] -= 1;
        sa[bucket_tail[c]] = pos;
    }

    let mut bucket_head = vec![0usize; upper + 1];
    {
        let mut sum = 0usize;
        for i in 0..=upper {
            bucket_head[i] = sum;
            sum += bucket_size[i];
        }
    }
    for i in 0..n {
        let v = sa[i];
        if v == !0 || v == 0 {
            continue;
        }
        let p = v - 1;
        if !is_s[p] {
            let c = s[p];
            sa[bucket_head[c]] = p;
            bucket_head[c] += 1;
        }
    }

    let mut bucket_tail = vec![0usize; upper + 1];
    {
        let mut sum = 0usize;
        for i in 0..=upper {
            sum += bucket_size[i];
            bucket_tail[i] = sum;
        }
    }
    for i in (0..n).rev() {
        let v = sa[i];
        if v == !0 || v == 0 {
            continue;
        }
        let p = v - 1;
        if is_s[p] {
            let c = s[p];
            bucket_tail[c] -= 1;
            sa[bucket_tail[c]] = p;
        }
    }

    sa
}

fn sa_is(s: &[usize], upper: usize) -> Vec<usize> {
    let n = s.len();
    match n {
        0 => return vec![],
        1 => return vec![0],
        _ => {}
    }

    let mut is_s = vec![false; n];
    is_s[n - 1] = true;
    for i in (0..n - 1).rev() {
        is_s[i] = if s[i] < s[i + 1] {
            true
        } else if s[i] > s[i + 1] {
            false
        } else {
            is_s[i + 1]
        };
    }

    let mut lms_map = vec![!0; n];
    let mut lms = vec![];
    for i in 1..n {
        if is_s[i] && !is_s[i - 1] {
            lms_map[i] = lms.len();
            lms.push(i);
        }
    }

    let mut sa = induced_sort(s, upper, &is_s, &lms);

    if lms.len() > 1 {
        let mut rec_s = vec![0usize; lms.len()];
        let mut rec_upper = 0usize;
        let mut prev = !0usize;
        for &pos in &sa {
            if pos != !0 && lms_map[pos] != !0 {
                if prev != !0 {
                    let mut offset = 0usize;
                    loop {
                        if s[prev + offset] != s[pos + offset]
                            || is_s[prev + offset] != is_s[pos + offset]
                        {
                            rec_upper += 1;
                            break;
                        }
                        if offset != 0 {
                            let prev_is_lms = lms_map[prev + offset] != !0;
                            let pos_is_lms = lms_map[pos + offset] != !0;
                            if prev_is_lms || pos_is_lms {
                                rec_upper += (prev_is_lms != pos_is_lms) as usize;
                                break;
                            }
                        }
                        offset += 1;
                    }
                }
                rec_s[lms_map[pos]] = rec_upper;
                prev = pos;
            }
        }

        let rec_sa = if rec_upper + 1 == lms.len() {
            let mut rec_sa = vec![0usize; lms.len()];
            for i in 0..lms.len() {
                rec_sa[rec_s[i]] = i;
            }
            rec_sa
        } else {
            sa_is(&rec_s, rec_upper)
        };

        let mut ordered_lms = Vec::with_capacity(lms.len());
        for &i in &rec_sa {
            ordered_lms.push(lms[i]);
        }
        sa = induced_sort(s, upper, &is_s, &ordered_lms);
    }

    sa
}

#[cfg(verus_keep_ghost)]
mod verification {
    use super::*;

    verus! {
        fn constant_text(n: usize, symbol: u8) -> (result: (Vec<usize>, Vec<usize>))
            requires n < usize::MAX,
            ensures result.0.len() == n, result.1.len() == n + 1,
                forall|r: int| 0 <= r < n ==> #[trigger] result.0@[r] == r,
                forall|p: int| 0 <= p <= n ==> #[trigger] result.1@[p] == n - p,
        {
            let text = vec![symbol; n];
            let mut sa = vec![0usize; n + 1];
            for r in 0..=n
                invariant sa.len() == n + 1,
                    forall|k: int| 0 <= k < r ==> #[trigger] sa@[k] == n - k,
            {
                sa[r] = n - r;
            }
            let ghost keys = Seq::new(n as nat, |_: int| 0int);
            proof {
                assert forall|p: usize| p <= n implies #[trigger] sa@.contains(p) by {
                    assert(sa@[n - p] == p);
                }
                assert forall|r: int, s: int| 0 <= r < s < sa.len() implies
                    #[trigger] lex_less(keys, sa@[r] as int, sa@[s] as int) by {
                    constant_order(keys, sa@[r] as int, sa@[s] as int);
                }
                assert(permutation(sa@, n as nat));
                assert(key_model(text@, keys));
                assert(sorted_suffixes(keys, sa@));
            }
            let suffixes = SuffixArray { sa };
            assert(suffixes.valid_for(text@));
            let result = suffixes.lcp_array_with_rank(&text);
            proof {
                assert forall|r: int| 0 <= r < n implies #[trigger] result.0@[r] == r by {
                    assert(common_prefix(text@, suffixes@[r] as int,
                        suffixes@[r + 1] as int, result.0@[r] as int));
                }
                #[verifier::proof_note("For every constant byte string, including the empty string, a concrete descending suffix permutation satisfies the LCP precondition and the actual LCP method returns [0, 1, ..., n - 1] with inverse ranks n - position")]
                assert(result.0.len() == n);
            }
            result
        }

        proof fn constant_order(keys: Seq<int>, a: int, b: int)
            requires 0 <= b < a <= keys.len(),
                forall|i: int| 0 <= i < keys.len() ==> #[trigger] keys[i] == 0,
            ensures lex_less(keys, a, b),
            decreases keys.len() - a,
        {
            if a < keys.len() { constant_order(keys, a + 1, b + 1); }
        }

        pub open spec fn permutation(sa: Seq<usize>, n: nat) -> bool {
            &&& n < usize::MAX
            &&& sa.len() == n + 1
            &&& sa.no_duplicates()
            &&& forall|i: int| 0 <= i < sa.len() ==> #[trigger] sa[i] <= n
            &&& forall|p: usize| p <= n ==> #[trigger] sa.contains(p)
        }

        pub open spec fn inverse_ranks(sa: Seq<usize>, rank: Seq<usize>) -> bool {
            &&& rank.len() == sa.len()
            &&& forall|i: int| 0 <= i < sa.len() ==>
                #[trigger] sa[i] < rank.len() && rank[sa[i] as int] == i
            &&& forall|p: int| 0 <= p < rank.len() ==>
                #[trigger] rank[p] < sa.len() && sa[rank[p] as int] == p
        }

        pub open spec fn key_model<T>(text: Seq<T>, keys: Seq<int>) -> bool
            where T: PartialEq,
        {
            keys.len() == text.len() && forall|i: int, j: int|
                0 <= i < text.len() && 0 <= j < text.len() ==>
                    #[trigger] text[i].eq_spec(&text[j]) == (keys[i] == keys[j])
        }

        pub open spec fn lex_less(keys: Seq<int>, left: int, right: int) -> bool
            recommends 0 <= left <= keys.len(), 0 <= right <= keys.len(),
            decreases keys.len() - left,
        {
            if left >= keys.len() { right < keys.len() }
            else if right >= keys.len() { false }
            else { keys[left] < keys[right]
                || (keys[left] == keys[right] && lex_less(keys, left + 1, right + 1)) }
        }

        pub open spec fn sorted_suffixes(keys: Seq<int>, sa: Seq<usize>) -> bool {
            forall|i: int, j: int| 0 <= i < j < sa.len() ==>
                #[trigger] lex_less(keys, sa[i] as int, sa[j] as int)
        }

        pub open spec fn same_at(keys: Seq<int>, a: int, b: int, k: int) -> bool {
            keys[a + k] == keys[b + k]
        }

        pub open spec fn matched(keys: Seq<int>, a: int, b: int, h: int) -> bool {
            &&& 0 <= a <= keys.len()
            &&& 0 <= b <= keys.len()
            &&& 0 <= h <= keys.len() - a
            &&& h <= keys.len() - b
            &&& forall|k: int| 0 <= k < h ==> #[trigger] same_at(keys, a, b, k)
        }

        pub open spec fn longest(keys: Seq<int>, a: int, b: int, h: int) -> bool {
            matched(keys, a, b, h) && (a + h == keys.len() || b + h == keys.len()
                || keys[a + h] != keys[b + h])
        }

        pub open spec fn common_prefix<T>(text: Seq<T>, a: int, b: int, h: int) -> bool
            where T: PartialEq,
        {
            &&& 0 <= a <= text.len()
            &&& 0 <= b <= text.len()
            &&& 0 <= h <= text.len() - a
            &&& h <= text.len() - b
            &&& forall|k: int| 0 <= k < h ==> #[trigger] text[a + k].eq_spec(&text[b + k])
            &&& a + h == text.len() || b + h == text.len() || !text[a + h].eq_spec(&text[b + h])
        }

        pub proof fn empty_suffix_first(keys: Seq<int>, sa: Seq<usize>)
            requires permutation(sa, keys.len()), sorted_suffixes(keys, sa),
            ensures sa[0] == keys.len(),
        {
            let n = keys.len();
            assert(sa.contains(n as usize));
            let i = sa.lemma_contains_to_index(n as usize);
            if i > 0 { assert(lex_less(keys, sa[0] as int, n as int)); }
        }

        pub proof fn matched_symmetric(keys: Seq<int>, a: int, b: int, h: int)
            requires matched(keys, a, b, h),
            ensures matched(keys, b, a, h),
        {
            assert forall|k: int| 0 <= k < h implies #[trigger] same_at(keys, b, a, k) by {
                assert(same_at(keys, a, b, k));
            }
        }

        proof fn lex_asymmetric(keys: Seq<int>, a: int, b: int)
            requires 0 <= a <= keys.len(), 0 <= b <= keys.len(),
            ensures !(lex_less(keys, a, b) && lex_less(keys, b, a)),
            decreases keys.len() - a,
        {
            if a < keys.len() && b < keys.len() && keys[a] == keys[b] {
                lex_asymmetric(keys, a + 1, b + 1);
            }
        }

        proof fn prefix_interval(keys: Seq<int>, a: int, b: int, c: int, h: int)
            requires matched(keys, a, c, h), 0 <= b <= keys.len(),
                lex_less(keys, a, b), lex_less(keys, b, c),
            ensures matched(keys, b, c, h),
            decreases h,
        {
            if h > 0 {
                assert(same_at(keys, a, c, 0));
                assert(keys[a] == keys[c]);
                assert(b < keys.len() && keys[a] == keys[b]);
                assert forall|k: int| 0 <= k < h - 1 implies
                    #[trigger] same_at(keys, a + 1, c + 1, k) by {
                    assert(same_at(keys, a, c, k + 1));
                }
                prefix_interval(keys, a + 1, b + 1, c + 1, h - 1);
                assert forall|k: int| 0 <= k < h implies
                    #[trigger] same_at(keys, b, c, k) by {
                    if k > 0 { assert(same_at(keys, b + 1, c + 1, k - 1)); }
                }
            }
        }

        pub proof fn shift_lower_bound(keys: Seq<int>, sa: Seq<usize>, rank: Seq<usize>, i: int, h: int)
            requires permutation(sa, keys.len()), sorted_suffixes(keys, sa), inverse_ranks(sa, rank),
                sa[0] == keys.len(), 0 <= i < keys.len() - 1,
                matched(keys, i, sa[rank[i] as int - 1] as int, h),
            ensures matched(keys, i + 1, sa[rank[i + 1] as int - 1] as int,
                if h > 0 { h - 1 } else { 0int }),
        {
            assert(rank[i] > 0 && rank[i + 1] > 0);
            let j = sa[rank[i] as int - 1] as int;
            let next = sa[rank[i + 1] as int - 1] as int;
            if h > 1 {
                assert(same_at(keys, i, j, 0));
                assert(keys[i] == keys[j]);
                assert(lex_less(keys, j, i));
                assert(lex_less(keys, j + 1, i + 1));
                if rank[j + 1] >= rank[i + 1] {
                    if rank[j + 1] == rank[i + 1] { assert(j + 1 == i + 1); }
                    else {
                        assert(lex_less(keys, i + 1, j + 1));
                        lex_asymmetric(keys, i + 1, j + 1);
                    }
                }
                assert forall|k: int| 0 <= k < h - 1 implies
                    #[trigger] same_at(keys, j + 1, i + 1, k) by {
                    assert(same_at(keys, i, j, k + 1));
                }
                if rank[j + 1] < rank[i + 1] - 1 {
                    assert(lex_less(keys, j + 1, next));
                    assert(lex_less(keys, next, i + 1));
                    prefix_interval(keys, j + 1, next, i + 1, h - 1);
                } else { assert(next == j + 1); }
                matched_symmetric(keys, next, i + 1, h - 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    use crate::tools::testutil::sequence_cases;

    #[test]
    fn test_suffix_array() {
        let mut rng = Xorshift::default();
        for s in sequence_cases(&mut rng, 0..=19, 0..=4_096) {
            let n = s.len();
            let sa = SuffixArray::new(&s);
            let mut suffixes: Vec<_> = (0..=n).collect();
            suffixes.sort_unstable_by_key(|&i| &s[i..]);
            assert_eq!(sa.sa, suffixes);
        }
    }

    #[test]
    fn test_lcp_array() {
        let mut rng = Xorshift::default();
        for s in sequence_cases(&mut rng, 0..=19, 0..=3_072) {
            let n = s.len();
            let suffix_array = SuffixArray::new(&s);
            let (lcp_array, rank) = suffix_array.lcp_array_with_rank(&s);
            assert_eq!(rank.len(), s.len() + 1);
            let mut sa: Vec<_> = (0..=n).collect();
            sa.sort_unstable_by_key(|&i| &s[i..]);
            for (i, &pos) in sa.iter().enumerate() {
                assert_eq!(suffix_array[i], pos);
            }
            if n == 0 {
                assert!(lcp_array.is_empty());
                continue;
            }
            for i in 1..=n {
                let h = s[sa[i - 1]..]
                    .iter()
                    .zip(s[sa[i]..].iter())
                    .take_while(|(a, b)| a == b)
                    .count();
                assert_eq!(lcp_array[i - 1], h);
            }
        }
    }
}

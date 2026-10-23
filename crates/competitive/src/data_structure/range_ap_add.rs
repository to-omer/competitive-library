#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

#[cfg_attr(verus_keep_ghost, verus_verify)]
// Clone itself has no formal postcondition.
#[cfg_attr(verus_keep_ghost, verifier::allow(autoderive_clone_without_spec))]
#[derive(Debug, Clone)]
pub struct RangeArithmeticProgressionAdd {
    pub dd: Vec<i64>,
}
#[cfg_attr(verus_keep_ghost, verus_verify)]
impl RangeArithmeticProgressionAdd {
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        ensures
            result.dd.len() == n,
            forall|i: int| 0 <= i < n ==> #[trigger] result.dd@[i] == 0,
    ))]
    pub fn new(n: usize) -> Self {
        Self { dd: vec![0; n] }
    }
    /// add a, a+d, ..., a+(k-1)d into [l, l + k)
    #[cfg_attr(verus_keep_ghost, verus_spec(
        requires valid_update(old(self).dd@, l as int, k as int, a as int, d as int),
        ensures progression_added(old(self).dd@, final(self).dd@, l as int, k as int, a as int, d as int),
    ))]
    pub fn update(&mut self, l: usize, k: usize, a: i64, d: i64) {
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost s0 = self.dd@; }
        if let Some(e) = self.dd.get_mut(l) {
            *e += a;
        }
        #[cfg(verus_keep_ghost)]
        proof! { assert(self.dd@ == add_at(s0, l as int, a as int)); }
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost s1 = self.dd@; }
        if let Some(e) = self.dd.get_mut(l + 1) {
            *e += d - a;
        }
        #[cfg(verus_keep_ghost)]
        proof! { assert(self.dd@ == add_at(s1, l + 1, d - a)); }
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost s2 = self.dd@; }
        if let Some(e) = self.dd.get_mut(l + k) {
            *e += -a - k as i64 * d;
        }
        #[cfg(verus_keep_ghost)]
        proof! { assert(self.dd@ == add_at(s2, l + k, -a - k * d)); }
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost s3 = self.dd@; }
        if let Some(e) = self.dd.get_mut(l + k + 1) {
            *e += a + (k as i64 - 1) * d;
        }
        #[cfg(verus_keep_ghost)]
        proof! {
            assert(self.dd@ == add_at(s3, l + k + 1, a + (k - 1) * d));
            assert forall|i: int| 0 <= i < s0.len() implies
                #[trigger] double_prefix_sum(self.dd@, i + 1)
                    == double_prefix_sum(s0, i + 1)
                        + if l <= i < l + k { a + (i - l) * d } else { 0 } by {
                double_prefix_sum_after_add(s0, l as int, a as int, i + 1);
                double_prefix_sum_after_add(s1, l + 1, d - a, i + 1);
                double_prefix_sum_after_add(s2, l + k, -a - k * d, i + 1);
                double_prefix_sum_after_add(s3, l + k + 1, a + (k - 1) * d, i + 1);
                let u0 = if l < i + 1 { i + 1 - l } else { 0 };
                let u1 = if l + 1 < i + 1 { i - l } else { 0 };
                let u2 = if l + k < i + 1 { i + 1 - l - k } else { 0 };
                let u3 = if l + k + 1 < i + 1 { i - l - k } else { 0 };
                assert(u0 * a + u1 * (d - a) + u2 * (-a - k * d) + u3 * (a + (k - 1) * d)
                    == if l <= i < l + k { a + (i - l) * d } else { 0 }) by(nonlinear_arith)
                    requires
                        k >= 0,
                        u0 == if l < i + 1 { i + 1 - l } else { 0 },
                        u1 == if l + 1 < i + 1 { i - l } else { 0 },
                        u2 == if l + k < i + 1 { i + 1 - l - k } else { 0 },
                        u3 == if l + k + 1 < i + 1 { i - l - k } else { 0 };
            }
            #[verifier::proof_note("For any buffer length, update adds a + (i - l)d exactly at positions in [l, l + k), including empty and clipped intervals, provided indices, coefficient expressions, and intermediate i64 buffer additions fit")]
            assert(progression_added(s0, self.dd@, l as int, k as int, a as int, d as int));
        }
    }
    /// add a, a+d, ..., a+(k-1)d into [l, l + k)
    #[cfg_attr(verus_keep_ghost, verus_spec(
        requires
            k <= isize::MAX,
            l >= 0 ==> valid_update(old(self).dd@, l as int, k as int, a as int, d as int),
            l < 0 && l + k > 0 ==> (
                i64::MIN <= l * d <= i64::MAX
                && i64::MIN <= a - l * d <= i64::MAX
                && valid_update(old(self).dd@, 0, l + k, a - l * d, d as int)
            ),
        ensures progression_added(old(self).dd@, final(self).dd@, l as int, k as int, a as int, d as int),
    ))]
    pub fn update_isize(&mut self, l: isize, k: usize, a: i64, d: i64) {
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost initial = self.dd@; }
        if l < 0 {
            let r = l + k as isize;
            if r > 0 {
                self.update(0, r as usize, a - l as i64 * d, d);
            }
        } else {
            self.update(l as usize, k, a, d);
        }
        #[cfg(verus_keep_ghost)]
        proof! {
            assert forall|i: int| 0 <= i < initial.len() implies
                #[trigger] double_prefix_sum(self.dd@, i + 1)
                    == double_prefix_sum(initial, i + 1)
                        + if l <= i < l + k { a + (i - l) * d } else { 0 } by {
                if l < 0 && l + k > 0 {
                    assert(a - l * d + i * d == a + (i - l) * d) by(nonlinear_arith);
                }
            }
            #[verifier::proof_note("For arbitrary buffer lengths and signed starts, update_isize clips an arithmetic progression at both array boundaries without changing its values, under the index and intermediate i64 arithmetic preconditions")]
            assert(progression_added(initial, self.dd@, l as int, k as int, a as int, d as int));
        }
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(
        requires
            prefix_sums_bounded(old(self).dd@),
            prefix_sums_bounded(prefix_sums(old(self).dd@)),
        ensures
            final(self).dd@ == prefix_sums(prefix_sums(old(self).dd@)),
            forall|i: int| 0 <= i < old(self).dd.len()
                ==> #[trigger] final(self).dd@[i] == double_prefix_sum(old(self).dd@, i + 1),
    ))]
    pub fn build_inplace(&mut self) {
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost initial = self.dd@; }
        #[cfg_attr(verus_keep_ghost, verus_spec(iter =>
            invariant
                self.dd.len() == initial.len(),
                prefix_sums_bounded(initial),
                prefix_sums_bounded(prefix_sums(initial)),
                iter.index() == 0 ==> self.dd@ == initial,
                iter.index() == 1 ==> self.dd@ == prefix_sums(initial),
                iter.index() == 2 ==> self.dd@ == prefix_sums(prefix_sums(initial)),
        ))]
        for _ in 0..2 {
            #[cfg(verus_keep_ghost)]
            proof_decl! { let ghost start = self.dd@; }
            #[cfg(verus_keep_ghost)]
            proof! {
                assert(prefix_sums_bounded(start));
                if self.dd.len() > 0 {
                    reveal_with_fuel(prefix_sum, 2);
                    assert(self.dd[0] == prefix_sum(start, 1));
                }
            }
            let n = self.dd.len();
            #[cfg_attr(verus_keep_ghost, verus_spec(iter =>
                invariant
                    self.dd.len() == start.len(), n == start.len(),
                    prefix_sums_bounded(start),
                    forall|j: int| 0 <= j < iter.index() + 1 && j < start.len()
                        ==> #[trigger] self.dd@[j] == prefix_sum(start, j + 1),
                    forall|j: int| iter.index() + 1 <= j < start.len()
                        ==> #[trigger] self.dd@[j] == start[j],
            ))]
            for i in 1..n {
                #[cfg(verus_keep_ghost)]
                proof! {
                    assert(prefix_sum(start, i as int + 1)
                        == prefix_sum(start, i as int) + start[i as int]);
                }
                self.dd[i] += self.dd[i - 1];
            }
            #[cfg(verus_keep_ghost)]
            proof! { assert(self.dd@ =~= prefix_sums(start)); }
        }
        #[cfg(verus_keep_ghost)]
        proof! {
            #[verifier::proof_note("For any length and i64 second differences whose first and second prefix sums fit i64, build_inplace produces exactly those double prefix sums")]
            assert(self.dd@ == prefix_sums(prefix_sums(initial)));
            assert forall|i: int| 0 <= i < initial.len() implies
                #[trigger] self.dd@[i] == double_prefix_sum(initial, i + 1) by {
                double_prefix_sum_matches(initial, i + 1);
            }
        }
    }
}

#[cfg(verus_keep_ghost)]
verus! {
    // The standalone checks on these recursive sum specifications establish termination.
    pub open spec fn prefix_sum(s: Seq<i64>, end: int) -> int
        recommends 0 <= end <= s.len(),
        decreases end,
    {
        if end <= 0 { 0 } else { prefix_sum(s, end - 1) + s[end - 1] }
    }

    pub open spec fn prefix_sums(s: Seq<i64>) -> Seq<i64> {
        Seq::new(s.len(), |i: int| prefix_sum(s, i + 1) as i64)
    }

    pub open spec fn prefix_sums_bounded(s: Seq<i64>) -> bool {
        forall|i: int| 0 <= i < s.len()
            ==> i64::MIN <= #[trigger] prefix_sum(s, i + 1) <= i64::MAX
    }

    pub open spec fn double_prefix_sum(s: Seq<i64>, end: int) -> int
        recommends 0 <= end <= s.len(),
        decreases end,
    {
        if end <= 0 { 0 } else { double_prefix_sum(s, end - 1) + prefix_sum(s, end) }
    }

    pub open spec fn progression_added(before: Seq<i64>, after: Seq<i64>, l: int, k: int, a: int, d: int) -> bool {
        after.len() == before.len()
            && forall|i: int| 0 <= i < before.len()
                ==> #[trigger] double_prefix_sum(after, i + 1)
                    == double_prefix_sum(before, i + 1)
                        + if l <= i < l + k { a + (i - l) * d } else { 0 }
    }

    pub open spec fn add_at(s: Seq<i64>, index: int, delta: int) -> Seq<i64> {
        if 0 <= index < s.len() { s.update(index, (s[index] + delta) as i64) } else { s }
    }

    pub open spec fn safe_add(s: Seq<i64>, index: int, delta: int) -> bool {
        0 <= index < s.len() ==> i64::MIN <= s[index] + delta <= i64::MAX
    }

    pub open spec fn valid_update(s: Seq<i64>, l: int, k: int, a: int, d: int) -> bool {
        let s1 = add_at(s, l, a);
        let s2 = add_at(s1, l + 1, d - a);
        let s3 = add_at(s2, l + k, -a - k * d);
        &&& 0 <= l && 0 <= k <= i64::MAX && l + k < usize::MAX
        &&& a > i64::MIN
        &&& i64::MIN <= d - a <= i64::MAX
        &&& i64::MIN <= k * d <= i64::MAX
        &&& i64::MIN <= -a - k * d <= i64::MAX
        &&& i64::MIN <= (k - 1) * d <= i64::MAX
        &&& i64::MIN <= a + (k - 1) * d <= i64::MAX
        &&& safe_add(s, l, a)
        &&& safe_add(s1, l + 1, d - a)
        &&& safe_add(s2, l + k, -a - k * d)
        &&& safe_add(s3, l + k + 1, a + (k - 1) * d)
    }

    proof fn prefix_sum_after_add(s: Seq<i64>, index: int, delta: int, end: int)
        requires 0 <= index, 0 <= end <= s.len(), safe_add(s, index, delta),
        ensures prefix_sum(add_at(s, index, delta), end)
            == prefix_sum(s, end) + if index < end { delta } else { 0 },
        decreases end,
    {
        if end > 0 { prefix_sum_after_add(s, index, delta, end - 1); }
        #[verifier::proof_note("Adding a representable delta at one buffer index changes a prefix sum exactly when that index belongs to the prefix")]
        assert(prefix_sum(add_at(s, index, delta), end)
            == prefix_sum(s, end) + if index < end { delta } else { 0 });
    }

    proof fn double_prefix_sum_after_add(s: Seq<i64>, index: int, delta: int, end: int)
        requires 0 <= index, 0 <= end <= s.len(), safe_add(s, index, delta),
        ensures double_prefix_sum(add_at(s, index, delta), end)
            == double_prefix_sum(s, end) + (if index < end { end - index } else { 0 }) * delta,
        decreases end,
    {
        reveal_with_fuel(double_prefix_sum, 2);
        if end > 0 {
            double_prefix_sum_after_add(s, index, delta, end - 1);
            prefix_sum_after_add(s, index, delta, end);
            if index < end - 1 {
                assert((end - 1 - index) * delta + delta == (end - index) * delta)
                    by(nonlinear_arith);
            } else if index == end - 1 {
                assert((end - index) * delta == delta) by(nonlinear_arith)
                    requires index == end - 1;
            }
        }
        #[verifier::proof_note("Adding delta at index j changes a double prefix sum ending at e by (e - j) times delta when j is before e, and otherwise leaves it unchanged")]
        assert(double_prefix_sum(add_at(s, index, delta), end)
            == double_prefix_sum(s, end) + (if index < end { end - index } else { 0 }) * delta);
    }

    proof fn double_prefix_sum_matches(s: Seq<i64>, end: int)
        requires 0 <= end <= s.len(), prefix_sums_bounded(s),
        ensures prefix_sum(prefix_sums(s), end) == double_prefix_sum(s, end),
        decreases end,
    {
        if end > 0 { double_prefix_sum_matches(s, end - 1); }
        #[verifier::proof_note("When first prefix sums fit i64, summing their stored values agrees with the mathematical double prefix sum")]
        assert(prefix_sum(prefix_sums(s), end) == double_prefix_sum(s, end));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{WithEmptySegment, Xorshift, testutil::integer_cases};

    const A: i64 = 1_000_000_000;

    #[test]
    fn test_range_ap_add() {
        let mut rng = Xorshift::default();
        let mut range_rng = Xorshift::default();
        for n in integer_cases(0..=8_192) {
            let mut expected = vec![0i64; n];
            let mut ap = RangeArithmeticProgressionAdd::new(n);
            let boundaries = [
                (0, 0),
                (0, n.min(1)),
                (0, n),
                (n.saturating_sub(1), n),
                (n, n),
            ];
            for (l, r) in boundaries
                .into_iter()
                .chain(range_rng.random_iter(WithEmptySegment(n)).take(n))
            {
                let (a, d) = rng.random((-A..=A, -A..=A));
                ap.update(l, r - l, a, d);
                for (i, value) in expected[l..r].iter_mut().enumerate() {
                    *value += a + i as i64 * d;
                }
            }
            ap.build_inplace();
            assert_eq!(ap.dd, expected, "n={n}");
        }
    }

    #[test]
    fn test_range_ap_add_isize() {
        let mut rng = Xorshift::default();
        let mut range_rng = Xorshift::default();
        for n in integer_cases(0..=4_096) {
            let size = n as isize;
            let boundaries = [
                (-size - 1, -1),
                (-1, size + 1),
                (0, size),
                (-1, 0),
                (size, size),
                (size, size + 1),
            ];
            let ranges = boundaries.into_iter().chain((0..n).map(|_| {
                let l = range_rng.random(-size..=size);
                let r = range_rng.random(l..=2 * size);
                (l, r)
            }));
            for (case, (l, r)) in ranges.enumerate() {
                let (a, magnitude) = rng.random((-A..=A, 1..=A));
                let d = if case % 2 == 0 { magnitude } else { -magnitude };
                let mut ap = RangeArithmeticProgressionAdd::new(n);
                ap.update_isize(l, (r - l) as usize, a, d);
                let expected: Vec<_> = (0..n)
                    .map(|i| {
                        let i = i as isize;
                        if l <= i && i < r {
                            a + (i - l) as i64 * d
                        } else {
                            0
                        }
                    })
                    .collect();
                ap.build_inplace();
                assert_eq!(ap.dd, expected, "n={n}, range={l}..{r}, a={a}, d={d}");
            }
        }
    }
}

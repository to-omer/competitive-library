#[cfg(verus_keep_ghost)]
use super::z_algorithm::lawful_eq;
#[cfg(verus_keep_ghost)]
use vstd::{prelude::*, std_specs::cmp::PartialEqSpec};

#[cfg_attr(verus_keep_ghost, verus_verify)]
// Verus checks the derived Clone body without a postcondition on the cloned value.
#[cfg_attr(verus_keep_ghost, verifier::allow(autoderive_clone_without_spec))]
#[derive(Clone, Debug)]
pub struct KnuthMorrisPratt<T: Eq> {
    pattern: Vec<T>,
    table: Vec<usize>,
}
#[cfg_attr(verus_keep_ghost, verus_verify)]
impl<T: Eq> KnuthMorrisPratt<T> {
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires
            pattern.len() < usize::MAX,
            lawful_eq::<T>(),
        ensures
            result@ == pattern@,
            result.valid(),
    ))]
    pub fn new(pattern: Vec<T>) -> Self {
        let mut table = vec![0; pattern.len() + 1];
        // Each iteration compares at most twice outside fallback steps.
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost mut fallbacks = 0int; }
        #[cfg_attr(verus_keep_ghost, verus_spec(iter =>
            invariant
                pattern.len() < usize::MAX,
                lawful_eq::<T>(),
                table.len() == pattern.len() + 1,
                0 <= fallbacks,
                fallbacks + table@[iter.index()] <= iter.index(),
                forall|q: int| 0 <= q < pattern.len() && q < iter.index() + 1
                    ==> longest_border(pattern@, q + 1, table@[q] as int),
        ))]
        for i in 1..pattern.len() {
            let mut j = table[i - 1];
            #[cfg(verus_keep_ghost)]
            proof! {
                assert forall|k: int| j < k < i implies
                    !#[trigger] prefix_suffix(pattern@, pattern@, i + 1, k + 1) by {
                    extend_border(pattern@, pattern@, i as int, k);
                }
            }
            #[cfg_attr(verus_keep_ghost, verus_spec(
                invariant
                    0 < i < pattern.len() < usize::MAX,
                    lawful_eq::<T>(),
                    table.len() == pattern.len() + 1,
                    forall|q: int| 0 <= q < i
                        ==> longest_border(pattern@, q + 1, table@[q] as int),
                    j < i,
                    0 <= fallbacks, fallbacks + j <= i - 1,
                    prefix_suffix(pattern@, pattern@, i as int, j as int),
                    forall|k: int| j < k < i
                        ==> !#[trigger] prefix_suffix(pattern@, pattern@, i + 1, k + 1),
                decreases j,
            ))]
            while j > 0 && pattern[i] != pattern[j] {
                #[cfg(verus_keep_ghost)]
                proof! {
                    let next = table@[j - 1] as int;
                    nested_borders(pattern@, pattern@, i as int, j as int, next);
                    assert forall|k: int| next < k < i implies
                        !#[trigger] prefix_suffix(pattern@, pattern@, i + 1, k + 1) by {
                        extend_border(pattern@, pattern@, i as int, k);
                        if k < j && prefix_suffix(pattern@, pattern@, i as int, k) {
                            nested_borders(pattern@, pattern@, i as int, j as int, k);
                        }
                    }
                    fallbacks = fallbacks + 1;
                }
                j = table[j - 1];
            }
            #[cfg(verus_keep_ghost)]
            proof! {
                extend_border(pattern@, pattern@, i as int, j as int);
                assert forall|k: int| j < k < i implies
                    !#[trigger] prefix_suffix(pattern@, pattern@, i + 1, k + 1) by {}
            }
            table[i] = j + (pattern[i] == pattern[j]) as usize;
            #[cfg(verus_keep_ghost)]
            proof! {
                assert forall|k: int| table@[i as int] < k < i + 1 implies
                    !#[trigger] prefix_suffix(pattern@, pattern@, i + 1, k) by {
                    extend_border(pattern@, pattern@, i as int, k - 1);
                }
                assert(longest_border(pattern@, i + 1, table@[i as int] as int));
            }
        }
        #[cfg(verus_keep_ghost)]
        proof! {
            #[verifier::proof_note("For any pattern with lawful Eq and length below usize::MAX, each KMP table entry is the longest proper prefix matching the corresponding prefix's suffix")]
            assert(forall|i: int| 0 <= i < pattern.len()
                ==> #[trigger] longest_border(pattern@, i + 1, table@[i] as int));
            #[verifier::proof_note("KMP preprocessing uses at most 3n Eq calls for a pattern of length n, excluding comparison internals and allocation")]
            assert(fallbacks + 2 * pattern.len() <= 3 * pattern.len());
        }
        Self { pattern, table }
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires
            self.valid(),
            #![verifier::proof_note("KMP search requires a nonempty pattern and lawful, stable Eq")]
            self@.len() > 0 && lawful_eq::<T>(),
        ensures
            search_results(self@, s@, s.len() as int, result@),
    ))]
    pub fn search_all(&self, s: &[T]) -> Vec<usize> {
        let mut res = vec![];
        let mut j = 0;
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost mut fallbacks = 0int; }
        #[cfg_attr(verus_keep_ghost, verus_spec(iter =>
            invariant
                self.valid(), self.pattern.len() > 0,
                lawful_eq::<T>(),
                j < self.pattern.len(),
                0 <= fallbacks, fallbacks + j <= iter.index(),
                prefix_suffix(self.pattern@, s@, iter.index(), j as int),
                forall|k: int| j < k < self.pattern.len()
                    ==> !#[trigger] prefix_suffix(self.pattern@, s@, iter.index(), k),
                search_results(self.pattern@, s@, iter.index(), res@),
                res.len() <= iter.index(),
        ))]
        for (i, x) in (0..s.len()).zip(s) {
            #[cfg(verus_keep_ghost)]
            proof_decl! { let ghost before = res@; }
            #[cfg(verus_keep_ghost)]
            proof! {
                assert forall|k: int| j < k < self.pattern.len() implies
                    !#[trigger] prefix_suffix(self.pattern@, s@, i + 1, k + 1) by {
                    if k <= i { extend_border(self.pattern@, s@, i as int, k); }
                }
            }
            #[cfg_attr(verus_keep_ghost, verus_spec(
                invariant
                    self.valid(), self.pattern.len() > 0,
                    lawful_eq::<T>(),
                    i < s.len(), *x == s@[i as int],
                    j < self.pattern.len(),
                    0 <= fallbacks, fallbacks + j <= i,
                    prefix_suffix(self.pattern@, s@, i as int, j as int),
                    forall|k: int| j < k < self.pattern.len()
                        ==> !#[trigger] prefix_suffix(self.pattern@, s@, i + 1, k + 1),
                decreases j,
            ))]
            while j > 0 && x != &self.pattern[j] {
                #[cfg(verus_keep_ghost)]
                proof! {
                    let next = self.table@[j - 1] as int;
                    nested_borders(self.pattern@, s@, i as int, j as int, next);
                    assert forall|k: int| next < k < self.pattern.len() implies
                        !#[trigger] prefix_suffix(self.pattern@, s@, i + 1, k + 1) by {
                        if k <= i { extend_border(self.pattern@, s@, i as int, k); }
                        if k < j && prefix_suffix(self.pattern@, s@, i as int, k) {
                            nested_borders(self.pattern@, s@, i as int, j as int, k);
                        }
                    }
                    fallbacks = fallbacks + 1;
                }
                j = self.table[j - 1];
            }
            #[cfg(verus_keep_ghost)]
            proof! { extend_border(self.pattern@, s@, i as int, j as int); }
            if x == &self.pattern[j] {
                j += 1;
            }
            #[cfg(verus_keep_ghost)]
            proof! {
                assert forall|k: int| j < k <= self.pattern.len() implies
                    !#[trigger] prefix_suffix(self.pattern@, s@, i + 1, k) by {
                    if k <= i + 1 { extend_border(self.pattern@, s@, i as int, k - 1); }
                }
                assert((j == self.pattern.len())
                    == prefix_suffix(self.pattern@, s@, i + 1, self.pattern.len() as int));
            }
            if j == self.pattern.len() {
                res.push(i - (j - 1));
                #[cfg(verus_keep_ghost)]
                proof! {
                    let next = self.table@[j - 1] as int;
                    nested_borders(self.pattern@, s@, i + 1, j as int, next);
                    assert forall|k: int| next < k < self.pattern.len() implies
                        !#[trigger] prefix_suffix(self.pattern@, s@, i + 1, k) by {
                        nested_borders(self.pattern@, s@, i + 1, j as int, k);
                    }
                }
                j = self.table[j - 1];
            }
            #[cfg(verus_keep_ghost)]
            proof! {
                broadcast use vstd::seq_lib::lemma_seq_contains_after_push;
                let p = self.pattern@;
                assert forall|q: int| 0 <= q < res.len() implies {
                    &&& res@[q] + p.len() <= i + 1
                    &&& prefix_suffix(p, s@, res@[q] + p.len(), p.len() as int)
                } by {
                    if q < before.len() { assert(res@[q] == before[q]); }
                }
                assert forall|pos: int| 0 <= pos && pos + p.len() <= i + 1 implies
                    (#[trigger] prefix_suffix(p, s@, pos + p.len(), p.len() as int)
                        == res@.contains(pos as usize)) by {
                    if pos + p.len() <= i {
                        assert(prefix_suffix(p, s@, pos + p.len(), p.len() as int)
                            == before.contains(pos as usize));
                    } else {
                        assert(pos == i + 1 - p.len());
                        if before.contains(pos as usize) {
                            let q = before.lemma_contains_to_index(pos as usize);
                            assert(before[q] + p.len() <= i);
                        }
                    }
                }
                assert forall|q: int, k: int| 0 <= q < k < res.len() implies
                    res@[q] < res@[k] by {
                    if k < before.len() {
                        assert(before[q] < before[k]);
                    } else {
                        assert(before[q] + p.len() <= i);
                    }
                }
            }
        }
        #[cfg(verus_keep_ghost)]
        proof! {
            #[verifier::proof_note("For arbitrary text length, a nonempty pattern, and lawful Eq, KMP returns exactly all matching starting positions in increasing order, including overlapping matches")]
            assert(search_results(self.pattern@, s@, s.len() as int, res@));
            #[verifier::proof_note("KMP search uses at most 3n Eq calls for text of length n, excluding comparison internals and allocation")]
            assert(fallbacks + 2 * s.len() <= 3 * s.len());
        }
        res
    }
}

#[cfg(verus_keep_ghost)]
verus! {
impl<T: Eq> View for KnuthMorrisPratt<T> {
    type V = Seq<T>;

    closed spec fn view(&self) -> Seq<T> { self.pattern@ }
}

impl<T: Eq> KnuthMorrisPratt<T> {
    pub closed spec fn valid(&self) -> bool {
        self.table.len() == self.pattern.len() + 1
        && forall|i: int| 0 <= i < self.pattern.len()
            ==> longest_border(self.pattern@, i + 1, self.table@[i] as int)
    }
}

pub open spec fn prefix_suffix<T>(p: Seq<T>, s: Seq<T>, n: int, k: int) -> bool
where
    T: Eq,
{
    &&& 0 <= k <= p.len()
    &&& k <= n <= s.len()
    &&& forall|t: int| 0 <= t < k ==> #[trigger] p[t].eq_spec(&s[n - k + t])
}

pub open spec fn longest_border<T>(p: Seq<T>, n: int, k: int) -> bool
where
    T: Eq,
{
    &&& 0 <= k < n
    &&& prefix_suffix(p, p, n, k)
    &&& forall|j: int| k < j < n ==> !prefix_suffix(p, p, n, j)
}

pub open spec fn search_results<T>(p: Seq<T>, s: Seq<T>, n: int, r: Seq<usize>) -> bool
where
    T: Eq,
{
    &&& forall|q: int| 0 <= q < r.len() ==> {
        &&& r[q] + p.len() <= n
        &&& prefix_suffix(p, s, r[q] + p.len(), p.len() as int)
    }
    &&& forall|pos: int| 0 <= pos && pos + p.len() <= n
        ==> (#[trigger] prefix_suffix(p, s, pos + p.len(), p.len() as int)
            == r.contains(pos as usize))
    &&& forall|q: int, k: int| 0 <= q < k < r.len() ==> r[q] < r[k]
}

proof fn extend_border<T>(p: Seq<T>, s: Seq<T>, n: int, k: int)
    where T: Eq,
    requires 0 <= k <= n < s.len(), k < p.len(),
    ensures prefix_suffix(p, s, n + 1, k + 1)
        == (prefix_suffix(p, s, n, k) && p[k].eq_spec(&s[n])),
{
    if prefix_suffix(p, s, n + 1, k + 1) {
        assert(p[k].eq_spec(&s[n + 1 - (k + 1) + k]));
        assert forall|t: int| 0 <= t < k implies
            #[trigger] p[t].eq_spec(&s[n - k + t]) by {
            assert(p[t].eq_spec(&s[n + 1 - (k + 1) + t]));
        }
    }
    if prefix_suffix(p, s, n, k) && p[k].eq_spec(&s[n]) {
        assert forall|t: int| 0 <= t < k + 1 implies
            #[trigger] p[t].eq_spec(&s[n - k + t]) by {}
    }
    #[verifier::proof_note("A prefix match extends by one exactly when the next pattern and text elements compare equal")]
    assert(prefix_suffix(p, s, n + 1, k + 1)
        == (prefix_suffix(p, s, n, k) && p[k].eq_spec(&s[n])));
}

proof fn nested_borders<T>(p: Seq<T>, s: Seq<T>, n: int, j: int, k: int)
    where T: Eq,
    requires lawful_eq::<T>(), prefix_suffix(p, s, n, j), 0 <= k < j,
    ensures prefix_suffix(p, s, n, k) == prefix_suffix(p, p, j, k),
{
    assert forall|t: int| 0 <= t < k implies
        #[trigger] p[t].eq_spec(&s[n - k + t]) == p[t].eq_spec(&p[j - k + t]) by {
        assert(p[j - k + t].eq_spec(&s[n - j + (j - k + t)]));
        assert(p[j - k + t].eq_spec(&s[n - k + t]));
    }
    if prefix_suffix(p, s, n, k) {
        assert forall|t: int| 0 <= t < k implies
            #[trigger] p[t].eq_spec(&p[j - k + t]) by {
            assert(p[t].eq_spec(&s[n - k + t]));
        }
    } else if prefix_suffix(p, p, j, k) {
        assert forall|t: int| 0 <= t < k implies
            #[trigger] p[t].eq_spec(&s[n - k + t]) by {
            assert(p[t].eq_spec(&p[j - k + t]));
        }
    }
    #[verifier::proof_note("Inside a matched suffix of length j, a shorter length k matches exactly when it is a border of the pattern prefix of length j")]
    assert(prefix_suffix(p, s, n, k) == prefix_suffix(p, p, j, k));
}
}

#[cfg(verus_keep_ghost)]
mod verification {
    use super::*;

    verus! {
    pub fn bytes(pattern: Vec<u8>, text: &[u8]) -> (result: Vec<usize>)
        requires 0 < pattern.len() < usize::MAX,
        ensures search_results(pattern@, text@, text.len() as int, result@),
    {
        let result = KnuthMorrisPratt::new(pattern).search_all(text);
        #[verifier::proof_note("Constructing KMP from any nonempty byte pattern establishes the search preconditions and returns exactly all matches in any byte text")]
        assert(search_results(pattern@, text@, text.len() as int, result@));
        result
    }
    }
}

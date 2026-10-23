#[cfg(verus_keep_ghost)]
use vstd::{prelude::*, std_specs::cmp::PartialEqSpec};

#[cfg_attr(verus_keep_ghost, verus_verify)]
// Verus checks the derived Clone body without a postcondition on the cloned value.
#[cfg_attr(verus_keep_ghost, verifier::allow(autoderive_clone_without_spec))]
#[derive(Clone, Debug)]
pub struct Zarray {
    z: Vec<usize>,
}
#[cfg_attr(verus_keep_ghost, verus_verify)]
impl Zarray {
    // Called operations must terminate. The comparison bound counts Eq calls,
    // excluding their internal work and allocation; loop variants prove loop termination.
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires
            #![verifier::proof_note("Zarray::new is called with nonempty input")]
            0 < s.len(),
            #![verifier::proof_note("Zarray::new is called with a lawful, stable Eq matching its specification")]
            lawful_eq::<T>(),
        ensures
            result@.len() == s.len(),
            forall|i: int| 0 <= i < s.len() ==> z_value(s@, i, result@[i] as int),
    ))]
    pub fn new<T>(s: &[T]) -> Self
    where
        T: Eq,
    {
        let n = s.len();
        let mut z = vec![0; n];
        z[0] = n;
        let (mut i, mut j) = (1, 0);
        #[cfg(verus_keep_ghost)]
        proof_decl! {
            let ghost mut successes = 0int;
            let ghost mut failures = 0int;
        }
        #[cfg_attr(verus_keep_ghost, verus_spec(
            invariant
                n == s.len(), 0 < n,
                lawful_eq::<T>(),
                z.len() == n,
                1 <= i <= n,
                matches(s@, i as int, j as int),
                forall|p: int| 0 <= p < i ==> z_value(s@, p, z@[p] as int),
                0 <= successes <= i + j - 1,
                0 <= failures <= i - 1,
            decreases n - i,
        ))]
        while i < n {
            #[cfg_attr(verus_keep_ghost, verus_spec(
                invariant
                    n == s.len(), 0 < i < n,
                    lawful_eq::<T>(),
                    matches(s@, i as int, j as int),
                    0 <= successes <= i + j - 1,
                decreases n - i - j,
            ))]
            while i + j < n && s[j] == s[i + j] {
                #[cfg(verus_keep_ghost)]
                proof! { successes = successes + 1; }
                j += 1;
            }
            #[cfg(verus_keep_ghost)]
            proof! { if i + j < n { failures = failures + 1; } }
            z[i] = j;
            if j == 0 {
                i += 1;
                continue;
            }
            let mut k = 1;
            #[cfg_attr(verus_keep_ghost, verus_spec(
                invariant
                    n == s.len(), 0 < i < n,
                    lawful_eq::<T>(),
                    z.len() == n,
                    1 <= k <= j,
                    z_value(s@, i as int, j as int),
                    forall|p: int| 0 <= p < i + k ==> z_value(s@, p, z@[p] as int),
                decreases j - k,
            ))]
            while i + k < n && k + z[k] < j {
                #[cfg(verus_keep_ghost)]
                proof! { copy_match(s@, i as int, j as int, k as int, z@[k as int] as int); }
                z[i + k] = z[k];
                k += 1;
            }
            #[cfg(verus_keep_ghost)]
            proof! {
                if i + k < n {
                    carry_match(s@, i as int, j as int, k as int, z@[k as int] as int);
                }
            }
            i += k;
            j -= k;
        }
        #[cfg(verus_keep_ghost)]
        proof! {
            #[verifier::proof_note("The Z-array has the same length as the input")]
            assert(z.len() == s.len());
            #[verifier::proof_note("For any nonempty input s with lawful Eq, z[i] is the longest common prefix length of s and s[i..]")]
            assert(forall|p: int| 0 <= p < s.len() ==> z_value(s@, p, z@[p] as int));
            #[verifier::proof_note("At most 2n - 2 Eq calls for any nonempty input of length n")]
            assert(successes + failures <= 2 * n - 2);
        }
        Self { z }
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires
            pat.len() > 0,
            pat.len() + s.len() < usize::MAX,
            lawful_eq::<T>(),
        ensures
            forall|q: int| 0 <= q < result.len() ==> occurs(s@, pat@, result@[q] as int),
            forall|p: int| 0 <= p < s.len()
                ==> (occurs(s@, pat@, p) == result@.contains(p as usize)),
            forall|q: int, r: int| 0 <= q < r < result.len() ==> result@[q] < result@[r],
    ))]
    pub fn search<T: Eq>(s: &[T], pat: &[T], sep: T) -> Vec<usize> {
        let mut res: Vec<usize> = vec![];
        let mut t: Vec<&T> = Vec::with_capacity(pat.len() + 1 + s.len());
        #[cfg_attr(verus_keep_ghost, verus_spec(iter =>
            invariant
                t.len() == iter.index(),
                forall|j: int| 0 <= j < iter.index() ==> *t@[j] == pat@[j],
        ))]
        for x in pat {
            t.push(x);
        }
        t.push(&sep);
        #[cfg_attr(verus_keep_ghost, verus_spec(iter =>
            invariant
                t.len() == pat.len() + 1 + iter.index(),
                forall|j: int| 0 <= j < pat.len() ==> *t@[j] == pat@[j],
                forall|j: int| 0 <= j < iter.index() ==> *t@[pat.len() + 1 + j] == s@[j],
        ))]
        for x in s {
            t.push(x);
        }
        #[cfg(verus_keep_ghost)]
        proof! { assert(lawful_eq::<&T>()); }
        let zarray = Self::new(&t);
        #[cfg_attr(verus_keep_ghost, verus_spec(iter =>
            invariant
                pat.len() > 0, pat.len() + s.len() < usize::MAX,
                lawful_eq::<T>(), lawful_eq::<&T>(),
                t.len() == pat.len() + 1 + s.len(),
                forall|j: int| 0 <= j < pat.len() ==> *t@[j] == pat@[j],
                forall|j: int| 0 <= j < s.len() ==> *t@[pat.len() + 1 + j] == s@[j],
                zarray@.len() == t.len(),
                forall|j: int| 0 <= j < t.len() ==> z_value(t@, j, zarray@[j] as int),
                res.len() <= iter.index(),
                forall|q: int| 0 <= q < res.len()
                    ==> res@[q] < iter.index() && occurs(s@, pat@, res@[q] as int),
                forall|p: int| 0 <= p < iter.index()
                    ==> (occurs(s@, pat@, p) == res@.contains(p as usize)),
                forall|q: int, r: int| 0 <= q < r < res.len() ==> res@[q] < res@[r],
        ))]
        for (i, &z) in (pat.len() + 1..t.len()).zip(&zarray.z[pat.len() + 1..]) {
            #[cfg(verus_keep_ghost)]
            proof_decl! { let ghost before = res@; }
            #[cfg(verus_keep_ghost)]
            proof! {
                let pos = i as int - pat.len() - 1;
                let z = zarray@[i as int] as int;
                assert(z_value(t@, i as int, z));
                if z >= pat.len() {
                    assert forall|j: int| 0 <= j < pat.len() implies
                        #[trigger] pat@[j].eq_spec(&s@[pos + j]) by {
                        assert(<&T as PartialEqSpec>::eq_spec(&t@[j], &t@[i + j]));
                    }
                } else if occurs(s@, pat@, pos) {
                    assert(pat@[z].eq_spec(&s@[pos + z]));
                    assert(<&T as PartialEqSpec>::eq_spec(&t@[z], &t@[i + z]));
                }
                assert((z >= pat.len()) == occurs(s@, pat@, pos));
            }
            if z >= pat.len() {
                res.push(i - pat.len() - 1);
            }
            #[cfg(verus_keep_ghost)]
            proof! {
                broadcast use vstd::seq_lib::lemma_seq_contains_after_push;
                let pos = i as int - pat.len() - 1;
                assert forall|p: int| 0 <= p <= pos implies
                    (occurs(s@, pat@, p) == res@.contains(p as usize)) by {
                    if p < pos {
                        assert(occurs(s@, pat@, p) == before.contains(p as usize));
                    } else if before.contains(p as usize) {
                        let q = before.lemma_contains_to_index(p as usize);
                        assert(before[q] < pos);
                    }
                }
                assert forall|q: int| 0 <= q < res.len() implies
                    res@[q] <= pos && occurs(s@, pat@, res@[q] as int) by {
                    if q < before.len() { assert(res@[q] == before[q]); }
                }
                assert forall|q: int, r: int| 0 <= q < r < res.len() implies
                    res@[q] < res@[r] by {
                    if r < before.len() { assert(before[q] < before[r]); }
                }
            }
        }
        #[cfg(verus_keep_ghost)]
        proof! {
            #[verifier::proof_note("For arbitrary text and nonempty pattern lengths whose combined allocation fits usize, lawful Eq, and any separator, Z search returns exactly all matching positions in increasing order, including overlaps and separator collisions")]
            assert((forall|q: int| 0 <= q < res.len() ==> occurs(s@, pat@, res@[q] as int))
                && (forall|p: int| 0 <= p < s.len() ==> (occurs(s@, pat@, p) == res@.contains(p as usize)))
                && (forall|q: int, r: int| 0 <= q < r < res.len() ==> res@[q] < res@[r]));
        }
        res
    }
}

impl std::ops::Index<usize> for Zarray {
    type Output = usize;
    fn index(&self, index: usize) -> &usize {
        &self.z[index]
    }
}

#[cfg(verus_keep_ghost)]
verus! {
impl View for Zarray {
    type V = Seq<usize>;

    closed spec fn view(&self) -> Seq<usize> { self.z@ }
}

pub open spec fn lawful_eq<T>() -> bool
where
    T: Eq,
{
    &&& T::obeys_eq_spec()
    &&& forall|x: T| #[trigger] x.eq_spec(&x)
    &&& forall|x: T, y: T| #[trigger] x.eq_spec(&y) == y.eq_spec(&x)
    &&& forall|x: T, y: T, z: T|
        #[trigger] x.eq_spec(&y) && #[trigger] y.eq_spec(&z) ==> x.eq_spec(&z)
}

pub open spec fn matches<T>(s: Seq<T>, i: int, n: int) -> bool
where
    T: Eq,
{
    &&& 0 <= i <= s.len()
    &&& 0 <= n <= s.len() - i
    &&& forall|k: int| 0 <= k < n ==> #[trigger] s[k].eq_spec(&s[i + k])
}

pub open spec fn z_value<T>(s: Seq<T>, i: int, n: int) -> bool
where
    T: Eq,
{
    matches(s, i, n) && (i + n == s.len() || !s[n].eq_spec(&s[i + n]))
}

pub open spec fn occurs<T>(s: Seq<T>, pat: Seq<T>, pos: int) -> bool
where
    T: Eq,
{
    &&& 0 <= pos && pos + pat.len() <= s.len()
    &&& forall|j: int| 0 <= j < pat.len() ==> #[trigger] pat[j].eq_spec(&s[pos + j])
}

proof fn copy_match<T>(s: Seq<T>, i: int, n: int, k: int, m: int)
    where T: Eq,
    requires lawful_eq::<T>(), z_value(s, i, n), z_value(s, k, m), 0 < k, k + m < n,
    ensures z_value(s, i + k, m),
{
    assert forall|t: int| 0 <= t < m implies
        #[verifier::proof_note("The m matching prefix elements transfer to the shifted position i + k")]
        #[trigger] s[t].eq_spec(&s[i + k + t]) by {
        assert(s[t].eq_spec(&s[k + t]));
        assert(s[k + t].eq_spec(&s[i + (k + t)]));
    }
    #[verifier::proof_note("The known Z-box also matches at the copied prefix's mismatch boundary")]
    assert(s[k + m].eq_spec(&s[i + (k + m)]));
}

proof fn carry_match<T>(s: Seq<T>, i: int, n: int, k: int, m: int)
    where T: Eq,
    requires lawful_eq::<T>(), matches(s, i, n), matches(s, k, m), 0 < k <= n, n <= k + m,
    ensures matches(s, i + k, n - k),
{
    assert forall|t: int| 0 <= t < n - k implies
        #[verifier::proof_note("Shifting an overlapping prefix match by k preserves n - k matching elements")]
        #[trigger] s[t].eq_spec(&s[i + k + t]) by {
        assert(s[t].eq_spec(&s[k + t]));
        assert(s[k + t].eq_spec(&s[i + (k + t)]));
    }
}
}

#[cfg(verus_keep_ghost)]
mod verification {
    use super::*;

    verus! {
    use vstd::std_specs::cmp::PartialEqSpecImpl;

    pub struct Classified {
        class: u8,
        payload: u64,
    }

    impl PartialEq for Classified {
        // The inherited PartialEq contract checks agreement with eq_spec below.
        fn eq(&self, other: &Self) -> bool { self.class == other.class }
    }
    impl Eq for Classified {}

    impl PartialEqSpecImpl for Classified {
        open spec fn obeys_eq_spec() -> bool { true }
        closed spec fn eq_spec(&self, other: &Self) -> bool { self.class == other.class }
    }

    pub fn bytes(s: &[u8]) -> Zarray
        requires s.len() > 0,
    {
        Zarray::new(s)
    }

    pub fn byte_search(text: &[u8], pattern: &[u8], separator: u8)
        requires pattern.len() > 0, pattern.len() + text.len() < usize::MAX,
    {
        let positions = Zarray::search(text, pattern, separator);
        #[verifier::proof_note("For byte texts and nonempty patterns fitting the combined buffer, Z search's contract is usable for any separator byte")]
        assert(forall|p: int| 0 <= p < text.len()
            ==> (occurs(text@, pattern@, p) == positions@.contains(p as usize)));
    }

    pub fn classified(s: &[Classified]) -> Zarray
        requires s.len() > 0,
    {
        #[verifier::proof_note("Class-based equality satisfies the Eq specification and equivalence laws")]
        assert(lawful_eq::<Classified>());
        Zarray::new(s)
    }

    pub fn distinct_payloads() {
        let a = Classified { class: 3, payload: 0 };
        let b = Classified { class: 3, payload: 42 };
        #[verifier::proof_note("The two example values are structurally distinct")]
        assert(a != b);
        let same = a == b;
        #[verifier::proof_note("The two example values compare equal via Eq despite different payloads")]
        assert(same);
    }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{
        Xorshift,
        testutil::{exhaustive_sequences, sequence_cases},
    };
    use std::cell::Cell;

    #[test]
    fn test_zarray() {
        struct Classified<'a> {
            class: u8,
            _payload: usize,
            comparisons: &'a Cell<usize>,
        }
        impl PartialEq for Classified<'_> {
            fn eq(&self, other: &Self) -> bool {
                self.comparisons.set(self.comparisons.get() + 1);
                self.class == other.class
            }
        }
        impl Eq for Classified<'_> {}

        let mut rng = Xorshift::default();
        for s in
            exhaustive_sequences(0..3, 1..=9).chain(sequence_cases(&mut rng, 0..=u8::MAX, 1..=1024))
        {
            let comparisons = Cell::new(0);
            let classified: Vec<_> = s
                .iter()
                .enumerate()
                .map(|(i, &class)| Classified {
                    class,
                    _payload: i,
                    comparisons: &comparisons,
                })
                .collect();
            let z = Zarray::new(&classified);
            assert!(comparisons.get() <= 2 * s.len() - 2);
            assert_eq!(z.z.len(), s.len());
            for i in 0..s.len() {
                let expected = s.iter().zip(&s[i..]).take_while(|(a, b)| a == b).count();
                assert_eq!(z[i], expected, "s={s:?}, i={i}");
            }
        }
    }
}

#[cfg(verus_keep_ghost)]
use vstd::{prelude::*, std_specs::cmp::PartialEqSpec};

#[cfg_attr(verus_keep_ghost, verus_verify)]
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
    pub fn new<T: Eq>(s: &[T]) -> Self {
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
    #[cfg_attr(verus_keep_ghost, verifier::external)]
    pub fn search<T: Eq>(s: &[T], pat: &[T], sep: T) -> Vec<usize> {
        let mut res = vec![];
        let mut t = vec![];
        t.extend(pat);
        t.push(&sep);
        t.extend(s);
        let zarray = Self::new(&t);
        for i in pat.len() + 1..t.len() {
            if zarray[i] >= pat.len() {
                res.push(i - pat.len() - 1);
            }
        }
        res
    }
}

#[cfg_attr(verus_keep_ghost, verus_verify)]
impl std::ops::Index<usize> for Zarray {
    type Output = usize;
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        ensures *result == self@[index as int],
    ))]
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

impl vstd::std_specs::core::IndexSpecImpl<usize> for Zarray {
    open spec fn index_req(&self, index: &usize) -> bool { *index < self@.len() }
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

    pub fn indexed_bytes(s: &[u8], index: usize) -> (result: usize)
        requires s.len() > 0, index < s.len(),
        ensures z_value(s@, index as int, result as int),
    {
        let array = Zarray::new(s);
        let result = array[index];
        #[verifier::proof_note("For every nonempty byte sequence and every valid index, constructing and indexing a Z-array returns the exact longest common prefix with that suffix")]
        assert(z_value(s@, index as int, result as int));
        result
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

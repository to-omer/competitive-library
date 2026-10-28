#[cfg(verus_keep_ghost)]
use vstd::{
    prelude::*,
    std_specs::cmp::{OrdSpec, PartialEqSpec, PartialOrdSpec},
};

#[cfg_attr(verus_keep_ghost, verus_verify)]
#[cfg_attr(verus_keep_ghost, verifier::allow(autoderive_clone_without_spec))]
#[derive(Debug, Default, Clone, Copy)]
pub struct PartialIgnoredOrd<T, U>(pub T, pub U);

#[cfg_attr(verus_keep_ghost, verus_verify)]
impl<T: Eq, U> Eq for PartialIgnoredOrd<T, U> {}
#[cfg_attr(verus_keep_ghost, verus_verify)]
impl<T, U> PartialEq for PartialIgnoredOrd<T, U>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
#[cfg_attr(verus_keep_ghost, verus_verify)]
impl<T, U> PartialOrd for PartialIgnoredOrd<T, U>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
#[cfg_attr(verus_keep_ghost, verus_verify)]
impl<T, U> Ord for PartialIgnoredOrd<T, U>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

#[cfg(verus_keep_ghost)]
verus! {
    impl<T: PartialEq, U> vstd::std_specs::cmp::PartialEqSpecImpl for PartialIgnoredOrd<T, U> {
        open spec fn obeys_eq_spec() -> bool { T::obeys_eq_spec() }
        open spec fn eq_spec(&self, other: &Self) -> bool { self.0.eq_spec(&other.0) }
    }

    impl<T: PartialOrd, U> vstd::std_specs::cmp::PartialOrdSpecImpl for PartialIgnoredOrd<T, U> {
        open spec fn obeys_partial_cmp_spec() -> bool { T::obeys_partial_cmp_spec() }
        open spec fn partial_cmp_spec(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.0.partial_cmp_spec(&other.0)
        }
    }

    impl<T: Ord, U> vstd::std_specs::cmp::OrdSpecImpl for PartialIgnoredOrd<T, U> {
        open spec fn obeys_cmp_spec() -> bool { T::obeys_cmp_spec() }
        open spec fn cmp_spec(&self, other: &Self) -> std::cmp::Ordering {
            self.0.cmp_spec(&other.0)
        }
    }
}

#[cfg(verus_keep_ghost)]
mod verification {
    use super::*;

    verus! {
        fn payload_is_ignored<U>(a: i64, b: i64, left: U, right: U) {
            let x = PartialIgnoredOrd(a, left);
            let y = PartialIgnoredOrd(b, right);
            let equal = x == y;
            let compared = x.cmp(&y);
            let partial = x.partial_cmp(&y);
            proof {
                #[verifier::proof_note("PartialIgnoredOrd equality and both ordering operations depend only on the key, for arbitrary payload types and values, without requiring the payload to implement any comparison trait")]
                assert(equal == (a == b) && compared == a.cmp_spec(&b) && partial == Some(compared));
            }
        }
    }
}

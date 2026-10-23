#[derive(Debug, Clone)]
pub struct SparseSet<Usize, const FIXED: bool = true> {
    data: Vec<Usize>,
    index: Vec<Usize>,
}

macro_rules! impl_sparse_set {
    ($($ty:ty)*) => {
        $(
            impl<const FIXED: bool> SparseSet<$ty, FIXED> {
                pub fn new(n: usize) -> Self {
                    Self {
                        data: Vec::with_capacity(n),
                        index: vec![!0; n],
                    }
                }

                pub unsafe fn insert_unchecked(&mut self, x: $ty) {
                    if !FIXED && x as usize >= self.index.len() {
                        self.index.resize(x as usize + 1, !0);
                    }
                    self.index[x as usize] = self.data.len() as _;
                    self.data.push(x);
                }

                pub fn insert(&mut self, x: $ty) -> bool {
                    if self.contains(x) {
                        return false;
                    }
                    unsafe { self.insert_unchecked(x) };
                    true
                }

                pub unsafe fn remove_unchecked(&mut self, x: $ty) {
                    let n = self.data.len();
                    let k = std::mem::replace(&mut self.index[x as usize], !0);
                    if k as usize != n - 1 {
                        self.data.swap(k as usize, n - 1);
                        self.index[self.data[k as usize] as usize] = k;
                    }
                    self.data.pop();
                }

                pub fn remove(&mut self, x: $ty) -> bool {
                    if !self.contains(x) {
                        return false;
                    }
                    unsafe { self.remove_unchecked(x) };
                    true
                }

                pub fn len(&self) -> usize {
                    self.data.len()
                }

                pub fn is_empty(&self) -> bool {
                    self.len() == 0
                }

                pub fn contains(&self, x: $ty) -> bool {
                    if !FIXED && x as usize >= self.index.len() {
                        false
                    } else {
                        // The last position of a full key domain shares the absent marker.
                        self.index[x as usize] != !0
                            || self.data.get(<$ty>::MAX as usize) == Some(&x)
                    }
                }

                pub fn iter(&self) -> std::slice::Iter<'_, $ty> {
                    self.data.iter()
                }
            }
        )*
    };
}
impl_sparse_set!(u8 u16 u32 u64 usize);

#[cfg(kani)]
mod verification {
    use super::*;

    #[kani::proof]
    #[kani::unwind(6)]
    fn proof_fixed_set() {
        let order: [u8; 4] = kani::any();
        let len: usize = kani::any();
        let key: u8 = kani::any();
        let insert: bool = kani::any();
        kani::assume(len <= order.len() && key < 4);
        let mut set = SparseSet::<u8>::new(4);
        let mut expected = [false; 4];
        for &value in &order[..len] {
            kani::assume(value < 4 && !expected[value as usize]);
            set.insert(value);
            expected[value as usize] = true;
        }
        let present = expected[key as usize];
        if insert {
            kani::assert(
                set.insert(key) != present,
                "For every ordered set of keys 0 through 3, fixed SparseSet<u8> insertion reports whether the key was absent",
            );
            expected[key as usize] = true;
        } else {
            kani::assert(
                set.remove(key) == present,
                "For every ordered set of keys 0 through 3, fixed SparseSet<u8> removal reports whether the key was present",
            );
            expected[key as usize] = false;
        }
        let mut actual = [false; 4];
        for &value in set.iter() {
            kani::assert(
                value < 4 && !actual[value as usize],
                "After insertion or removal in the four-key domain, sparse-set iteration contains valid keys without duplicates",
            );
            actual[value as usize] = true;
        }
        kani::assert(
            actual == expected && set.len() == expected.iter().filter(|&&x| x).count(),
            "After insertion or removal in the four-key domain, sparse-set iteration and length equal the mathematical set",
        );
        for value in 0..4 {
            kani::assert(
                set.contains(value as u8) == expected[value],
                "After insertion or removal in the four-key domain, every sparse-set membership query agrees with the mathematical set",
            );
        }
        kani::cover!(len == 4 && !insert);
        kani::cover!(len == 0 && insert && key == 3);
    }

    #[kani::proof]
    #[kani::unwind(6)]
    fn proof_growing_set() {
        let a: u8 = kani::any();
        let b: u8 = kani::any();
        let query: u8 = kani::any();
        kani::assume(a < 4 && b < 4 && query < 4);
        let mut set = SparseSet::<u8, false>::new(0);
        kani::assert(
            set.insert(a),
            "A growing SparseSet<u8> accepts every first key from 0 through 3 starting at zero capacity",
        );
        kani::assert(
            set.contains(query) == (query == a),
            "After one insertion into a growing sparse set, holes and keys beyond the allocated index are absent",
        );
        kani::assert(
            set.insert(b) == (a != b) && set.len() == 1 + usize::from(a != b),
            "For all two-key insertion sequences on keys 0 through 3, a growing sparse set suppresses duplicates and tracks the exact size",
        );
        kani::assert(
            set.contains(query) == (query == a || query == b),
            "After every two-key insertion sequence on keys 0 through 3, growing sparse-set membership is exactly the inserted set",
        );
        set.remove(a);
        kani::assert(
            set.contains(query) == (a != b && query == b) && set.len() == usize::from(a != b),
            "Removing the first key after two insertions preserves exactly the distinct second key in a growing sparse set",
        );
        kani::cover!(a == 0 && b == 3);
        kani::cover!(a == 3 && b == 0);
        kani::cover!(a == b);
    }

    #[kani::proof]
    #[kani::unwind(258)]
    fn proof_full_key_domain() {
        let key: u8 = kani::any();
        let query: u8 = kani::any();
        // The valid full state reached by inserting keys in ascending order.
        let mut set = SparseSet::<u8> {
            data: (0..=u8::MAX).collect(),
            index: (0..=u8::MAX).collect(),
        };
        kani::assert(
            set.contains(key) && !set.insert(key) && set.len() == 256,
            "In the full u8 set with ascending dense order, every key is present and reinsertion is rejected, including the index that equals the absent marker",
        );
        kani::assert(
            set.remove(key) && set.len() == 255 && set.contains(query) == (query != key),
            "Removing any key from the full u8 domain preserves every other key and removes exactly one element",
        );
        kani::assert(
            set.insert(key) && set.len() == 256 && set.contains(query),
            "Reinserting the removed key restores membership of every u8 key even when the last dense position uses the absent marker",
        );
        kani::cover!(key == u8::MAX);
        kani::cover!(key == 0 && query == u8::MAX);
    }
}

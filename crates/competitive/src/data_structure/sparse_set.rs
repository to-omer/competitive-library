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
                        self.index.resize_with(x as usize + 1, Default::default);
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
                    self.index[x as usize] != !0
                }

                pub fn iter(&self) -> std::slice::Iter<'_, $ty> {
                    self.data.iter()
                }
            }
        )*
    };
}
impl_sparse_set!(u8 u16 u32 u64 usize);

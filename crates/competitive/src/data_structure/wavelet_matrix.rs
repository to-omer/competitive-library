use super::{BitVector, RankSelectDictionaries};

#[snippet::entry(include = "RankSelectDictionaries")]
pub struct WaveletMatrix {
    len: usize,
    table: Vec<(usize, BitVector)>,
}
#[snippet::entry("WaveletMatrix")]
impl WaveletMatrix {
    pub fn new<T: Clone + RankSelectDictionaries>(mut v: Vec<T>, bit_length: usize) -> Self {
        let len = v.len();
        let mut table = Vec::new();
        for d in (0..bit_length).rev() {
            let b: BitVector = v.iter().map(|x| x.access(d)).collect();
            table.push((b.rank0(len), b));
            v = v
                .iter()
                .filter(|&x| !x.access(d))
                .chain(v.iter().filter(|&x| x.access(d)))
                .cloned()
                .collect();
        }
        Self { len, table }
    }
    /// get k-th value
    pub fn access(&self, mut k: usize) -> usize {
        let mut val = 0;
        for (d, &(c, ref b)) in self.table.iter().rev().enumerate().rev() {
            if b.access(k) {
                k = c + b.rank1(k);
                val |= 1 << d;
            } else {
                k = b.rank0(k);
            }
        }
        val
    }
    /// the number of val in range
    pub fn rank(&self, val: usize, mut range: std::ops::Range<usize>) -> usize {
        for (d, &(c, ref b)) in self.table.iter().rev().enumerate().rev() {
            if val.access(d) {
                range.start = c + b.rank1(range.start);
                range.end = c + b.rank1(range.end);
            } else {
                range.start = b.rank0(range.start);
                range.end = b.rank0(range.end);
            }
        }
        range.end - range.start
    }
    /// index of k-th val
    pub fn select(&self, val: usize, k: usize) -> Option<usize> {
        if self.rank(val, 0..self.len) <= k {
            return None;
        }
        let mut i = 0;
        for (d, &(c, ref b)) in self.table.iter().rev().enumerate().rev() {
            if val.access(d) {
                i = c + b.rank1(i);
            } else {
                i = b.rank0(i);
            }
        }
        i += k;
        for &(c, ref b) in self.table.iter().rev() {
            if i >= c {
                i = b.select1(i - c).unwrap();
            } else {
                i = b.select0(i).unwrap();
            }
        }
        Some(i)
    }
    /// get k-th smallest value in range
    pub fn quantile(&self, mut range: std::ops::Range<usize>, mut k: usize) -> usize {
        let mut val = 0;
        for (d, &(c, ref b)) in self.table.iter().rev().enumerate().rev() {
            let z = b.rank0(range.end) - b.rank0(range.start);
            if z <= k {
                k -= z;
                val |= 1 << d;
                range.start = c + b.rank1(range.start);
                range.end = c + b.rank1(range.end);
            } else {
                range.start = b.rank0(range.start);
                range.end = b.rank0(range.end);
            }
        }
        val
    }
    /// the number of value less than val in range
    pub fn rank_lessthan(&self, val: usize, mut range: std::ops::Range<usize>) -> usize {
        let mut res = 0;
        for (d, &(c, ref b)) in self.table.iter().rev().enumerate().rev() {
            if val.access(d) {
                res += b.rank0(range.end) - b.rank0(range.start);
                range.start = c + b.rank1(range.start);
                range.end = c + b.rank1(range.end);
            } else {
                range.start = b.rank0(range.start);
                range.end = b.rank0(range.end);
            }
        }
        res
    }
    /// the number of valrange in range
    pub fn rank_range(
        &self,
        valrange: std::ops::Range<usize>,
        range: std::ops::Range<usize>,
    ) -> usize {
        self.rank_lessthan(valrange.end, range.clone()) - self.rank_lessthan(valrange.start, range)
    }
}

#[test]
fn test_wavelet_matrix() {
    use crate::tools::Xorshift;
    const N: usize = 1_000;
    const Q: usize = 1_000;
    const A: usize = 1 << 8;
    let mut rand = Xorshift::time();
    let v: Vec<_> = (0..N).map(|_| rand.rand(A as u64) as usize).collect();
    let wm = WaveletMatrix::new(v.clone(), 8);
    for (i, v) in v.iter().cloned().enumerate() {
        assert_eq!(wm.access(i), v);
    }
    for _ in 0..Q {
        let l = rand.rand(N as u64) as usize;
        let r = rand.rand(N as u64) as usize;
        let (l, r) = if l < r { (l, r) } else { (r, l) };
        let a = rand.rand(A as u64) as usize;
        assert_eq!(
            wm.rank(a, l..r),
            v[l..r].iter().filter(|&&x| x == a).count()
        );

        let mut a = rand.rand(A as u64) as usize;
        while wm.rank(a, 0..N) == 0 {
            a = rand.rand(A as u64) as usize;
        }
        let k = rand.rand(wm.rank(a, 0..N) as u64) as usize;
        assert_eq!(
            wm.select(a, k).unwrap().min(N),
            (0..N)
                .position(|i| wm.rank(a, 0..i + 1) == k + 1)
                .unwrap_or(N)
        );

        assert_eq!(
            (0..r - l).map(|k| wm.quantile(l..r, k)).collect::<Vec<_>>(),
            {
                let mut v: Vec<_> = v[l..r].to_vec();
                v.sort_unstable();
                v
            }
        );

        assert_eq!(
            wm.rank_lessthan(a, l..r),
            v[l..r].iter().filter(|&&x| x < a).count()
        );

        let p = rand.rand(A as u64) as usize;
        let q = rand.rand(A as u64) as usize;
        let (p, q) = if p < q { (p, q) } else { (q, p) };
        assert_eq!(
            wm.rank_range(p..q, l..r),
            v[l..r].iter().filter(|&&x| p <= x && x < q).count()
        );
    }
}

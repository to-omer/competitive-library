use super::BitSet;
#[cfg(target_arch = "x86_64")]
use super::{SimdBackend, simd_backend};
use std::ops::{BitXorAssign, Index, IndexMut, Mul};

/// A matrix over GF(2), stored as packed rows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitMatrix {
    pub shape: (usize, usize),
    pub data: Vec<BitSet>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitMatrixSolution {
    pub particular: BitSet,
    pub basis: Vec<BitSet>,
}

impl BitMatrix {
    pub fn zeros(shape: (usize, usize)) -> Self {
        Self {
            shape,
            data: vec![BitSet::new(shape.1); shape.0],
        }
    }

    pub fn from_vec(data: Vec<BitSet>) -> Self {
        let shape = (data.len(), data.first().map_or(0, BitSet::len));
        assert!(data.iter().all(|row| row.len() == shape.1));
        Self { shape, data }
    }

    pub fn new_with(shape: (usize, usize), mut f: impl FnMut(usize, usize) -> bool) -> Self {
        let mut a = Self::zeros(shape);
        for (i, row) in a.data.iter_mut().enumerate() {
            for (w, word) in row.words_mut().iter_mut().enumerate() {
                for j in w * 64..shape.1.min((w + 1) * 64) {
                    *word |= u64::from(f(i, j)) << (j & 63);
                }
            }
        }
        a
    }

    pub fn eye(shape: (usize, usize)) -> Self {
        let mut a = Self::zeros(shape);
        for i in 0..shape.0.min(shape.1) {
            a[i].set(i, true);
        }
        a
    }

    pub fn transpose(&self) -> Self {
        let mut a = Self::zeros((self.shape.1, self.shape.0));
        for (i, row) in self.data.iter().enumerate() {
            for j in row.iter_ones() {
                a[j].set(i, true);
            }
        }
        a
    }

    /// Replaces the matrix with reduced row echelon form and returns its pivot columns.
    pub fn row_reduction(&mut self) -> Vec<usize> {
        self.eliminate(self.shape.1, true, false)
    }

    /// Replaces the matrix with row echelon form and returns its rank.
    pub fn rank(&mut self) -> usize {
        self.eliminate(self.shape.1, false, false).len()
    }

    /// Computes the determinant in place. The matrix must be square.
    pub fn determinant(&mut self) -> bool {
        assert_eq!(self.shape.0, self.shape.1);
        self.eliminate(self.shape.1, false, true).len() == self.shape.0
    }

    /// Returns the inverse, or `None` if singular. The matrix must be square.
    pub fn inverse(&self) -> Option<Self> {
        let (n, m) = self.shape;
        assert_eq!(n, m);
        let mut a = Self::zeros((n, 2 * n));
        for i in 0..n {
            a[i].words_mut()[..self[i].words().len()].copy_from_slice(self[i].words());
            a[i].set(n + i, true);
        }
        if a.eliminate(n, true, true).len() != n {
            return None;
        }
        for row in &mut a.data {
            *row >>= n;
        }
        let mut inverse = Self::zeros((n, n));
        for (row, source) in inverse.data.iter_mut().zip(&a.data) {
            let len = row.words().len();
            row.words_mut().copy_from_slice(&source.words()[..len]);
        }
        Some(inverse)
    }

    /// Returns a particular solution and a basis of the kernel, or `None` if inconsistent.
    /// `b.len()` must equal the number of rows.
    pub fn solve_system_of_linear_equations(&self, b: &BitSet) -> Option<BitMatrixSolution> {
        let (n, m) = self.shape;
        assert_eq!(b.len(), n);
        let mut a = Self::zeros((n, m + 1));
        for i in 0..n {
            a[i].words_mut()[..self[i].words().len()].copy_from_slice(self[i].words());
            a[i].set(m, b.get(i));
        }
        let pivots = a.eliminate(m, true, false);
        if a.data[pivots.len()..].iter().any(|row| row.get(m)) {
            return None;
        }
        let mut particular = BitSet::new(m);
        let mut free = BitSet::ones(m);
        for (i, &c) in pivots.iter().enumerate() {
            particular.set(c, a[i].get(m));
            free.set(c, false);
        }
        let columns: Vec<_> = free.iter_ones().collect();
        let mut basis: Vec<_> = columns
            .iter()
            .map(|&c| {
                let mut row = BitSet::new(m);
                row.set(c, true);
                row
            })
            .collect();
        for (i, &p) in pivots.iter().enumerate() {
            for (row, &c) in basis.iter_mut().zip(&columns) {
                row.words_mut()[p / 64] |= u64::from(a[i].get(c)) << (p & 63);
            }
        }
        Some(BitMatrixSolution { particular, basis })
    }

    pub fn pow(self, mut n: usize) -> Self {
        assert_eq!(self.shape.0, self.shape.1);
        let mut result = Self::eye(self.shape);
        let mut a = self;
        while n != 0 {
            if n & 1 != 0 {
                result = &result * &a;
            }
            n >>= 1;
            if n != 0 {
                a = &a * &a;
            }
        }
        result
    }

    fn eliminate(&mut self, cols: usize, full: bool, require_full_rank: bool) -> Vec<usize> {
        #[cfg(target_arch = "x86_64")]
        match simd_backend() {
            // SAFETY: the dispatcher checks the required CPU features.
            SimdBackend::Avx512 => {
                return unsafe { self.eliminate_avx512(cols, full, require_full_rank) };
            }
            // SAFETY: the dispatcher checks AVX2 support.
            SimdBackend::Avx2 => {
                return unsafe { self.eliminate_avx2(cols, full, require_full_rank) };
            }
            SimdBackend::Scalar => {}
        }
        self.eliminate_impl(cols, full, require_full_rank)
    }

    // Inlined into each target-feature entry point to vectorize the row operations.
    #[inline(always)]
    fn eliminate_impl(&mut self, cols: usize, full: bool, require_full_rank: bool) -> Vec<usize> {
        let n = self.shape.0;
        let mut pivots = Vec::with_capacity(n.min(cols));
        if n < 32 {
            let mut c = 0;
            while c < cols {
                let r = pivots.len();
                if r == n {
                    break;
                }
                let Some(p) = (r..n).find(|&i| self[i].get(c)) else {
                    if require_full_rank {
                        return pivots;
                    }
                    c = self.next_column(r, c + 1, cols);
                    continue;
                };
                self.data.swap(r, p);
                let (upper, lower) = self.data.split_at_mut(r);
                let (pivot, lower) = lower.split_first_mut().unwrap();
                for row in lower
                    .iter_mut()
                    .chain(upper.iter_mut().take(if full { r } else { 0 }))
                {
                    if row.get(c) {
                        xor(&mut row.words_mut()[c / 64..], &pivot.words()[c / 64..]);
                    }
                }
                pivots.push(c);
                c += 1;
            }
            return pivots;
        }
        // Eliminating a shared leading coefficient preserves the two-coefficient bound.
        if self
            .data
            .iter()
            .all(|row| row.iter_ones().take_while(|&c| c < cols).take(3).count() <= 2)
        {
            return self.eliminate_sparse(cols, full, require_full_rank);
        }
        let block: usize = if n < 512 {
            4
        } else if n < 1536 {
            8
        } else {
            32
        };
        let mut table = vec![BitSet::new(self.shape.1); (1 << block.min(8)) * block.div_ceil(8)];
        let mut reduced = vec![0; n];
        let mut start = 0;
        while start < cols {
            let first = pivots.len();
            let word = start / 64;
            reduced.fill(first);
            let end = cols.min(start + block);
            let mut c = start;
            while c < end {
                let r = pivots.len();
                let mut pivot = None;
                for (i, reduced) in reduced.iter_mut().enumerate().skip(r) {
                    let (upper, lower) = self.data.split_at_mut(i);
                    let row = &mut lower[0];
                    for p in *reduced..r {
                        if row.get(pivots[p]) {
                            xor(&mut row.words_mut()[word..], &upper[p].words()[word..]);
                        }
                    }
                    *reduced = r;
                    if row.get(c) {
                        pivot = Some(i);
                        break;
                    }
                }
                if let Some(p) = pivot {
                    self.data.swap(r, p);
                    reduced.swap(r, p);
                    pivots.push(c);
                    c += 1;
                } else if require_full_rank {
                    return pivots;
                } else {
                    c = self.next_column(r, c + 1, end);
                }
            }
            let rank = pivots.len();
            if first == rank {
                let next = self.next_column(rank, start + block, cols);
                if next == cols {
                    break;
                }
                start = next / block * block;
                continue;
            }
            // Make the panel's pivot columns an identity matrix before indexing its table.
            for r in (first..rank).rev() {
                let (upper, lower) = self.data.split_at_mut(r);
                for row in &mut upper[first..] {
                    if row.get(pivots[r]) {
                        xor(&mut row.words_mut()[word..], &lower[0].words()[word..]);
                    }
                }
            }
            let mut indices = [[0usize; 256]; 4];
            let mut masks = [0usize; 4];
            for group in 0..block.div_ceil(8) {
                let mut keys = [0usize; 256];
                let mut count = 0;
                for (row, &c) in pivots[first..].iter().enumerate() {
                    if (c - start) / 8 != group {
                        continue;
                    }
                    let half = 1 << count;
                    count += 1;
                    for index in 0..half {
                        keys[index + half] = keys[index] | (1 << ((c - start) % 8));
                        let (lower, upper) = table.split_at_mut(group * 256 + index + half);
                        let target = &mut upper[0].words_mut()[word..];
                        let source = &lower[group * 256 + index].words()[word..];
                        let pivot = &self[first + row].words()[word..];
                        for ((x, y), z) in target.iter_mut().zip(source).zip(pivot) {
                            *x = y ^ z;
                        }
                    }
                }
                masks[group] = keys[(1 << count) - 1];
                for (i, &key) in keys[..1 << count].iter().enumerate() {
                    indices[group][key] = i;
                }
            }
            for i in (rank..n).chain(0..if full { first } else { 0 }) {
                let key = (self[i].words()[word] >> (start & 63)) as usize;
                let x = indices[0][key & masks[0]];
                if block <= 8 {
                    if x != 0 {
                        xor(&mut self[i].words_mut()[word..], &table[x].words()[word..]);
                    }
                    continue;
                }
                let y = indices[1][(key >> 8) & masks[1]];
                let z = indices[2][(key >> 16) & masks[2]];
                let w = indices[3][(key >> 24) & masks[3]];
                if x | y | z | w != 0 {
                    let p = &table[x].words()[word..];
                    let q = &table[256 + y].words()[word..];
                    let r = &table[512 + z].words()[word..];
                    let s = &table[768 + w].words()[word..];
                    for ((((x, y), z), r), s) in self[i].words_mut()[word..]
                        .iter_mut()
                        .zip(p)
                        .zip(q)
                        .zip(r)
                        .zip(s)
                    {
                        *x ^= y ^ z ^ r ^ s;
                    }
                }
            }
            if rank == n {
                break;
            }
            start += block;
        }
        pivots
    }

    fn next_column(&self, row: usize, mut start: usize, cols: usize) -> usize {
        while start < cols {
            let word = start / 64;
            let bits = self.data[row..]
                .iter()
                .fold(0, |x, row| x | row.words()[word])
                & (u64::MAX << (start & 63));
            if bits != 0 {
                return (word * 64 + bits.trailing_zeros() as usize).min(cols);
            }
            start = (word + 1) * 64;
        }
        cols
    }

    #[inline(always)]
    fn eliminate_sparse(&mut self, cols: usize, full: bool, require_full_rank: bool) -> Vec<usize> {
        let n = self.shape.0;
        let mut basis = vec![n; cols];
        let mut pivots = Vec::new();
        for i in 0..n {
            loop {
                let Some(c) = self[i].iter_ones().next().filter(|&c| c < cols) else {
                    if require_full_rank {
                        return pivots;
                    }
                    break;
                };
                if basis[c] == n {
                    basis[c] = i;
                    pivots.push(c);
                    break;
                }
                let (upper, lower) = self.data.split_at_mut(i);
                xor(
                    &mut lower[0].words_mut()[c / 64..],
                    &upper[basis[c]].words()[c / 64..],
                );
            }
        }
        pivots.sort_unstable();
        self.data
            .sort_by_cached_key(|row| row.iter_ones().next().filter(|&c| c < cols).unwrap_or(cols));
        if full {
            for (i, &c) in pivots.iter().enumerate() {
                basis[c] = i;
            }
            for i in (0..pivots.len()).rev() {
                let next = self[i].iter_ones().take_while(|&c| c < cols).nth(1);
                if let Some(c) = next
                    && basis[c] != n
                {
                    let (upper, lower) = self.data.split_at_mut(basis[c]);
                    xor(
                        &mut upper[i].words_mut()[c / 64..],
                        &lower[0].words()[c / 64..],
                    );
                }
            }
        }
        pivots
    }

    #[inline(always)]
    fn mul_impl(&self, rhs: &Self) -> Self {
        let mut result = Self::zeros((self.shape.0, rhs.shape.1));
        let ones = self.data.iter().map(BitSet::count_ones).sum::<u64>();
        let size = self.shape.0 as u64 * self.shape.1 as u64;
        if self.shape.0 < 256 || self.shape.1 < 32 || ones <= size / 8 {
            for (a, c) in self.data.iter().zip(&mut result.data) {
                for j in a.iter_ones() {
                    xor(c.words_mut(), rhs[j].words());
                }
            }
            return result;
        }
        if size - ones <= size / 8 {
            let mut sum = BitSet::new(rhs.shape.1);
            for row in &rhs.data {
                sum ^= row;
            }
            for (a, c) in self.data.iter().zip(&mut result.data) {
                c.words_mut().copy_from_slice(sum.words());
                for j in (!a.clone()).iter_ones() {
                    xor(c.words_mut(), rhs[j].words());
                }
            }
            return result;
        }
        let width = rhs.shape.1.div_ceil(64);
        if width == 0 {
            return result;
        }

        // Separate the table groups by a cache line to avoid mapping them to the same sets.
        let group = 256 * width + 8;
        let mut storage = BitSet::new(8 * group * 64);
        let table = storage.words_mut();
        for start in (0..self.shape.1).step_by(64) {
            for (t, table) in table.chunks_exact_mut(group).enumerate() {
                let col = start + t * 8;
                for bit in 0..self.shape.1.saturating_sub(col).min(8) {
                    let row = rhs[col + bit].words();
                    let half = (1 << bit) * width;
                    let (lower, upper) = table.split_at_mut(half);
                    for (source, dest) in
                        lower.chunks_exact(width).zip(upper.chunks_exact_mut(width))
                    {
                        for ((x, y), z) in dest.iter_mut().zip(source).zip(row) {
                            *x = y ^ z;
                        }
                    }
                }
            }
            for (a, c) in self.data.iter().zip(&mut result.data) {
                let key = a.words()[start / 64];
                let offset = (key & 255) as usize * width;
                let p0 = &table[offset..offset + width];
                let offset = group + (key >> 8 & 255) as usize * width;
                let p1 = &table[offset..offset + width];
                let offset = 2 * group + (key >> 16 & 255) as usize * width;
                let p2 = &table[offset..offset + width];
                let offset = 3 * group + (key >> 24 & 255) as usize * width;
                let p3 = &table[offset..offset + width];
                let offset = 4 * group + (key >> 32 & 255) as usize * width;
                let p4 = &table[offset..offset + width];
                let offset = 5 * group + (key >> 40 & 255) as usize * width;
                let p5 = &table[offset..offset + width];
                let offset = 6 * group + (key >> 48 & 255) as usize * width;
                let p6 = &table[offset..offset + width];
                let offset = 7 * group + (key >> 56 & 255) as usize * width;
                let p7 = &table[offset..offset + width];
                for ((((((((x, p0), p1), p2), p3), p4), p5), p6), p7) in c
                    .words_mut()
                    .iter_mut()
                    .zip(p0)
                    .zip(p1)
                    .zip(p2)
                    .zip(p3)
                    .zip(p4)
                    .zip(p5)
                    .zip(p6)
                    .zip(p7)
                {
                    *x ^= p0 ^ p1 ^ p2 ^ p3 ^ p4 ^ p5 ^ p6 ^ p7;
                }
            }
        }
        result
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn eliminate_avx2(
        &mut self,
        cols: usize,
        full: bool,
        require_full_rank: bool,
    ) -> Vec<usize> {
        self.eliminate_impl(cols, full, require_full_rank)
    }
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
    unsafe fn eliminate_avx512(
        &mut self,
        cols: usize,
        full: bool,
        require_full_rank: bool,
    ) -> Vec<usize> {
        self.eliminate_impl(cols, full, require_full_rank)
    }
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn mul_avx2(&self, rhs: &Self) -> Self {
        self.mul_impl(rhs)
    }
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
    unsafe fn mul_avx512(&self, rhs: &Self) -> Self {
        self.mul_impl(rhs)
    }
}

#[inline(always)]
fn xor(row: &mut [u64], pivot: &[u64]) {
    for (x, y) in row.iter_mut().zip(pivot) {
        *x ^= y;
    }
}

impl Index<usize> for BitMatrix {
    type Output = BitSet;
    fn index(&self, i: usize) -> &Self::Output {
        &self.data[i]
    }
}
impl IndexMut<usize> for BitMatrix {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.data[i]
    }
}
impl BitXorAssign<&Self> for BitMatrix {
    fn bitxor_assign(&mut self, rhs: &Self) {
        assert_eq!(self.shape, rhs.shape);
        for (a, b) in self.data.iter_mut().zip(&rhs.data) {
            *a ^= b;
        }
    }
}
impl Mul<&BitMatrix> for &BitMatrix {
    type Output = BitMatrix;
    fn mul(self, rhs: &BitMatrix) -> BitMatrix {
        assert_eq!(self.shape.1, rhs.shape.0);
        #[cfg(target_arch = "x86_64")]
        match simd_backend() {
            // SAFETY: the dispatcher checks the required CPU features.
            SimdBackend::Avx512 => return unsafe { self.mul_avx512(rhs) },
            // SAFETY: the dispatcher checks AVX2 support.
            SimdBackend::Avx2 => return unsafe { self.mul_avx2(rhs) },
            SimdBackend::Scalar => {}
        }
        self.mul_impl(rhs)
    }
}
impl Mul for BitMatrix {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        &self * &rhs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    use std::collections::BTreeSet;

    #[test]
    fn test_random_small_linear_algebra() {
        let mut rng = Xorshift::new_with_seed(918217);
        for _ in 0..256 {
            let n = rng.random(0..=8);
            let m = rng.random(0..=8);
            let density = rng.rand(1010);
            let rows: Vec<Vec<bool>> = (0..n)
                .map(|_| (0..m).map(|_| rng.rand(1009) < density).collect())
                .collect();
            let a = BitMatrix::new_with((n, m), |i, j| rows[i][j]);
            // Enumerate the image and every solution independently of elimination.
            let images: Vec<usize> = (0..1usize << m)
                .map(|x| {
                    rows.iter().enumerate().fold(0, |image, (i, row)| {
                        let bit = row
                            .iter()
                            .enumerate()
                            .fold(false, |v, (j, &b)| v ^ (b && x >> j & 1 != 0));
                        image | (usize::from(bit) << i)
                    })
                })
                .collect();
            let image: BTreeSet<_> = images.iter().copied().collect();
            let rank = image.len().ilog2() as usize;
            assert_eq!(a.clone().rank(), rank);
            if n == m {
                assert_eq!(a.clone().determinant(), rank == n);
                let inverse = a.inverse();
                assert_eq!(inverse.is_some(), rank == n);
                if let Some(inverse) = inverse {
                    assert_eq!(&a * &inverse, BitMatrix::eye((n, n)));
                    assert_eq!(&inverse * &a, BitMatrix::eye((n, n)));
                }
            }
            for b in 0..1usize << n {
                let rhs = (0..n).map(|i| b >> i & 1 != 0).collect();
                let solution = a.solve_system_of_linear_equations(&rhs);
                assert_eq!(solution.is_some(), image.contains(&b));
                if let Some(solution) = solution {
                    assert_eq!(solution.basis.len(), m - rank);
                    let solutions: BTreeSet<_> = (0..1 << solution.basis.len())
                        .map(|mask| {
                            let mut x = solution.particular.clone();
                            for (i, row) in solution.basis.iter().enumerate() {
                                if mask >> i & 1 != 0 {
                                    x ^= row;
                                }
                            }
                            x.iter_ones().map(|i| 1 << i).sum::<usize>()
                        })
                        .collect();
                    let expected: BTreeSet<_> = images
                        .iter()
                        .enumerate()
                        .filter_map(|(x, &y)| (y == b).then_some(x))
                        .collect();
                    assert_eq!(solutions, expected);
                }
            }
        }
    }

    #[test]
    fn test_random_rectangular_matrices() {
        let mut rng = Xorshift::new_with_seed(1234567);
        for case in 0..184 {
            // Sample different scales and aspect ratios while bounding the Boolean oracle's cost.
            let (n, m, k) = match case {
                0..128 => (rng.random(0..=32), rng.random(0..=32), rng.random(0..=32)),
                128..160 => (rng.random(0..256), rng.random(0..256), rng.random(0..256)),
                _ => match case % 3 {
                    0 => (
                        rng.random(256..800),
                        rng.random(32..160),
                        rng.random(128..1600),
                    ),
                    1 => (
                        rng.random(32..160),
                        rng.random(256..1800),
                        rng.random(0..48),
                    ),
                    _ => (
                        rng.random(512..800),
                        rng.random(256..800),
                        rng.random(0..32),
                    ),
                },
            };
            let density = rng.rand(1010);
            let mut rows: Vec<Vec<bool>> = (0..n)
                .map(|_| (0..m).map(|_| rng.rand(1009) < density).collect())
                .collect();
            match rng.rand(5) {
                0 => {
                    for row in &mut rows {
                        row.fill(false);
                        if m != 0 {
                            for _ in 0..rng.rand(3) {
                                row[rng.random(0..m)] = true;
                            }
                        }
                    }
                }
                1 => {
                    let rank_bound = rng.random(0..=n.min(16));
                    for i in rank_bound..n {
                        rows[i] = if rank_bound == 0 {
                            vec![false; m]
                        } else {
                            let p = rng.random(0..rank_bound);
                            let q = rng.random(0..rank_bound);
                            rows[p].iter().zip(&rows[q]).map(|(x, y)| x ^ y).collect()
                        };
                    }
                }
                2 => {
                    let start = rng.random(0..=m);
                    let active: Vec<_> = (0..m).map(|j| j >= start && rng.rand(3) != 0).collect();
                    for row in &mut rows {
                        for (x, active) in row.iter_mut().zip(&active) {
                            *x &= active;
                        }
                    }
                }
                _ => {}
            }
            let a = BitMatrix::new_with((n, m), |i, j| rows[i][j]);
            let density = rng.rand(1010);
            let right: Vec<Vec<bool>> = (0..m)
                .map(|_| (0..k).map(|_| rng.rand(1009) < density).collect())
                .collect();
            let b = BitMatrix::new_with((m, k), |i, j| right[i][j]);
            let mut product = vec![vec![false; k]; n];
            for (row, result) in rows.iter().zip(&mut product) {
                for (&bit, source) in row.iter().zip(&right) {
                    if bit {
                        for (x, y) in result.iter_mut().zip(source) {
                            *x ^= y;
                        }
                    }
                }
            }
            let expected = BitMatrix::new_with((n, k), |i, j| product[i][j]);
            assert_eq!(&a * &b, expected, "case {case}, shape {:?}", (n, m, k));
            let complement = BitMatrix::new_with((n, m), |i, j| !rows[i][j]);
            let complement_expected = BitMatrix::new_with((n, k), |i, j| {
                (0..m).fold(false, |v, t| v ^ (!rows[i][t] && right[t][j]))
            });
            assert_eq!(&complement * &b, complement_expected, "case {case}");
            assert_eq!(
                a.transpose(),
                BitMatrix::new_with((m, n), |i, j| rows[j][i])
            );

            // Reduce two right-hand sides with the same independent Boolean operations:
            // one known to be consistent and one unrestricted random vector.
            let x: Vec<bool> = (0..m).map(|_| rng.rand(1009) < 504).collect();
            let rhs: Vec<[bool; 2]> = rows
                .iter()
                .map(|row| {
                    [
                        row.iter().zip(&x).fold(false, |v, (a, b)| v ^ (a & b)),
                        rng.rand(1009) < 504,
                    ]
                })
                .collect();
            let mut reduced_rows = rows.clone();
            for (row, rhs) in reduced_rows.iter_mut().zip(&rhs) {
                row.extend(rhs);
            }
            let mut pivots = Vec::new();
            for c in 0..m {
                let r = pivots.len();
                if let Some(p) = (r..n).find(|&i| reduced_rows[i][c]) {
                    reduced_rows.swap(r, p);
                    let pivot = reduced_rows[r].clone();
                    for (i, row) in reduced_rows.iter_mut().enumerate() {
                        if i != r && row[c] {
                            for (x, y) in row.iter_mut().zip(&pivot) {
                                *x ^= y;
                            }
                        }
                    }
                    pivots.push(c);
                }
            }
            let rref = BitMatrix::new_with((n, m), |i, j| reduced_rows[i][j]);
            let mut reduced = a.clone();
            assert_eq!(reduced.row_reduction(), pivots, "case {case}");
            assert_eq!(reduced, rref, "case {case}");
            assert_eq!(a.clone().rank(), pivots.len(), "case {case}");
            let free: Vec<_> = (0..m).filter(|j| !pivots.contains(j)).collect();
            for t in 0..2 {
                let rhs_bits = rhs.iter().map(|b| b[t]).collect();
                let solution = a.solve_system_of_linear_equations(&rhs_bits);
                let consistent = reduced_rows[pivots.len()..].iter().all(|row| !row[m + t]);
                assert_eq!(solution.is_some(), consistent, "case {case}, rhs {t}");
                if let Some(solution) = solution {
                    assert_eq!(solution.particular.len(), m);
                    assert_eq!(solution.basis.len(), free.len());
                    for (row, rhs) in rows.iter().zip(&rhs) {
                        assert_eq!(
                            row.iter().enumerate().fold(false, |v, (j, &b)| {
                                v ^ (b && solution.particular.get(j))
                            }),
                            rhs[t]
                        );
                    }
                    for (i, vector) in solution.basis.iter().enumerate() {
                        assert_eq!(vector.len(), m);
                        for (j, &c) in free.iter().enumerate() {
                            assert_eq!(vector.get(c), i == j);
                        }
                        for (r, &c) in pivots.iter().enumerate() {
                            assert_eq!(vector.get(c), reduced_rows[r][free[i]]);
                        }
                    }
                }
            }
            #[cfg(target_arch = "x86_64")]
            {
                if is_x86_feature_detected!("avx2") {
                    // SAFETY: AVX2 support was checked above.
                    unsafe {
                        assert_eq!(a.mul_avx2(&b), expected);
                        assert_eq!(complement.mul_avx2(&b), complement_expected);
                        let mut reduced = a.clone();
                        assert_eq!(reduced.eliminate_avx2(m, true, false), pivots);
                        assert_eq!(reduced, rref);
                    }
                }
                if crate::tools::avx512_supported() {
                    // SAFETY: all required AVX-512 features were checked above.
                    unsafe {
                        assert_eq!(a.mul_avx512(&b), expected);
                        assert_eq!(complement.mul_avx512(&b), complement_expected);
                        let mut reduced = a.clone();
                        assert_eq!(reduced.eliminate_avx512(m, true, false), pivots);
                        assert_eq!(reduced, rref);
                    }
                }
                assert_eq!(a.mul_impl(&b), expected);
                assert_eq!(complement.mul_impl(&b), complement_expected);
                let mut reduced = a.clone();
                assert_eq!(reduced.eliminate_impl(m, true, false), pivots);
                assert_eq!(reduced, rref);
            }
        }
    }

    #[test]
    fn test_random_square_matrices() {
        let mut rng = Xorshift::new_with_seed(7712389);
        for case in 0..80 {
            let n = if case < 64 {
                rng.random(0..160)
            } else {
                rng.random(256..1200)
            };
            let mut a = BitMatrix::eye((n, n));
            match rng.rand(3) {
                0 => {
                    for i in 0..n {
                        for j in i + 1..n {
                            a[i].set(j, rng.rand(1009) < 504);
                        }
                    }
                }
                1 => {
                    for i in 0..n.saturating_sub(1) {
                        a[i].set(rng.random(i + 1..n), true);
                    }
                }
                _ => {}
            }
            rng.shuffle(&mut a.data);
            let inv = a.inverse().unwrap();
            assert!(a.clone().determinant());
            let identity = BitMatrix::eye((n, n));
            assert_eq!(&a * &inv, identity);
            assert_eq!(&inv * &a, identity);
            let b: BitSet = (0..n).map(|_| rng.rand(1009) < 504).collect();
            let sol = a.solve_system_of_linear_equations(&b).unwrap();
            assert!(sol.basis.is_empty());
            for i in 0..n {
                assert_eq!(
                    (0..n).fold(false, |v, j| v ^ (a[i].get(j) && sol.particular.get(j))),
                    b.get(i)
                );
            }
            let exponent = rng.random(0..10);
            let mut power = identity;
            for _ in 0..exponent {
                power = &power * &a;
            }
            assert_eq!(a.clone().pow(exponent), power);
            let other = BitMatrix::new_with((n, n), |_, _| rng.rand(1009) < 504);
            let mut sum = a.clone();
            sum ^= &other;
            assert_eq!(
                sum,
                BitMatrix::new_with((n, n), |i, j| a[i].get(j) ^ other[i].get(j))
            );
            if n != 0 {
                let i = rng.random(0..n);
                if n > 1 && rng.rand(2) != 0 {
                    let j = (i + rng.random(1..n)) % n;
                    a.data[i] = a[j].clone();
                } else {
                    a[i].reset();
                }
                assert!(!a.clone().determinant());
                assert!(a.inverse().is_none());
                assert_eq!(a.rank(), n - 1);
            }
        }
    }
}

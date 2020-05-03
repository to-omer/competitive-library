#[cargo_snippet::snippet("SuffixArray")]
#[derive(Clone, Debug)]
pub struct SuffixArray<T> {
    pat: Vec<T>,
    sa: Vec<usize>,
    rank: Vec<usize>,
}
#[cargo_snippet::snippet("SuffixArray")]
impl<T: Ord> SuffixArray<T> {
    pub fn new(pat: Vec<T>) -> Self {
        let n = pat.len();
        let mut sa = (0..n + 1).collect::<Vec<_>>();
        let mut rank = vec![0; n + 1];
        let mut ford = (0..n).collect::<Vec<_>>();
        ford.sort_by_key(|&i| &pat[i]);
        let mut c = 1;
        for i in 0..n {
            rank[ford[i]] = c;
            if i + 1 < n && pat[ford[i]] != pat[ford[i + 1]] {
                c += 1;
            }
        }
        let mut k = 1;
        while k <= n {
            sa.sort_by_key(|&i| (rank[i], rank.get(i + k).unwrap_or(&0)));
            let mut tmp = vec![0; n + 1];
            tmp[sa[0]] = 1;
            for i in 1..n + 1 {
                let x = sa[i - 1];
                let y = sa[i];
                let b = (rank[x], rank.get(x + k).unwrap_or(&0))
                    < (rank[y], rank.get(y + k).unwrap_or(&0));
                tmp[y] = tmp[x] + b as usize;
            }
            rank = tmp;
            k *= 2;
        }
        SuffixArray {
            pat: pat,
            sa: sa,
            rank: rank,
        }
    }
    pub fn len(&self) -> usize {
        self.sa.len()
    }
    pub fn longest_common_prefix_array(&self) -> Vec<usize> {
        let n = self.pat.len();
        let mut h = 0;
        let mut lcp = vec![0; n];
        for i in 0..n {
            let j = self[self.rank[i] - 2];
            if h > 0 {
                h -= 1;
            }
            while j + h < n && i + h < n && self.pat[j + h] == self.pat[i + h] {
                h += 1;
            }
            lcp[self.rank[i] - 2] = h;
        }
        lcp
    }
}
impl<T> std::ops::Index<usize> for SuffixArray<T> {
    type Output = usize;
    fn index(&self, index: usize) -> &Self::Output {
        &self.sa[index]
    }
}

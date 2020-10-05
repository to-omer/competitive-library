#[snippet::entry("ZAlgorithm")]
#[derive(Clone, Debug)]
pub struct Zarray {
    z: Vec<usize>,
}
#[snippet::entry("ZAlgorithm")]
impl Zarray {
    pub fn new<T: Eq>(s: &[T]) -> Self {
        let n = s.len();
        let mut z = vec![0; n];
        z[0] = n;
        let (mut i, mut j) = (1, 0);
        while i < n {
            while i + j < n && s[j] == s[i + j] {
                j += 1;
            }
            z[i] = j;
            if j == 0 {
                i += 1;
                continue;
            }
            let mut k = 1;
            while i + k < n && k + z[k] < j {
                z[i + k] = z[k];
                k += 1;
            }
            i += k;
            j -= k;
        }
        Self { z }
    }
    pub fn search<T: Eq>(s: &[T], pat: &[T], sep: T) -> Vec<usize> {
        let mut res = vec![];
        let mut t = vec![];
        t.extend(pat);
        t.push(&sep);
        t.extend(s);
        let zarray = Self::new(&t);
        for i in 0..t.len() {
            if zarray[i] == pat.len() {
                res.push(i - pat.len() - 1);
            }
        }
        res
    }
}
#[snippet::entry("ZAlgorithm")]
impl std::ops::Index<usize> for Zarray {
    type Output = usize;
    fn index(&self, index: usize) -> &usize {
        &self.z[index]
    }
}

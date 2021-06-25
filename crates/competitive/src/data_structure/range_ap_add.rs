#[derive(Debug, Clone)]
pub struct RangeArithmeticProgressionAdd {
    pub dd: Vec<i64>,
}
impl RangeArithmeticProgressionAdd {
    pub fn new(n: usize) -> Self {
        Self { dd: vec![0; n] }
    }
    /// add a, a+d, ..., a+(k-1)d into [l, l + k)
    pub fn update(&mut self, l: usize, k: usize, a: i64, d: i64) {
        if let Some(e) = self.dd.get_mut(l) {
            *e += a;
        }
        if let Some(e) = self.dd.get_mut(l + 1) {
            *e += d - a;
        }
        if let Some(e) = self.dd.get_mut(l + k) {
            *e += -a - k as i64 * d;
        }
        if let Some(e) = self.dd.get_mut(l + k + 1) {
            *e += a + (k as i64 - 1) * d;
        }
    }
    /// add a, a+d, ..., a+(k-1)d into [l, l + k)
    pub fn update_isize(&mut self, l: isize, k: usize, a: i64, d: i64) {
        if l < 0 {
            let r = l + k as isize;
            if r > 0 {
                self.update(0, r as usize, a - l as i64 * d, d);
            }
        } else {
            self.update(l as usize, k, a, d);
        }
    }
    pub fn build_inplace(&mut self) {
        for _ in 0..2 {
            for i in 0..self.dd.len() - 1 {
                self.dd[i + 1] += self.dd[i];
            }
        }
    }
}

#[test]
fn test_range_ap_add() {
    use crate::tools::{NotEmptySegment as Nes, Xorshift};
    const N: usize = 1000;
    const Q: usize = 10000;
    const A: i64 = 1_000_000_000;
    let mut rng = Xorshift::time();
    let mut v = vec![0i64; N];
    let mut ap = RangeArithmeticProgressionAdd::new(N);
    for ((l, r), a, d) in rng.gen_iter((Nes(N), -A..=A, -A..=A)).take(Q) {
        for (i, v) in v[l..r].iter_mut().enumerate() {
            *v += a + i as i64 * d;
        }
        ap.update(l, r - l, a, d);
    }
    ap.build_inplace();
    assert_eq!(ap.dd, v);
}

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
            for i in 1..self.dd.len() {
                self.dd[i] += self.dd[i - 1];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{WithEmptySegment, Xorshift, testutil::integer_cases};

    const A: i64 = 1_000_000_000;

    #[test]
    fn test_range_ap_add() {
        let mut rng = Xorshift::default();
        let mut range_rng = Xorshift::default();
        for n in integer_cases(0..=8_192) {
            let mut expected = vec![0i64; n];
            let mut ap = RangeArithmeticProgressionAdd::new(n);
            let boundaries = [
                (0, 0),
                (0, n.min(1)),
                (0, n),
                (n.saturating_sub(1), n),
                (n, n),
            ];
            for (l, r) in boundaries
                .into_iter()
                .chain(range_rng.random_iter(WithEmptySegment(n)).take(n))
            {
                let (a, d) = rng.random((-A..=A, -A..=A));
                ap.update(l, r - l, a, d);
                for (i, value) in expected[l..r].iter_mut().enumerate() {
                    *value += a + i as i64 * d;
                }
            }
            ap.build_inplace();
            assert_eq!(ap.dd, expected, "n={n}");
        }
    }

    #[test]
    fn test_range_ap_add_isize() {
        let mut rng = Xorshift::default();
        let mut range_rng = Xorshift::default();
        for n in integer_cases(0..=4_096) {
            let size = n as isize;
            let boundaries = [
                (-size - 1, -1),
                (-1, size + 1),
                (0, size),
                (-1, 0),
                (size, size),
                (size, size + 1),
            ];
            let ranges = boundaries.into_iter().chain((0..n).map(|_| {
                let l = range_rng.random(-size..=size);
                let r = range_rng.random(l..=2 * size);
                (l, r)
            }));
            for (case, (l, r)) in ranges.enumerate() {
                let (a, magnitude) = rng.random((-A..=A, 1..=A));
                let d = if case % 2 == 0 { magnitude } else { -magnitude };
                let mut ap = RangeArithmeticProgressionAdd::new(n);
                ap.update_isize(l, (r - l) as usize, a, d);
                let expected: Vec<_> = (0..n)
                    .map(|i| {
                        let i = i as isize;
                        if l <= i && i < r {
                            a + (i - l) as i64 * d
                        } else {
                            0
                        }
                    })
                    .collect();
                ap.build_inplace();
                assert_eq!(ap.dd, expected, "n={n}, range={l}..{r}, a={a}, d={d}");
            }
        }
    }
}

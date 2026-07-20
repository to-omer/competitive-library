use super::{ConvolveRealFft, Xorshift};

pub fn wildcard_pattern_matching(p: &[u8], s: &[u8]) -> Vec<bool> {
    assert!(!p.is_empty());
    assert!(p.len() <= s.len());
    let mut rng = Xorshift::new();
    let mut direct = [0.0; 256];
    let mut inverse = [0.0; 256];
    for i in 0..256 {
        let x = 1.25 + 0.75 * rng.randf();
        direct[i] = x;
        inverse[i] = 1.0 / x;
    }
    direct[b'?' as usize] = 0.0;
    inverse[b'?' as usize] = 0.0;
    ConvolveRealFft::middle_product_f64(
        s.iter().map(|&c| direct[c as usize]),
        p.iter().rev().map(|&c| inverse[c as usize]),
    )
    .into_iter()
    .map(|x| (x - x.round()).abs() < 1e-8)
    .collect()
}

#[cfg(test)]
mod tests {
    use super::wildcard_pattern_matching;
    use crate::tools::Xorshift;

    #[test]
    fn test_wildcard_pattern_matching() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            let n = rng.rand(20) as usize + 1;
            let m = n + rng.rand(30) as usize;
            let mut p: Vec<_> = (0..n).map(|_| b"abc?"[rng.rand(4) as usize]).collect();
            let mut s: Vec<_> = (0..m).map(|_| b"abc?"[rng.rand(4) as usize]).collect();
            match rng.rand(4) {
                0 => p.copy_from_slice(&s[..n]),
                1 => p.fill(b'?'),
                2 => s.fill(b'?'),
                _ => {}
            }
            let expected: Vec<_> = (0..=s.len() - p.len())
                .map(|i| {
                    p.iter()
                        .zip(&s[i..])
                        .all(|(&a, &b)| a == b || a == b'?' || b == b'?')
                })
                .collect();
            assert_eq!(expected, wildcard_pattern_matching(&p, &s));
        }
    }
}

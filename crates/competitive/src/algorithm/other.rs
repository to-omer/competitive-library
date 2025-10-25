#[codesnip::entry]
/// return: \[(elem, length)\]
pub fn run_length_encoding<T, I>(iter: I) -> Vec<(T, usize)>
where
    T: Clone + PartialEq,
    I: IntoIterator<Item = T>,
{
    let mut res = Vec::new();
    for a in iter.into_iter() {
        if let Some((p, len)) = res.last_mut() {
            if p == &a {
                *len += 1;
                continue;
            }
        }
        res.push((a, 1));
    }
    res
}

#[codesnip::entry]
/// $y = \left\lfloor\frac{n}{x}\right\rfloor$
///
/// segments that have same x or y
pub fn floor_kernel(n: usize) -> Vec<usize> {
    let m = (n as f64).sqrt() as usize;
    let mut res = Vec::with_capacity(m * 2 + 1);
    for i in 1..=m {
        res.push(i);
    }
    if n / m + 1 != m + 1 {
        res.push(m + 1);
    }
    for i in (1..=m).rev() {
        res.push(n / i + 1);
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{rand, tools::Xorshift};

    #[test]
    fn test_run_length_encoding() {
        let mut rng = Xorshift::default();
        const N: usize = 100_000;
        rand!(rng, v: [0u8..8u8; N]);
        let r = run_length_encoding(v.iter());
        let mut s = 0;
        for (a, l) in r {
            for v in &v[s..s + l] {
                assert_eq!(a, v);
            }
            s += l;
        }
    }

    #[test]
    fn test_floor_kernel() {
        for n in 1..1000 {
            let k = floor_kernel(n);
            let from = k.iter().cloned().zip(k.iter().cloned().skip(1));
            let to = k.iter().cloned().zip(k.iter().cloned().skip(1)).rev();
            for ((a, b), (c, d)) in from.zip(to) {
                assert!(a < b);
                assert!(c < d);
                for x in a..b {
                    for y in c..d {
                        assert!(x * y <= n);
                    }
                }
            }
        }
    }
}

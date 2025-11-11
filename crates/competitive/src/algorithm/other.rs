#[codesnip::entry]
/// return: \[(elem, length)\]
pub fn run_length_encoding<T, I>(iter: I) -> Vec<(T, usize)>
where
    T: Clone + PartialEq,
    I: IntoIterator<Item = T>,
{
    let mut res = Vec::new();
    for a in iter.into_iter() {
        if let Some((p, len)) = res.last_mut()
            && p == &a
        {
            *len += 1;
            continue;
        }
        res.push((a, 1));
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
}

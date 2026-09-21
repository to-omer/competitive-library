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
    use crate::tools::Xorshift;
    use crate::tools::testutil::{exhaustive_sequences, sample_usize, structured_sequences};
    use std::iter::repeat_n;

    #[test]
    fn test_run_length_encoding() {
        let mut rng = Xorshift::default();
        let lengths = sample_usize(&mut rng, 16, 0..=100_000, 2);
        for values in
            exhaustive_sequences(0..3, 0..=8).chain(structured_sequences(&mut rng, 0..8, lengths))
        {
            let runs = run_length_encoding(values.iter().copied());
            assert!(runs.iter().all(|&(_, len)| len > 0));
            assert!(runs.windows(2).all(|w| w[0].0 != w[1].0));
            let restored: Vec<_> = runs
                .into_iter()
                .flat_map(|(value, len)| repeat_n(value, len))
                .collect();
            assert_eq!(restored, values);
        }
    }
}

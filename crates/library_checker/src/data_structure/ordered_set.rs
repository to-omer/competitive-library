use competitive::prelude::*;
use competitive::{
    algorithm::SliceSortExt,
    data_structure::{RankSelectBitSet, RankSelectDictionaries, SqrtSet, StaticSearch},
    tools::IteratorExt,
};

#[verify::library_checker(
    "ordered_set",
    include("RankSelectBitSet", "StaticSearch", "sort", "IteratorExt")
)]
pub fn ordered_set(reader: impl Read, writer: impl Write) {
    prepare_io!(buffered; reader, writer);
    sc!(n, q, a: [u32; n], queries: [(u8, u32); q]);
    let mut updates: Vec<_> = queries
        .iter()
        .filter_map(|&(t, x)| (t == 0).then_some(x))
        .collect();
    let minimum = a
        .first()
        .copied()
        .into_iter()
        .chain(updates.iter().copied())
        .min();
    let maximum = a
        .last()
        .copied()
        .into_iter()
        .chain(updates.iter().copied())
        .max();
    let (values, words) = if let Some((minimum, maximum)) = minimum.zip(maximum)
        && ((maximum - minimum) as usize) < 2 * (n + updates.len())
    {
        drop(updates);
        let values: Vec<_> = (minimum..=maximum).collect();
        let mut words = vec![0; values.len().div_ceil(64)];
        for x in a {
            let index = (x - minimum) as usize;
            words[index / 64] |= 1 << (index % 64);
        }
        (values, words)
    } else {
        updates.radix_sort_by_key(|&x| x);
        updates.dedup();
        let mut values = Vec::with_capacity(n + updates.len());
        let mut words = vec![0; (n + updates.len()).div_ceil(64)];
        let merged = a
            .into_iter()
            .map(|x| (x, true))
            .merge_by(updates.into_iter().map(|x| (x, false)), |x, y| x.0 <= y.0);
        for (value, present) in merged {
            if values.last() != Some(&value) {
                values.push(value);
            }
            if present {
                let k = values.len() - 1;
                words[k / 64] |= 1 << (k % 64);
            }
        }
        words.truncate(values.len().div_ceil(64));
        (values, words)
    };
    let search = StaticSearch::from_sorted(&values);
    let endpoints: Vec<_> = queries
        .iter()
        .filter_map(|&(t, x)| (t != 2).then_some(if t == 3 || t == 4 { x + 1 } else { x }))
        .collect();
    let mut positions = vec![0; endpoints.len()];
    search.lower_bound_batch(&endpoints, &mut positions);
    let mut set = RankSelectBitSet::from_words(&words, values.len());
    let mut positions = positions.into_iter();
    for (t, x) in queries {
        let k = if t == 2 { 0 } else { positions.next().unwrap() };
        match t {
            0 => set.set(k, true),
            1 => {
                if values.get(k) == Some(&x) {
                    set.set(k, false);
                }
            }
            2 => {
                pp!(set.select1(x as usize - 1).map_or(-1, |k| values[k] as i64));
            }
            3 => {
                pp!(set.rank1(k));
            }
            4 => {
                pp!(set.prev_one(k).map_or(-1, |k| values[k] as i64));
            }
            5 => {
                pp!(set.next_one(k).map_or(-1, |k| values[k] as i64));
            }
            _ => unreachable!(),
        }
    }
}

#[verify::library_checker("ordered_set", include("SqrtTree"))]
pub fn ordered_set_sqrt_set(reader: impl Read, writer: impl Write) {
    prepare_io!(buffered; reader, writer);
    sc!(n, q, a: [u32; iter n]);
    let mut set: SqrtSet<_> = a.collect();
    for _ in 0..q {
        sc!(t: u8, x: u32);
        match t {
            0 => {
                set.insert(x);
            }
            1 => {
                set.remove(&x);
            }
            2 => pp!(set.get_at(x as usize - 1).map_or(-1, |&v| v as i64)),
            3 => pp!(set.rank_le(&x)),
            4 => pp!(set.range(..=x).next_back().map_or(-1, |&v| v as i64)),
            5 => pp!(set.range(x..).next().map_or(-1, |&v| v as i64)),
            _ => unreachable!(),
        }
    }
}

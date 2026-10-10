use super::BipartiteMatching;

/// Colors in `0..degree`, in edge order; `None` if nonregular. Each part has `size` vertices.
pub fn regular_bipartite_edge_coloring(
    size: usize,
    edges: &[(usize, usize)],
) -> Option<Vec<usize>> {
    let mut left_degree = vec![0; size];
    let mut right_degree = vec![0; size];
    for &(l, r) in edges {
        left_degree[l] += 1;
        right_degree[r] += 1;
    }
    let degree = left_degree.first().copied().unwrap_or(0);
    if left_degree
        .iter()
        .chain(&right_degree)
        .any(|&d| d != degree)
    {
        return None;
    }
    let mut colors = vec![!0; edges.len()];
    let mut remaining: Vec<_> = (0..edges.len()).collect();
    let mut selected = vec![!0; size];
    for color in 0..degree {
        let mut matching = BipartiteMatching::new(size, size);
        for &eid in &remaining {
            let (l, r) = edges[eid];
            matching.add_edge(l, r);
        }
        for (l, r) in matching.maximum_matching() {
            selected[l] = r;
        }
        remaining.retain(|&eid| {
            let (l, r) = edges[eid];
            if selected[l] == r {
                colors[eid] = color;
                selected[l] = !0;
                false
            } else {
                true
            }
        });
    }
    Some(colors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{Xorshift, testutil::exhaustive_sequences};
    use std::collections::BTreeSet;

    #[test]
    fn test_regular_bipartite_edge_coloring() {
        let check = |size: usize, edges: &[(usize, usize)]| {
            let degrees: BTreeSet<_> = (0..size)
                .flat_map(|v| {
                    [
                        edges.iter().filter(|&&(l, _)| l == v).count(),
                        edges.iter().filter(|&&(_, r)| r == v).count(),
                    ]
                })
                .collect();
            let regular = degrees.len() <= 1;
            let colors = regular_bipartite_edge_coloring(size, edges);
            assert_eq!(colors.is_some(), regular);
            if let Some(colors) = colors {
                let degree = degrees.first().copied().unwrap_or(0);
                assert_eq!(colors.len(), edges.len());
                let mut left = vec![vec![false; degree]; size];
                let mut right = left.clone();
                for (&(l, r), color) in edges.iter().zip(colors) {
                    assert!(color < degree);
                    assert!(!left[l][color]);
                    assert!(!right[r][color]);
                    left[l][color] = true;
                    right[r][color] = true;
                }
                assert!(left.into_iter().chain(right).flatten().all(|used| used));
            }
        };
        for size in 0usize..=4 {
            let multiplicities = if size <= 3 { 0..4 } else { 0..2 };
            let endpoints = (0..size).flat_map(|l| (0..size).map(move |r| (l, r)));
            for counts in exhaustive_sequences(multiplicities, size * size..=size * size) {
                let mut edges: Vec<_> = endpoints
                    .clone()
                    .zip(counts)
                    .flat_map(|(edge, count)| std::iter::repeat_n(edge, count))
                    .collect();
                check(size, &edges);
                if !edges.is_empty() {
                    edges.reverse();
                    check(size, &edges);
                }
            }
        }
        let mut rng = Xorshift::default();
        for size in 1..=16 {
            for degree in 0..=16 {
                for _ in 0..10 {
                    let mut edges = Vec::new();
                    for _ in 0..degree {
                        let mut permutation: Vec<_> = (0..size).collect();
                        rng.shuffle(&mut permutation);
                        edges.extend(permutation.into_iter().enumerate());
                    }
                    rng.shuffle(&mut edges);
                    check(size, &edges);
                }
            }
        }
        for size in [20, 64, 128] {
            for degree in [1, 2, 20, 64] {
                for _ in 0..10 {
                    let mut edges = Vec::new();
                    for _ in 0..degree {
                        let mut permutation: Vec<_> = (0..size).collect();
                        rng.shuffle(&mut permutation);
                        edges.extend(permutation.into_iter().enumerate());
                    }
                    rng.shuffle(&mut edges);
                    check(size, &edges);
                }
                let edges: Vec<_> = (0..size)
                    .flat_map(|v| std::iter::repeat_n((v, v), degree))
                    .collect();
                check(size, &edges);
            }
        }
    }
}

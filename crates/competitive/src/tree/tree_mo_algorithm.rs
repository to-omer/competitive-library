/// `vertices; tree, root, paths; |u| add, |u| remove, |i| answer`, or `edges; ...`.
/// `paths` yields `(u, v)`. Callbacks receive vertex or original edge IDs and input query IDs.
/// Start with empty state; `answer` preserves it. The final state is unspecified.
/// Path elements are unordered; answers are visited in Mo order.
#[macro_export]
macro_rules! tree_mo_algorithm {
    (vertices; $tree:expr, $root:expr, $paths:expr;
        |$u:tt| $add:expr, |$v:tt| $remove:expr, |$i:tt| $answer:expr $(,)?) => {{
        let tree = $tree;
        let root: usize = $root;
        let mut trace = Vec::new();
        let tour = tree
            .path_euler_tour_builder(root)
            .build_with_trace(|u| trace.push(u));
        let lca = tree.lca(root);
        let (ranges, extras): (Vec<_>, Vec<_>) = ($paths)
            .into_iter()
            .map(|(mut u, mut v): (usize, usize)| {
                if tour.vidx[u][0] > tour.vidx[v][0] {
                    ::std::mem::swap(&mut u, &mut v);
                }
                let p = lca.lca(u, v);
                if p == u {
                    ((tour.vidx[u][0], tour.vidx[v][0] + 1), !0usize)
                } else {
                    ((tour.vidx[u][1], tour.vidx[v][0] + 1), p)
                }
            })
            .unzip();
        $crate::tree_mo_algorithm!(@run &trace, tour.vidx.len(), &ranges;
            |j| extras[j]; |$u| $add, |$v| $remove, |$i| $answer);
    }};
    (edges; $tree:expr, $root:expr, $paths:expr;
        |$u:tt| $add:expr, |$v:tt| $remove:expr, |$i:tt| $answer:expr $(,)?) => {{
        let tree = $tree;
        let root: usize = $root;
        let tour = tree.path_euler_tour_builder(root).build();
        let lca = tree.lca(root);
        let mut trace = vec![0; 2 * tour.eidx.len()];
        for (e, &[first, after_last]) in tour.eidx.iter().enumerate() {
            trace[first - 1] = e;
            trace[after_last - 2] = e;
        }
        let ranges: Vec<_> = ($paths)
            .into_iter()
            .map(|(mut u, mut v): (usize, usize)| {
                if tour.vidx[u][0] > tour.vidx[v][0] {
                    ::std::mem::swap(&mut u, &mut v);
                }
                if lca.lca(u, v) == u {
                    (tour.vidx[u][0], tour.vidx[v][0])
                } else {
                    (tour.vidx[u][1] - 1, tour.vidx[v][0])
                }
            })
            .collect();
        $crate::tree_mo_algorithm!(@run &trace, tour.eidx.len(), &ranges;
            |_| !0usize; |$u| $add, |$v| $remove, |$i| $answer);
    }};
    (@run $trace:expr, $elements:expr, $ranges:expr;
        |$query:tt| $extra:expr; |$u:tt| $add:expr, |$v:tt| $remove:expr, |$i:tt| $answer:expr) => {{
        let trace: &[usize] = $trace;
        let mut used = vec![false; $elements];
        $crate::mo_algorithm!(
            $ranges,
            (l, r),
            |j| $crate::tree_mo_algorithm!(@toggle trace[j], used; |$u| $add, |$v| $remove),
            |j| $crate::tree_mo_algorithm!(@toggle trace[j], used; |$u| $add, |$v| $remove),
            |query_index| {
                let $query: usize = query_index;
                let p: usize = $extra;
                if p != !0 {
                    let $u = p;
                    $add;
                }
                let $i: usize = query_index;
                $answer;
                if p != !0 {
                    let $v = p;
                    $remove;
                }
            },
        );
    }};
    (@toggle $element:expr, $used:ident;
        |$u:tt| $add:expr, |$v:tt| $remove:expr) => {{
        let element = $element;
        if $used[element] {
            let $v = element;
            $remove;
        } else {
            let $u = element;
            $add;
        }
        $used[element] = !$used[element];
    }};
}

#[cfg(test)]
mod tests {
    use crate::{
        graph::{Graph, UndirectedSparseGraph},
        tools::{Xorshift, testutil::exhaustive_sequences},
        tree::{MixedTree, PathTree, StarTree},
    };

    fn verify(tree: &UndirectedSparseGraph, root: usize) {
        let n = tree.vertices_size();
        let mut expected = Vec::new();
        for s in 0..n {
            let mut parent = vec![n; n];
            let mut edge = vec![n; n];
            let mut queue = std::collections::VecDeque::from([s]);
            parent[s] = s;
            while let Some(u) = queue.pop_front() {
                for a in tree.neighbors(u) {
                    if parent[a.to] == n {
                        parent[a.to] = u;
                        edge[a.to] = a.label;
                        queue.push_back(a.to);
                    }
                }
            }
            for t in 0..n {
                let mut vertices = vec![false; n];
                let mut edges = vec![false; n - 1];
                let mut u = t;
                while u != s {
                    vertices[u] = true;
                    edges[edge[u]] = true;
                    u = parent[u];
                }
                vertices[s] = true;
                expected.push((vertices, edges));
            }
        }
        let mut count = 0;
        tree_mo_algorithm!(vertices; tree, root, std::iter::once((root, root));
            |_| count += 1, |_| count -= 1,
            |_| assert_eq!(count, expected[root * n + root].0.iter().filter(|&&v| v).count()));
        let mut paths: Vec<_> = (0..n).flat_map(|u| (0..n).map(move |v| (u, v))).collect();
        paths.reverse();
        paths.extend_from_within(..n);
        for paths in [&[][..], &paths[..]] {
            let mut present = vec![false; n];
            let mut seen = vec![0; paths.len()];
            tree_mo_algorithm!(vertices; tree, root, paths.iter().copied();
            |u| { assert!(!present[u]); present[u] = true; },
            |u| { assert!(present[u]); present[u] = false; },
            |i| {
                let (s, t) = paths[i];
                assert_eq!(present, expected[s * n + t].0);
                seen[i] += 1;
            });
            assert!(seen.iter().all(|&count| count == 1));
            let mut present = vec![false; n - 1];
            let mut seen = vec![0; paths.len()];
            tree_mo_algorithm!(edges; tree, root, paths.iter().copied();
            |e| { assert!(!present[e]); present[e] = true; },
            |e| { assert!(present[e]); present[e] = false; },
            |i| {
                let (s, t) = paths[i];
                assert_eq!(present, expected[s * n + t].1);
                seen[i] += 1;
            });
            assert!(seen.iter().all(|&count| count == 1));
        }
    }

    #[test]
    fn test_tree_mo_algorithm() {
        for n in 1..=6 {
            for parents in exhaustive_sequences(0..n, n - 1..=n - 1) {
                if !parents.iter().enumerate().all(|(i, _)| {
                    let mut u = i + 1;
                    for _ in 0..n {
                        if u == 0 {
                            return true;
                        }
                        u = parents[u - 1];
                    }
                    false
                }) {
                    continue;
                }
                let tree = UndirectedSparseGraph::from_edges(
                    n,
                    parents
                        .into_iter()
                        .enumerate()
                        .map(|(i, p)| (p, i + 1))
                        .collect(),
                );
                for root in 0..n {
                    verify(&tree, root);
                }
            }
        }
        let mut rng = Xorshift::default();
        for n in 1..=40 {
            for tree in [
                rng.random(PathTree(n)),
                rng.random(StarTree(n)),
                rng.random(MixedTree(n)),
            ] {
                verify(&tree, rng.random(0..n));
            }
        }
    }
}

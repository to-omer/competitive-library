use super::{EdgeListGraph, Field, Invertible, Matrix, Ring};

impl EdgeListGraph {
    /// Nonempty undirected graph; parallel edges are distinct.
    pub fn count_spanning_trees<R>(&self) -> R::T
    where
        R: Field<T: PartialEq, Additive: Invertible, Multiplicative: Invertible>,
    {
        laplacian_cofactor::<R>(self, self.vertices_size() - 1, false).determinant()
    }

    /// Directed away from `root`; parallel edges are distinct.
    pub fn count_spanning_arborescences<R>(&self, root: usize) -> R::T
    where
        R: Field<T: PartialEq, Additive: Invertible, Multiplicative: Invertible>,
    {
        laplacian_cofactor::<R>(self, root, true).determinant()
    }

    /// Directed edge-ID sequences modulo rotation; the empty sequence counts as one.
    pub fn count_eulerian_circuits<R>(&self) -> R::T
    where
        R: Field<T: PartialEq, Additive: Invertible, Multiplicative: Invertible>,
    {
        let n = self.vertices_size();
        let mut indegree = vec![0; n];
        let mut outdegree = vec![0; n];
        for &(u, v) in self.edges() {
            outdegree[u] += 1;
            indegree[v] += 1;
        }
        if indegree != outdegree {
            return R::zero();
        }
        let Some(root) = outdegree.iter().position(|&d| d != 0) else {
            return R::one();
        };
        let mut a = laplacian_cofactor::<R>(self, root, true);
        for (v, &d) in outdegree.iter().enumerate() {
            if d == 0 {
                let v = v - usize::from(v > root);
                a[v][v] = R::one();
            }
        }
        let maximum = *outdegree.iter().max().unwrap();
        let mut factorials = vec![R::one(); maximum];
        let mut integer = R::zero();
        for d in 1..maximum {
            R::add_assign(&mut integer, &R::one());
            factorials[d] = R::mul(&factorials[d - 1], &integer);
        }
        let mut answer = a.determinant();
        for d in outdegree {
            if d != 0 {
                R::mul_assign(&mut answer, &factorials[d - 1]);
            }
        }
        answer
    }
}

fn laplacian_cofactor<R>(graph: &EdgeListGraph, root: usize, directed: bool) -> Matrix<R>
where
    R: Ring<Additive: Invertible>,
{
    let n = graph.vertices_size();
    let mut a = Matrix::<R>::zeros((n - 1, n - 1));
    let one = R::one();
    for &(u, v) in graph.edges() {
        if v != root {
            let vv = v - usize::from(v > root);
            R::add_assign(&mut a[vv][vv], &one);
            if u != root {
                R::sub_assign(&mut a[u - usize::from(u > root)][vv], &one);
            }
        }
        if !directed && u != root {
            let uu = u - usize::from(u > root);
            R::add_assign(&mut a[uu][uu], &one);
            if v != root {
                R::sub_assign(&mut a[v - usize::from(v > root)][uu], &one);
            }
        }
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::AddMulOperation, num::mint_basic::MInt998244353 as M,
        tools::testutil::exhaustive_sequences,
    };

    fn brute_tree(graph: &EdgeListGraph, root: Option<usize>) -> usize {
        let n = graph.vertices_size();
        let mut result = 0;
        for mask in 0usize..1 << graph.edges_size() {
            if mask.count_ones() as usize != n - 1 {
                continue;
            }
            let mut adjacency = vec![vec![]; n];
            let mut indegree = vec![0; n];
            for (eid, &(u, v)) in graph.edges().enumerate() {
                if mask >> eid & 1 != 0 {
                    adjacency[u].push(v);
                    indegree[v] += 1;
                    if root.is_none() {
                        adjacency[v].push(u);
                    }
                }
            }
            if let Some(root) = root
                && (0..n).any(|u| indegree[u] != usize::from(u != root))
            {
                continue;
            }
            let mut reached = vec![false; n];
            let mut stack = vec![root.unwrap_or(0)];
            reached[stack[0]] = true;
            while let Some(u) = stack.pop() {
                for &v in &adjacency[u] {
                    if !reached[v] {
                        reached[v] = true;
                        stack.push(v);
                    }
                }
            }
            result += usize::from(reached.iter().all(|&b| b));
        }
        result
    }

    #[test]
    fn test_count_spanning_trees() {
        for n in 1..=4 {
            let possible: Vec<_> = (0..n)
                .flat_map(|u| (u + 1..n).map(move |v| (u, v)))
                .collect();
            for mask in 0usize..1 << possible.len() {
                let graph = EdgeListGraph::from_edges(
                    n,
                    possible
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &e)| (mask >> i & 1 != 0).then_some(e))
                        .collect(),
                );
                assert_eq!(
                    graph.count_spanning_trees::<AddMulOperation<M>>(),
                    M::from(brute_tree(&graph, None))
                );
            }
        }
        for n in 1..=3 {
            let possible: Vec<_> = (0..n).flat_map(|u| (u..n).map(move |v| (u, v))).collect();
            for counts in exhaustive_sequences(0..3, possible.len()..=possible.len()) {
                let graph = EdgeListGraph::from_edges(
                    n,
                    possible
                        .iter()
                        .zip(counts)
                        .flat_map(|(&edge, count)| std::iter::repeat_n(edge, count))
                        .collect(),
                );
                assert_eq!(
                    graph.count_spanning_trees::<AddMulOperation<M>>(),
                    M::from(brute_tree(&graph, None))
                );
            }
        }
    }

    #[test]
    fn test_count_spanning_arborescences() {
        for n in 1..=3 {
            let possible: Vec<_> = (0..n).flat_map(|u| (0..n).map(move |v| (u, v))).collect();
            let multiplicities = if n <= 2 { 0..3 } else { 0..2 };
            for counts in exhaustive_sequences(multiplicities, possible.len()..=possible.len()) {
                let graph = EdgeListGraph::from_edges(
                    n,
                    possible
                        .iter()
                        .zip(counts)
                        .flat_map(|(&edge, count)| std::iter::repeat_n(edge, count))
                        .collect(),
                );
                for root in 0..n {
                    assert_eq!(
                        graph.count_spanning_arborescences::<AddMulOperation<M>>(root),
                        M::from(brute_tree(&graph, Some(root)))
                    );
                }
            }
        }
    }

    fn brute_euler(edges: &[(usize, usize)]) -> usize {
        if edges.is_empty() {
            return 1;
        }
        fn visit(edges: &[(usize, usize)], used: usize, vertex: usize, start: usize) -> usize {
            if used.count_ones() as usize == edges.len() {
                return usize::from(vertex == start);
            }
            edges
                .iter()
                .enumerate()
                .filter(|&(i, &(u, _))| used >> i & 1 == 0 && u == vertex)
                .map(|(i, &(_, v))| visit(edges, used | 1 << i, v, start))
                .sum()
        }
        visit(edges, 1, edges[0].1, edges[0].0)
    }

    #[test]
    fn test_count_eulerian_circuits() {
        for n in 0..=3 {
            let possible: Vec<_> = (0..n).flat_map(|u| (0..n).map(move |v| (u, v))).collect();
            for mask in 0usize..1 << possible.len() {
                let graph = EdgeListGraph::from_edges(
                    n,
                    possible
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &e)| (mask >> i & 1 != 0).then_some(e))
                        .collect(),
                );
                assert_eq!(
                    graph.count_eulerian_circuits::<AddMulOperation<M>>(),
                    M::from(brute_euler(graph.edges().as_slice()))
                );
            }
        }
        for counts in exhaustive_sequences(0..3, 4..=4) {
            let edges = (0..2)
                .flat_map(|u| (0..2).map(move |v| (u, v)))
                .zip(counts)
                .flat_map(|(edge, count)| std::iter::repeat_n(edge, count))
                .collect();
            let graph = EdgeListGraph::from_edges(2, edges);
            assert_eq!(
                graph.count_eulerian_circuits::<AddMulOperation<M>>(),
                M::from(brute_euler(graph.edges().as_slice()))
            );
        }
    }
}

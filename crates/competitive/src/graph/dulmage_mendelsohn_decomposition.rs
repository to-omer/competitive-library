use super::{BipartiteMatching, DirectedSparseGraph, StronglyConnectedComponent};

pub fn dulmage_mendelsohn_decomposition(
    l: usize,
    r: usize,
    edges: &[(usize, usize)],
) -> Vec<(Vec<usize>, Vec<usize>)> {
    let mut matching = vec![!0usize; l + r];
    let mut medges: Vec<_> = edges.iter().map(|&(u, v)| (u, v + l)).collect();
    for (u, v) in BipartiteMatching::from_edges(l, r, edges).maximum_matching() {
        medges.push((v + l, u));
        matching[u] = v + l;
        matching[v + l] = u;
    }
    let rmedges = medges.iter().map(|&(u, v)| (v, u)).collect();

    let g = DirectedSparseGraph::from_edges(l + r, medges);
    let rg = DirectedSparseGraph::from_edges(l + r, rmedges);
    let scc = StronglyConnectedComponent::new(&g);
    let csize = scc.size();

    let mut cmap = vec![!0usize - 1; csize];
    let mut visited = vec![false; l + r];
    let mut stack = vec![];
    for u in 0..l {
        if matching[u] == !0 && !visited[u] {
            visited[u] = true;
            stack.push(u);
            while let Some(u) = stack.pop() {
                cmap[scc[u]] = !0;
                for a in g.adjacencies(u) {
                    if !visited[a.to] {
                        visited[a.to] = true;
                        stack.push(a.to);
                    }
                }
            }
        }
    }
    for u in l..l + r {
        if matching[u] == !0 && !visited[u] {
            visited[u] = true;
            stack.push(u);
            while let Some(u) = stack.pop() {
                cmap[scc[u]] = 0;
                for a in rg.adjacencies(u) {
                    if !visited[a.to] {
                        visited[a.to] = true;
                        stack.push(a.to);
                    }
                }
            }
        }
    }

    let mut nset = 1usize;
    for v in &mut cmap {
        if *v == !0 - 1 {
            *v = nset;
            nset += 1;
        }
    }
    for v in &mut cmap {
        if *v == !0 {
            *v = nset;
        }
    }
    nset += 1;

    let mut groups = vec![(vec![], vec![]); nset];
    for u in 0..l {
        if matching[u] != !0 {
            let c = cmap[scc[u]];
            groups[c].0.push(u);
            groups[c].1.push(matching[u] - l);
        }
    }
    for u in 0..l {
        if matching[u] == !0 {
            let c = cmap[scc[u]];
            groups[c].0.push(u);
        }
    }
    for u in 0..r {
        if matching[u + l] == !0 {
            let c = cmap[scc[u + l]];
            groups[c].1.push(u);
        }
    }
    groups
}

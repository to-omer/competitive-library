use competitive::prelude::*;
use competitive::tree::XorLinkedRootedTree;

#[verify::library_checker("tree_diameter")]
pub fn tree_diameter(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, edges: [(u32, u32, u32); n - 1]);
    let mut parent = vec![n as u32; n];
    let mut farthest: Vec<_> = (0..n).map(|u| (0, u)).collect();
    let (mut diameter, mut left, mut right, mut center) = (0, 0, 0, 0);
    XorLinkedRootedTree::builder(n).with_eindexed().run(
        0,
        edges.iter().map(|&(u, v, _)| (u as usize, v as usize)),
        |u, p, e| {
            parent[u] = p as u32;
            let distance = farthest[u].0 + edges[e].2 as u64;
            if diameter < distance + farthest[p].0 {
                diameter = distance + farthest[p].0;
                left = farthest[u].1;
                right = farthest[p].1;
                center = p;
            }
            if farthest[p].0 < distance {
                farthest[p] = (distance, farthest[u].1);
            }
        },
    );
    let mut path = Vec::new();
    while left != center {
        path.push(left);
        left = parent[left] as usize;
    }
    path.push(center);
    let middle = path.len();
    while right != center {
        path.push(right);
        right = parent[right] as usize;
    }
    path[middle..].reverse();
    pp!(diameter, path.len(); @it path);
}

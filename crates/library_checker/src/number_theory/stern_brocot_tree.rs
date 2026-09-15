use competitive::prelude::*;
use competitive::{
    algorithm::{SbtNode, SbtPath, SternBrocotTree},
    num::URational,
};

competitive::define_enum_scan! {
    enum Query: raw {
        "ENCODE_PATH" => EncodePath { a: u32, b: u32 }
        "DECODE_PATH" => DecodePath { k: usize, path: [(char, u32); k] }
        "LCA" => Lca { a: u32, b: u32, c: u32, d: u32 }
        "ANCESTOR" => Ancestor { k: u32, a: u32, b: u32 }
        "RANGE" => Range { a: u32, b: u32 }
    }
}

#[verify::library_checker("stern_brocot_tree")]
pub fn stern_brocot_tree(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(t);
    for _ in 0..t {
        sc!(query: Query);
        match query {
            Query::EncodePath { a, b } => {
                let path = SbtPath::from(URational::new(a, b));
                let len = if path.path.first() == Some(&0) {
                    path.path.len() - 1
                } else {
                    path.path.len()
                };
                pp!(len, !);
                for (i, count) in path.into_iter().enumerate() {
                    if count == 0 {
                        continue;
                    }
                    if i % 2 == 0 {
                        pp!(@ns " R ", count, !);
                    } else {
                        pp!(@ns " L ", count, !);
                    }
                }
                pp!();
            }
            Query::DecodePath { path, .. } => {
                let node: SbtNode<u32> = if path.first().is_some_and(|t| t.0 == 'L') {
                    [0].into_iter()
                        .chain(path.into_iter().map(|(_, c)| c))
                        .collect()
                } else {
                    path.into_iter().map(|(_, c)| c).collect()
                };
                let val = node.eval();
                pp!(val.num, val.den);
            }
            Query::Lca { a, b, c, d } => {
                let path1 = SbtPath::from(URational::new(a, b));
                let path2 = SbtPath::from(URational::new(c, d));
                let val = SbtNode::lca(path1, path2).eval();
                pp!(val.num, val.den);
            }
            Query::Ancestor { k, a, b } => {
                let mut path = SbtPath::from(URational::new(a, b));
                let depth = path.depth();
                if k <= depth {
                    path.up(depth - k);
                    let val = path.eval();
                    pp!(val.num, val.den);
                } else {
                    pp!("-1");
                }
            }
            Query::Range { a, b } => {
                let node = SbtPath::from(URational::new(a, b)).to_node();
                pp!(node.l.num, node.l.den, node.r.num, node.r.den);
            }
        }
    }
}

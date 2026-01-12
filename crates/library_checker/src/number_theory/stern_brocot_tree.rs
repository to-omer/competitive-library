use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
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
pub fn stern_brocot_tree(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, t);
    for _ in 0..t {
        scan!(scanner, query: Query);
        match query {
            Query::EncodePath { a, b } => {
                let path = SbtPath::from(URational::new(a, b));
                let len = if path.path.first() == Some(&0) {
                    path.path.len() - 1
                } else {
                    path.path.len()
                };
                write!(writer, "{}", len).ok();
                for (i, count) in path.into_iter().enumerate() {
                    if count == 0 {
                        continue;
                    }
                    if i % 2 == 0 {
                        write!(writer, " R {}", count).ok();
                    } else {
                        write!(writer, " L {}", count).ok();
                    }
                }
                writeln!(writer).ok();
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
                writeln!(writer, "{} {}", val.num, val.den).ok();
            }
            Query::Lca { a, b, c, d } => {
                let path1 = SbtPath::from(URational::new(a, b));
                let path2 = SbtPath::from(URational::new(c, d));
                let val = SbtNode::lca(path1, path2).eval();
                writeln!(writer, "{} {}", val.num, val.den).ok();
            }
            Query::Ancestor { k, a, b } => {
                let mut path = SbtPath::from(URational::new(a, b));
                let depth = path.depth();
                if k <= depth {
                    path.up(depth - k);
                    let val = path.eval();
                    writeln!(writer, "{} {}", val.num, val.den).ok();
                } else {
                    writeln!(writer, "-1").ok();
                }
            }
            Query::Range { a, b } => {
                let node = SbtPath::from(URational::new(a, b)).to_node();
                writeln!(
                    writer,
                    "{} {} {} {}",
                    node.l.num, node.l.den, node.r.num, node.r.den
                )
                .ok();
            }
        }
    }
}

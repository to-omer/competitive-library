use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algorithm::{SbtNode, SbtPath, SternBrocotTree},
    num::Rational,
};

#[verify::library_checker("stern_brocot_tree")]
pub fn stern_brocot_tree(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, t);
    for _ in 0..t {
        scan!(scanner, type_: String);
        match type_.as_str() {
            "ENCODE_PATH" => {
                scan!(scanner, a: i32, b: i32);
                let path = SbtPath::from(Rational::new(a, b));
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
            "DECODE_PATH" => {
                scan!(scanner, k, path: [(char, i32); k]);
                let node: SbtNode<i32> = if path.first().is_some_and(|t| t.0 == 'L') {
                    [0].into_iter()
                        .chain(path.into_iter().map(|(_, c)| c))
                        .collect()
                } else {
                    path.into_iter().map(|(_, c)| c).collect()
                };
                let val = node.eval();
                writeln!(writer, "{} {}", val.num, val.den).ok();
            }
            "LCA" => {
                scan!(scanner, [a, b, c, d]: [i32; const 4]);
                let path1 = SbtPath::from(Rational::new(a, b));
                let path2 = SbtPath::from(Rational::new(c, d));
                let val = SbtNode::lca(path1, path2).eval();
                writeln!(writer, "{} {}", val.num, val.den).ok();
            }
            "ANCESTOR" => {
                scan!(scanner, [k, a, b]: [i32; const 3]);
                let mut path = SbtPath::from(Rational::new(a, b));
                let depth = path.depth();
                if k <= depth {
                    path.up(depth - k);
                    let val = path.eval();
                    writeln!(writer, "{} {}", val.num, val.den).ok();
                } else {
                    writeln!(writer, "-1").ok();
                }
            }
            "RANGE" => {
                scan!(scanner, [a, b]: [i32; const 2]);
                let node = SbtPath::from(Rational::new(a, b)).to_node();
                writeln!(
                    writer,
                    "{} {} {} {}",
                    node.l.num, node.l.den, node.r.num, node.r.den
                )
                .ok();
            }
            _ => unreachable!(),
        }
    }
}

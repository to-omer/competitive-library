pub use crate::graph::TwoSatisfiability;
use crate::prelude::*;

#[verify_attr::verify("https://judge.yosupo.jp/problem/two_sat")]
pub fn two_sat(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(
        scanner,
        _p: String,
        _cnf: String,
        n,
        m,
        ab: [(i64, i64, i64)]
    );
    let mut two_sat = TwoSatisfiability::new(n);
    for (a, b, _) in ab.take(m) {
        let u = (a.abs() as usize - 1) * 2;
        let v = (b.abs() as usize - 1) * 2;
        let na = (a < 0) as usize;
        let nb = (b < 0) as usize;
        two_sat.add_inner(u ^ na ^ 1, v ^ nb);
    }
    if let Some(v) = two_sat.two_satisfiability() {
        writeln!(writer, "s SATISFIABLE").ok();
        write!(writer, "v").ok();
        for (i, v) in v.into_iter().enumerate() {
            write!(
                writer,
                " {}",
                if v { i as i64 + 1 } else { -(i as i64 + 1) }
            )
            .ok();
        }
        write!(writer, " 0").ok();
    } else {
        writeln!(writer, "s UNSATISFIABLE").ok();
    }
}

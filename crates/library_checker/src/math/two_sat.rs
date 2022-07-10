#[doc(no_inline)]
pub use competitive::graph::TwoSatisfiability;
use competitive::prelude::*;

#[verify::library_checker("two_sat")]
pub fn two_sat(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(
        scanner,
        _p: String,
        _cnf: String,
        n,
        m,
        ab: [(i32, i32, i32)]
    );
    let mut two_sat = TwoSatisfiability::new(n);
    for (a, b, _) in ab.take(m) {
        two_sat.add_clause(a.abs() as usize - 1, a >= 0, b.abs() as usize - 1, b >= 0);
    }
    if let Some(v) = two_sat.two_satisfiability() {
        writeln!(writer, "s SATISFIABLE").ok();
        write!(writer, "v").ok();
        for (i, v) in v.into_iter().enumerate() {
            write!(
                writer,
                " {}",
                if v { i as i32 + 1 } else { -(i as i32 + 1) }
            )
            .ok();
        }
        write!(writer, " 0").ok();
    } else {
        writeln!(writer, "s UNSATISFIABLE").ok();
    }
}

use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::AdditiveOperation, data_structure::PotentializedUnionFind,
    num::montgomery::MInt998244353,
};

#[verify::library_checker("unionfind_with_potential")]
pub fn unionfind_with_potential(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut uf = PotentializedUnionFind::<AdditiveOperation<MInt998244353>>::new(n);
    for _ in 0..q {
        scan!(scanner, t: u8);
        if t == 0 {
            scan!(scanner, u, v, x: MInt998244353);
            if let Some(diff) = uf.difference(u, v) {
                writeln!(writer, "{}", (diff == x) as u8).ok();
            } else {
                uf.unite_with(u, v, x);
                writeln!(writer, "1").ok();
            }
        } else {
            scan!(scanner, u, v);
            if let Some(diff) = uf.difference(u, v) {
                writeln!(writer, "{}", diff).ok();
            } else {
                writeln!(writer, "-1").ok();
            }
        }
    }
}

use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::{Associative, Invertible, Magma, Unital},
    data_structure::PotentializedUnionFind,
    define_monoid,
    num::{One, Zero, montgomery::MInt998244353},
};

competitive::define_enum_scan! {
    enum Query: u8 {
        0 => Unite { u: usize, v: usize, x: [[MInt998244353; const 2]; const 2] }
        1 => Diff { u: usize, v: usize }
    }
}

define_monoid!(
    Sl2,
    [[MInt998244353; 2]; 2],
    |a, b| {
        let [[a00, a01], [a10, a11]] = a;
        let [[b00, b01], [b10, b11]] = b;
        [
            [a00 * b00 + a01 * b10, a00 * b01 + a01 * b11],
            [a10 * b00 + a11 * b10, a10 * b01 + a11 * b11],
        ]
    },
    [
        [MInt998244353::one(), MInt998244353::zero()],
        [MInt998244353::zero(), MInt998244353::one()]
    ]
);

impl Invertible for Sl2 {
    fn inverse(x: &Self::T) -> Self::T {
        [[x[1][1], -x[0][1]], [-x[1][0], x[0][0]]]
    }
}

#[verify::library_checker("unionfind_with_potential_non_commutative_group")]
pub fn unionfind_with_potential_non_commutative_group(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut uf = PotentializedUnionFind::<Sl2>::new(n);
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Unite { u, v, x } => {
                if let Some(diff) = uf.difference(v, u) {
                    writeln!(writer, "{}", (diff == x) as u8).ok();
                } else {
                    uf.unite_with(v, u, x);
                    writeln!(writer, "1").ok();
                }
            }
            Query::Diff { u, v } => {
                if let Some(diff) = uf.difference(v, u) {
                    iter_print!(writer, @it diff.into_iter().flatten());
                } else {
                    writeln!(writer, "-1").ok();
                }
            }
        }
    }
}

use competitive::{
    algebra::{AddMulOperation, AdditiveOperation, ArrayOperation},
    math::QuotientArray,
    num::mint_basic::MInt998244353,
};
use competitive::{num::One, prelude::*};

type M = MInt998244353;

#[verify::library_checker("sum_of_totient_function")]
pub fn sum_of_totient_function(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n: u64);
    let mut s = 1;
    let mut pp = 0;
    let mut pc = 0;
    let inv2 = M::new(2).inv();
    let qa = QuotientArray::from_fn(n, |i| [M::from(i), M::from(i) * M::from(i + 1) * inv2])
        .map(|[x, y]| [x - M::one(), y - M::one()])
        .lucy_dp::<ArrayOperation<AdditiveOperation<_>, 2>>(|[x, y], p| [x, y * M::from(p)])
        .map(|[x, y]| y - x)
        .min_25_sieve::<AddMulOperation<_>>(|p, c| {
            if pp != p || pc > c {
                pp = p;
                pc = 1;
                s = p - 1;
            }
            while pc < c {
                pc += 1;
                s *= p;
            }
            M::from(s)
        });
    pp!(qa[n]);
}

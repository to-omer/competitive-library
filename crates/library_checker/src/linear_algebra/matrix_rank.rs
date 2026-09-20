use competitive::prelude::*;
use competitive::{algebra::AddMulOperation, math::Matrix, num::mint_basic::MInt998244353};

#[verify::library_checker("matrix_rank")]
pub fn matrix_rank(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m);
    let mut a = if n <= m {
        sc!(a: [[MInt998244353; m]; n]);
        Matrix::<AddMulOperation<_>>::from_vec(a)
    } else {
        let mut a = Matrix::<AddMulOperation<_>>::zeros((m, n));
        for j in 0..n {
            for row in &mut a.data {
                sc!(x: MInt998244353);
                row[j] = x;
            }
        }
        a
    };
    let rank = a.rank();
    pp!(rank);
}

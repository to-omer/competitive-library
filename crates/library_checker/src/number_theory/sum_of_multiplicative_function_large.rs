use competitive::algebra::{AddMulOperation, AdditiveOperation, ArrayOperation};
use competitive::math::QuotientArray;
use competitive::num::{One, montgomery::MInt469762049 as M};
use competitive::prelude::*;

#[verify::library_checker(
    "sum_of_multiplicative_function_large",
    include("QuotientArray", "ArrayOperation", "montgomery")
)]
pub fn sum_of_multiplicative_function_large(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(t, cases: [(u64, M, M); iter t]);
    let half = M::from(2u32).inv();
    for (n, a, b) in cases {
        let primes = QuotientArray::from_fn(n, |x| {
            let x = M::from(x);
            [x, x * (x + M::one()) * half]
        })
        .lucy_dp::<ArrayOperation<AdditiveOperation<_>, 2>>(|[_, p], [count, sum]| {
            [*count, *p * *sum]
        });
        let sum = primes
            .map(|[count, sum]| a * *count + b * *sum)
            .multiplicative_sum::<AddMulOperation<_>>(|p, e| a * M::from(e) + b * M::from(p));
        pp!(sum);
    }
}

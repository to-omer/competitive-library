use super::{prime_factors, BarrettReduction};

pub fn primitive_root(p: u64) -> u64 {
    if p == 2 {
        return 1;
    }
    let pf = prime_factors(p - 1);
    let br = BarrettReduction::<u128>::new(p as _);
    let mut g = 2;
    loop {
        if pf.iter().all(|&(q, _)| {
            let mut g = g as u128;
            let mut k = (p - 1) / q;
            let mut r: u128 = 1;
            while k > 0 {
                if k & 1 == 1 {
                    r = br.rem(r * g);
                }
                g = br.rem(g * g);
                k >>= 1;
            }
            r != 1
        }) {
            return g;
        }
        g += 1;
    }
}

#[test]
fn test_primitive_root() {
    assert_eq!(3, primitive_root(998244353));
}

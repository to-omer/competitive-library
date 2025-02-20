use super::{Convolve, ConvolveSteps, Zero, montgomery};

pub fn wildcard_pattern_matching(p: &[u8], s: &[u8]) -> Vec<bool> {
    type M = montgomery::MInt2013265921;
    let n = p.len();
    let m = s.len();
    assert!(n >= 1);
    assert!(m >= 1);
    let mut sum = vec![M::zero(); m - n + 1];
    macro_rules! add {
        ($f:expr; $g:expr;) => {{
            let x: Vec<M> = p.iter().map($f).rev().collect();
            let y: Vec<M> = s.iter().map($g).collect();
            let z = Convolve::<montgomery::Modulo2013265921>::convolve(x, y);
            for i in 0..=m - n {
                sum[i] += z[n + i - 1];
            }
        }};
    }
    add!(
        |&x: &u8| M::from((x != b'?') as u32 * x as u32 * x as u32);
        |&x: &u8| M::from((x != b'?') as u32);
    );
    add!(
        |&x: &u8| -M::from((x != b'?') as u32 * x as u32 * 2);
        |&x: &u8| M::from((x != b'?') as u32 * x as u32);
    );
    add!(
        |&x: &u8| M::from((x != b'?') as u32);
        |&x: &u8| M::from((x != b'?') as u32 * x as u32 * x as u32);
    );
    sum.into_iter().map(|s| s.is_zero()).collect()
}

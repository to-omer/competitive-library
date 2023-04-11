use super::Monoid;
use std::{collections::HashSet, hash::Hash};

/// $\min\{0\le i < n | x^i=y\}$
pub fn baby_step_giant_step<M>(x: M::T, y: M::T, n: usize) -> Option<usize>
where
    M: Monoid,
    M::T: Eq + Hash,
{
    if M::is_unit(&y) {
        return Some(0);
    }
    let block_size = 1usize.max((n as f64).sqrt() as _);
    let mut baby = HashSet::new();
    let mut t = y.clone();
    for _ in 0..block_size {
        t = M::operate(&x, &t);
        baby.insert(t.clone());
    }
    let g = M::pow(x.clone(), block_size);
    let mut t = M::unit();
    let mut fail = 0usize;
    for k in (0..n).step_by(block_size) {
        let nt = M::operate(&g, &t);
        if baby.contains(&nt) {
            for m in k..n.min(k + block_size) {
                if t == y {
                    return Some(m);
                }
                t = M::operate(&x, &t);
            }
            fail += 1;
            if fail >= 2 {
                break;
            }
        }
        t = nt;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::MultiplicativeOperation as MulOp, num::mint_basic::DynMIntU32, tools::Xorshift,
    };

    #[test]
    fn test_bsgs_small() {
        for n in 2..50 {
            DynMIntU32::set_mod(n);
            for x in 0..n {
                for y in 0..n {
                    let (x, y) = (DynMIntU32::new(x), DynMIntU32::new(y));
                    let exp = (0..n).position(|i| x.pow(i as _) == y);
                    let ans = baby_step_giant_step::<MulOp<DynMIntU32>>(x, y, n as _);
                    assert_eq!(exp, ans);
                }
            }
        }
    }

    #[test]
    fn test_bsgs_midium() {
        let mut rng = Xorshift::new();
        for _ in 0..10 {
            let n = rng.gen(2..100_000u32);
            DynMIntU32::set_mod(n);
            let x = DynMIntU32::new(rng.gen(..n));
            let y = DynMIntU32::new(rng.gen(..n));
            let exp = (0..n).position(|i| x.pow(i as _) == y);
            let ans = baby_step_giant_step::<MulOp<DynMIntU32>>(x, y, n as _);
            assert_eq!(exp, ans);
        }
    }

    #[test]
    fn test_bsgs_large() {
        let mut rng = Xorshift::new();
        for _ in 0..20 {
            let n = rng.gen(2..1_000_000_000u32);
            DynMIntU32::set_mod(n);
            let x = DynMIntU32::new(rng.gen(..n));
            let y = DynMIntU32::new(rng.gen(..n));
            let ans = baby_step_giant_step::<MulOp<DynMIntU32>>(x, y, n as _);
            if let Some(i) = ans {
                assert_eq!(x.pow(i), y);
                assert!(i < n as usize);
            }
        }
    }
}

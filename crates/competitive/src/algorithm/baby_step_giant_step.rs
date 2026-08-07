use super::Monoid;
use std::{collections::HashSet, hash::Hash};

/// $\min\{0\le i < n | a x^i=b\}$
pub fn baby_step_giant_step<M>(a: M::T, x: M::T, b: M::T, n: usize) -> Option<usize>
where
    M: Monoid<T: Eq + Hash>,
{
    if a == b {
        return Some(0);
    }
    let block_size = 1usize.max((n as f64).sqrt() as _);
    let mut baby = HashSet::new();
    let mut t = b.clone();
    for _ in 0..block_size {
        t = M::operate(&t, &x);
        baby.insert(t.clone());
    }
    let g = M::pow(x.clone(), block_size);
    let mut t = a;
    let mut fail = 0usize;
    for k in (0..n).step_by(block_size) {
        let nt = M::operate(&t, &g);
        if baby.contains(&nt) {
            for m in k..n.min(k + block_size) {
                if t == b {
                    return Some(m);
                }
                t = M::operate(&t, &x);
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
        for n in 2..30 {
            DynMIntU32::set_mod(n);
            for a in 0..n {
                for x in 0..n {
                    for b in 0..n {
                        let (a, x, b) =
                            (DynMIntU32::new(a), DynMIntU32::new(x), DynMIntU32::new(b));
                        let exp = (0..n).position(|i| a * x.pow(i as _) == b);
                        let ans = baby_step_giant_step::<MulOp<DynMIntU32>>(a, x, b, n as _);
                        assert_eq!(exp, ans);
                    }
                }
            }
        }
    }

    #[test]
    fn test_bsgs_midium() {
        let mut rng = Xorshift::default();
        for _ in 0..10 {
            let n = rng.random(2..100_000u32);
            DynMIntU32::set_mod(n);
            let a = DynMIntU32::new(rng.random(..n));
            let x = DynMIntU32::new(rng.random(..n));
            let b = DynMIntU32::new(rng.random(..n));
            let exp = (0..n).position(|i| a * x.pow(i as _) == b);
            let ans = baby_step_giant_step::<MulOp<DynMIntU32>>(a, x, b, n as _);
            assert_eq!(exp, ans);
        }
    }

    #[test]
    fn test_bsgs_large() {
        let mut rng = Xorshift::default();
        for _ in 0..20 {
            let n = rng.random(2..1_000_000_000u32);
            DynMIntU32::set_mod(n);
            let a = DynMIntU32::new(rng.random(..n));
            let x = DynMIntU32::new(rng.random(..n));
            let b = DynMIntU32::new(rng.random(..n));
            let ans = baby_step_giant_step::<MulOp<DynMIntU32>>(a, x, b, n as _);
            if let Some(i) = ans {
                assert_eq!(a * x.pow(i), b);
                assert!(i < n as usize);
            }
        }
    }
}

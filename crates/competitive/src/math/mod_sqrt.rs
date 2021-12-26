use crate::num::{MInt, MIntConvert, One, Zero};

#[cfg_attr(nightly, codesnip::entry("mod_sqrt", include("MInt")))]
impl<M: MIntConvert<u32>> MInt<M> {
    pub fn sqrt(self) -> Option<Self> {
        fn jacobi<M: MIntConvert<u32>>(mut x: u32) -> i8 {
            let mut s = 1i8;
            let mut m = M::mod_into();
            while m > 1 {
                x %= m;
                if x == 0 {
                    return 0;
                }
                let k = x.trailing_zeros();
                if k % 2 == 1 && (m + 2) & 4 != 0 {
                    s = -s;
                }
                x >>= k;
                if x & m & 2 != 0 {
                    s = -s;
                }
                std::mem::swap(&mut x, &mut m);
            }
            s
        }
        if M::mod_into() == 2 {
            return Some(self);
        }
        let j = jacobi::<M>(u32::from(self));
        match j.cmp(&0) {
            std::cmp::Ordering::Less => {
                return None;
            }
            std::cmp::Ordering::Equal => {
                return Some(Self::zero());
            }
            std::cmp::Ordering::Greater => {}
        }
        let mut r = 1;
        let (mut f0, d) = loop {
            r ^= r << 5;
            r ^= r >> 17;
            r ^= r << 11;
            let b = Self::from(r);
            let d = b * b - self;
            if jacobi::<M>(u32::from(d)) == -1 {
                break (b, d);
            }
        };
        let (mut f1, mut g0, mut g1, mut e) = (
            Self::one(),
            Self::one(),
            Self::zero(),
            (M::mod_into() + 1) / 2,
        );
        while e > 0 {
            if e % 2 == 1 {
                let t = g0 * f0 + d * g1 * f1;
                g1 = g0 * f1 + g1 * f0;
                g0 = t;
            }
            let t = f0 * f0 + d * f1 * f1;
            f1 = Self::from(2) * f0 * f1;
            f0 = t;
            e /= 2;
        }
        if u32::from(g0) > M::mod_into() - u32::from(g0) {
            g0 = -g0;
        }
        Some(g0)
    }
}

use crate::{
    num::{MInt, Modulus, One, Zero},
    tools::Xorshift,
};

#[codesnip::entry("mod_sqrt", include("MInt", "Xorshift"))]
impl<M: Modulus> MInt<M> {
    pub fn sqrt(self) -> Option<Self> {
        fn jacobi<M: Modulus>(mut x: u32) -> i8 {
            let mut s = 1i8;
            let mut m = M::get_modulus();
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
        if M::get_modulus() == 2 {
            return Some(self);
        }
        let j = jacobi::<M>(self.inner());
        match j.cmp(&0) {
            std::cmp::Ordering::Less => {
                return None;
            }
            std::cmp::Ordering::Equal => {
                return Some(Self::zero());
            }
            std::cmp::Ordering::Greater => {}
        }
        let mut rand = Xorshift::default();
        let (mut f0, d) = loop {
            let b = Self::new_unchecked(rand.rand(M::get_modulus() as u64) as u32);
            let d = b * b - self;
            if jacobi::<M>(d.inner()) == -1 {
                break (b, d);
            }
        };
        let (mut f1, mut g0, mut g1, mut e) = (
            Self::one(),
            Self::one(),
            Self::zero(),
            (M::get_modulus() + 1) / 2,
        );
        while e > 0 {
            if e % 2 == 1 {
                let t = g0 * f0 + d * g1 * f1;
                g1 = g0 * f1 + g1 * f0;
                g0 = t;
            }
            let t = f0 * f0 + d * f1 * f1;
            f1 = Self::new_unchecked(2) * f0 * f1;
            f0 = t;
            e /= 2;
        }
        if g0.inner() > M::get_modulus() - g0.inner() {
            g0 = -g0;
        }
        Some(g0)
    }
}

use cargo_snippet::snippet;

#[snippet("algebra")]
pub trait Magma {
    type T: Clone + PartialEq;
    fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T;
}
#[snippet("algebra")]
pub trait Associative {}

#[snippet("algebra")]
pub trait SemiGroup: Magma + Associative {}

#[snippet("algebra")]
pub trait Unital: Magma {
    fn unit(&self) -> Self::T;
}

#[snippet("algebra")]
pub trait Monoid: SemiGroup + Unital {
    fn power(&self, x: Self::T, n: usize) -> Self::T {
        let mut n = n;
        let mut res = self.unit();
        let mut base = x;
        while n > 0 {
            if n & 1 == 1 {
                res = self.operate(&res, &base);
            }
            base = self.operate(&base, &base);
            n = n >> 1;
        }
        res
    }
}

#[snippet("algebra")]
pub trait Invertible: Magma {
    fn inverse(&self, x: &Self::T) -> Self::T;
}

#[snippet("algebra")]
pub trait Group: Monoid + Invertible {}

#[snippet("algebra")]
pub trait Commutative {}

#[snippet("algebra")]
pub trait AbelianMonoid: Monoid + Commutative {}

#[snippet("algebra")]
pub trait AbelianGroup: Group + Commutative {}

#[snippet("algebra")]
pub trait Idempotent {}

#[snippet("algebra")]
pub trait IdempotentMonoid: Monoid + Idempotent {}

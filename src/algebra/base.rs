#[cargo_snippet::snippet("algebra")]
pub trait Magma {
    type T: Clone + PartialEq;
    fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T;
}
#[cargo_snippet::snippet("algebra")]
pub trait Associative {}

#[cargo_snippet::snippet("algebra")]
pub trait SemiGroup: Magma + Associative {}

#[cargo_snippet::snippet("algebra")]
pub trait Unital: Magma {
    fn unit(&self) -> Self::T;
}

#[cargo_snippet::snippet("algebra")]
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

#[cargo_snippet::snippet("algebra")]
pub trait Invertible: Magma {
    fn inverse(&self, x: &Self::T) -> Self::T;
}

#[cargo_snippet::snippet("algebra")]
pub trait Group: Monoid + Invertible {}

#[cargo_snippet::snippet("algebra")]
pub trait Commutative {}

#[cargo_snippet::snippet("algebra")]
pub trait AbelianMonoid: Monoid + Commutative {}

#[cargo_snippet::snippet("algebra")]
pub trait AbelianGroup: Group + Commutative {}

#[cargo_snippet::snippet("algebra")]
pub trait Idempotent {}

#[cargo_snippet::snippet("algebra")]
pub trait IdempotentMonoid: Monoid + Idempotent {}

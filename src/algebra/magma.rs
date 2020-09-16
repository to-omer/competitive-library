//! algebraic traits

/// binary operaion: $T \circ T \to T$
#[cargo_snippet::snippet("algebra")]
pub trait Magma {
    /// type of operands: $T$
    type T: Clone + PartialEq;
    /// binary operaion: $\circ$
    fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T;
    #[inline]
    fn reverse_operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
        self.operate(y, x)
    }
}

/// $\forall a,\forall b,\forall c \in T, (a \circ b) \circ c = a \circ (b \circ c)$
#[cargo_snippet::snippet("algebra")]
pub trait Associative {}

/// associative binary operation
#[cargo_snippet::snippet("algebra")]
pub trait SemiGroup: Magma + Associative {}

#[cargo_snippet::snippet("algebra")]
impl<S: Magma + Associative> SemiGroup for S {}

/// $\exists e \in T, \forall a \in T, e \circ a = a \circ e = e$
#[cargo_snippet::snippet("algebra")]
pub trait Unital: Magma {
    /// identity element: $e$
    fn unit(&self) -> Self::T;
}

/// associative binary operation and an identity element
#[cargo_snippet::snippet("algebra")]
pub trait Monoid: SemiGroup + Unital {
    /// binary exponentiation: $x^n = x\circ\ddots\circ x$
    fn pow(&self, x: Self::T, n: usize) -> Self::T {
        let mut n = n;
        let mut res = self.unit();
        let mut base = x;
        while n > 0 {
            if n & 1 == 1 {
                res = self.operate(&res, &base);
            }
            base = self.operate(&base, &base);
            n >>= 1;
        }
        res
    }
}

#[cargo_snippet::snippet("algebra")]
impl<M: SemiGroup + Unital> Monoid for M {}

/// $\exists e \in T, \forall a \in T, \exists b,c \in T, b \circ a = a \circ c = e$
#[cargo_snippet::snippet("algebra")]
pub trait Invertible: Magma {
    /// $a$ where $a \circ x = e$
    fn inverse(&self, x: &Self::T) -> Self::T;
    #[inline]
    fn rinv_operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
        self.operate(x, &self.inverse(y))
    }
}

/// associative binary operation and an identity element and inverse elements
#[cargo_snippet::snippet("algebra")]
pub trait Group: Monoid + Invertible {}

#[cargo_snippet::snippet("algebra")]
impl<G: Monoid + Invertible> Group for G {}

/// $\forall a,\forall b \in T, a \circ b = b \circ a$
#[cargo_snippet::snippet("algebra")]
pub trait Commutative {}

/// commutative monoid
#[cargo_snippet::snippet("algebra")]
pub trait AbelianMonoid: Monoid + Commutative {}

#[cargo_snippet::snippet("algebra")]
impl<M: Monoid + Commutative> AbelianMonoid for M {}

/// commutative group
#[cargo_snippet::snippet("algebra")]
pub trait AbelianGroup: Group + Commutative {}

#[cargo_snippet::snippet("algebra")]
impl<G: Group + Commutative> AbelianGroup for G {}

/// $\forall a \in T, a \circ a = a$
#[cargo_snippet::snippet("algebra")]
pub trait Idempotent {}

/// idempotent monoid
#[cargo_snippet::snippet("algebra")]
pub trait IdempotentMonoid: Monoid + Idempotent {}

#[cargo_snippet::snippet("algebra")]
impl<M: Monoid + Idempotent> IdempotentMonoid for M {}

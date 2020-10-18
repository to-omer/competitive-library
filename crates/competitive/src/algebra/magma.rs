//! algebraic traits

/// binary operaion: $T \circ T \to T$
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
pub trait Associative {}

/// associative binary operation
pub trait SemiGroup: Magma + Associative {}

impl<S: Magma + Associative> SemiGroup for S {}

/// $\exists e \in T, \forall a \in T, e \circ a = a \circ e = e$
pub trait Unital: Magma {
    /// identity element: $e$
    fn unit(&self) -> Self::T;
}

/// associative binary operation and an identity element
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

impl<M: SemiGroup + Unital> Monoid for M {}

/// $\exists e \in T, \forall a \in T, \exists b,c \in T, b \circ a = a \circ c = e$
pub trait Invertible: Magma {
    /// $a$ where $a \circ x = e$
    fn inverse(&self, x: &Self::T) -> Self::T;
    #[inline]
    fn rinv_operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
        self.operate(x, &self.inverse(y))
    }
}

/// associative binary operation and an identity element and inverse elements
pub trait Group: Monoid + Invertible {}

impl<G: Monoid + Invertible> Group for G {}

/// $\forall a,\forall b \in T, a \circ b = b \circ a$
pub trait Commutative {}

/// commutative monoid
pub trait AbelianMonoid: Monoid + Commutative {}

impl<M: Monoid + Commutative> AbelianMonoid for M {}

/// commutative group
pub trait AbelianGroup: Group + Commutative {}

impl<G: Group + Commutative> AbelianGroup for G {}

/// $\forall a \in T, a \circ a = a$
pub trait Idempotent {}

/// idempotent monoid
pub trait IdempotentMonoid: Monoid + Idempotent {}

impl<M: Monoid + Idempotent> IdempotentMonoid for M {}

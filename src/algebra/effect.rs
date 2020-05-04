use super::base::*;

pub trait MonoidEffect: Monoid {
    type A;
    fn effect(&self, a: &Self::A, x: &Self::T) -> Self::A;
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AnyMonoidEffect<M: Monoid, T, F: Fn(&T, &M::T) -> T> {
    m: M,
    f: F,
    phantom: std::marker::PhantomData<T>,
}
pub mod any_monoid_effect_impl {
    use super::*;
    impl<M: Monoid, T, F: Fn(&T, &M::T) -> T> AnyMonoidEffect<M, T, F> {
        pub fn new(m: M, f: F) -> Self {
            AnyMonoidEffect {
                m: m,
                f: f,
                phantom: std::marker::PhantomData,
            }
        }
    }
    impl<M: Monoid, T, F: Fn(&T, &M::T) -> T> Magma for AnyMonoidEffect<M, T, F> {
        type T = M::T;
        #[inline]
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            self.m.operate(x, y)
        }
    }
    impl<M: Monoid, T, F: Fn(&T, &M::T) -> T> Unital for AnyMonoidEffect<M, T, F> {
        #[inline]
        fn unit(&self) -> Self::T {
            self.m.unit()
        }
    }
    impl<M: Monoid, T, F: Fn(&T, &M::T) -> T> Associative for AnyMonoidEffect<M, T, F> {}
    impl<M: Monoid, T, F: Fn(&T, &M::T) -> T> SemiGroup for AnyMonoidEffect<M, T, F> {}
    impl<M: Monoid, T, F: Fn(&T, &M::T) -> T> Monoid for AnyMonoidEffect<M, T, F> {}
    impl<M: Monoid, T, F: Fn(&T, &M::T) -> T> Commutative for AnyMonoidEffect<M, T, F> {}
    impl<M: Monoid, T, F: Fn(&T, &M::T) -> T> AbelianMonoid for AnyMonoidEffect<M, T, F> {}
    impl<M: Monoid, T, F: Fn(&T, &M::T) -> T> Idempotent for AnyMonoidEffect<M, T, F> {}
    impl<M: Monoid, T, F: Fn(&T, &M::T) -> T> IdempotentMonoid for AnyMonoidEffect<M, T, F> {}
    impl<M: Monoid, T, F: Fn(&T, &M::T) -> T> MonoidEffect for AnyMonoidEffect<M, T, F> {
        type A = T;
        fn effect(&self, a: &Self::A, x: &Self::T) -> Self::A {
            (self.f)(a, x)
        }
    }
}

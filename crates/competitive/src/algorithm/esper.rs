#![allow(clippy::type_complexity)]

use super::{Field, Invertible, Matrix};
use std::{collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData};

type Marker<T> = PhantomData<fn() -> T>;

#[derive(Debug, Clone)]
pub struct EsperEstimator<R, Input, Class, FC, FF>
where
    R: Field,
    R::Additive: Invertible,
    R::Multiplicative: Invertible,
    Class: Eq + Hash,
    FC: Fn(&Input) -> Class,
    FF: Fn(&Input) -> Vec<R::T>,
{
    class: FC,
    feature: FF,
    data: HashMap<Class, (Vec<Vec<R::T>>, Vec<R::T>)>,
    _marker: Marker<(R::T, Input, Class)>,
}

#[derive(Debug, Clone)]
pub struct EsperSolver<R, Input, Class, FC, FF>
where
    R: Field,
    R::Additive: Invertible,
    R::Multiplicative: Invertible,
    Class: Eq + Hash,
    FC: Fn(&Input) -> Class,
    FF: Fn(&Input) -> Vec<R::T>,
{
    class: FC,
    feature: FF,
    data: HashMap<Class, Option<Vec<R::T>>>,
    _marker: Marker<(R::T, Input, Class)>,
}

impl<R, Input, Class, FC, FF> EsperEstimator<R, Input, Class, FC, FF>
where
    R: Field,
    R::Additive: Invertible,
    R::Multiplicative: Invertible,
    Class: Eq + Hash,
    FC: Fn(&Input) -> Class,
    FF: Fn(&Input) -> Vec<R::T>,
{
    pub fn new(class: FC, feature: FF) -> Self {
        Self {
            class,
            feature,
            data: Default::default(),
            _marker: PhantomData,
        }
    }

    pub fn push(&mut self, input: Input, output: R::T) {
        let class = (self.class)(&input);
        let feature = (self.feature)(&input);
        let entry = self.data.entry(class).or_default();
        entry.0.push(feature);
        entry.1.push(output);
    }
}

impl<R, Input, Class, FC, FF> EsperEstimator<R, Input, Class, FC, FF>
where
    R: Field,
    R::Additive: Invertible,
    R::Multiplicative: Invertible,
    R::T: PartialEq,
    Class: Eq + Hash,
    FC: Fn(&Input) -> Class,
    FF: Fn(&Input) -> Vec<R::T>,
{
    pub fn solve(self) -> EsperSolver<R, Input, Class, FC, FF> {
        let data: HashMap<_, _> = self
            .data
            .into_iter()
            .map(|(key, (a, b))| {
                (
                    key,
                    Matrix::<R>::from_vec(a)
                        .solve_system_of_linear_equations(&b)
                        .map(|sol| sol.particular),
                )
            })
            .collect();
        EsperSolver {
            class: self.class,
            feature: self.feature,
            data,
            _marker: PhantomData,
        }
    }

    pub fn solve_checked(self) -> EsperSolver<R, Input, Class, FC, FF>
    where
        Class: Debug,
        R::T: Debug,
    {
        let data: HashMap<_, _> = self
            .data
            .into_iter()
            .map(|(key, (a, b))| {
                let mat = Matrix::<R>::from_vec(a);
                let coeff = mat
                    .solve_system_of_linear_equations(&b)
                    .map(|sol| sol.particular);
                if coeff.is_none() {
                    eprintln!(
                        "failed to solve linear equations: key={:?} A={:?} b={:?}",
                        key, &mat.data, &b
                    );
                }
                (key, coeff)
            })
            .collect();
        EsperSolver {
            class: self.class,
            feature: self.feature,
            data,
            _marker: PhantomData,
        }
    }
}

impl<R, Input, Class, FC, FF> EsperSolver<R, Input, Class, FC, FF>
where
    R: Field,
    R::Additive: Invertible,
    R::Multiplicative: Invertible,
    Class: Eq + Hash,
    FC: Fn(&Input) -> Class,
    FF: Fn(&Input) -> Vec<R::T>,
{
    pub fn solve(&self, input: Input) -> R::T {
        let coeff = self
            .data
            .get(&(self.class)(&input))
            .expect("unrecognized class")
            .as_ref()
            .expect("failed to solve");
        let feature = (self.feature)(&input);
        feature
            .into_iter()
            .zip(coeff)
            .map(|(x, y)| R::mul(&x, y))
            .fold(R::zero(), |x, y| R::add(&x, &y))
    }
}

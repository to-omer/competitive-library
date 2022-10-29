use super::{Matrix, One, Zero};
use std::{
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    ops::{Add, Div, Mul, Sub},
};

type Marker<T> = PhantomData<fn() -> T>;

#[derive(Debug, Clone)]
pub struct EsperEstimator<T, Input, Class, FC, FF>
where
    Class: Eq + Hash,
    FC: Fn(&Input) -> Class,
    FF: Fn(&Input) -> Vec<T>,
{
    class: FC,
    feature: FF,
    data: HashMap<Class, (Vec<Vec<T>>, Vec<T>)>,
    _marker: Marker<(T, Input, Class)>,
}

#[derive(Debug, Clone)]
pub struct EsperSolver<T, Input, Class, FC, FF>
where
    Class: Eq + Hash,
    FC: Fn(&Input) -> Class,
    FF: Fn(&Input) -> Vec<T>,
{
    class: FC,
    feature: FF,
    data: HashMap<Class, Option<Vec<T>>>,
    _marker: Marker<(T, Input, Class)>,
}

impl<T, Input, Class, FC, FF> EsperEstimator<T, Input, Class, FC, FF>
where
    Class: Eq + Hash,
    FC: Fn(&Input) -> Class,
    FF: Fn(&Input) -> Vec<T>,
{
    pub fn new(class: FC, feature: FF) -> Self {
        Self {
            class,
            feature,
            data: Default::default(),
            _marker: PhantomData,
        }
    }

    pub fn push(&mut self, input: Input, output: T) {
        let class = (self.class)(&input);
        let feature = (self.feature)(&input);
        let entry = self.data.entry(class).or_default();
        entry.0.push(feature);
        entry.1.push(output);
    }
}

impl<T, Input, Class, FC, FF> EsperEstimator<T, Input, Class, FC, FF>
where
    Class: Eq + Hash,
    T: Copy + PartialEq + Zero + One + Sub<Output = T> + Mul<Output = T> + Div<Output = T>,
    FC: Fn(&Input) -> Class,
    FF: Fn(&Input) -> Vec<T>,
{
    pub fn solve(self) -> EsperSolver<T, Input, Class, FC, FF> {
        let data: HashMap<_, _> = self
            .data
            .into_iter()
            .map(|(key, (a, b))| {
                (
                    key,
                    Matrix::from_vec(a).solve_system_of_linear_equations(&b),
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

    pub fn solve_checked(self) -> EsperSolver<T, Input, Class, FC, FF>
    where
        Class: Debug,
        T: Debug,
    {
        let data: HashMap<_, _> = self
            .data
            .into_iter()
            .map(|(key, (a, b))| {
                let mat = Matrix::from_vec(a);
                let coeff = mat.solve_system_of_linear_equations(&b);
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

impl<T, Input, Class, FC, FF> EsperSolver<T, Input, Class, FC, FF>
where
    Class: Eq + Hash,
    T: Copy + Zero + Add<Output = T> + Mul<Output = T>,
    FC: Fn(&Input) -> Class,
    FF: Fn(&Input) -> Vec<T>,
{
    pub fn solve(&self, input: Input) -> T {
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
            .map(|(x, &y)| x * y)
            .fold(T::zero(), |x, y| x + y)
    }
}

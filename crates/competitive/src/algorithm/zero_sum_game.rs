use std::{collections::HashMap, hash::Hash, marker::PhantomData};

    type Marker<T> = PhantomData<fn() -> T>;

    pub trait ZeroSumGame {
        type State;
        type Iter: Iterator<Item = Result<i64, Self::State>>;
        fn next_state(&mut self, s: &Self::State) -> Self::Iter;
    }

    pub struct ZeroSumGamer<S, F, I>
    where
        F: FnMut(&S) -> I,
        I: Iterator<Item = Result<i64, S>>,
    {
        f: F,
        _marker: Marker<(S, I)>,
    }

    impl<S, F, I> ZeroSumGamer<S, F, I>
    where
        F: FnMut(&S) -> I,
        I: Iterator<Item = Result<i64, S>>,
    {
        pub fn new(f: F) -> Self {
            Self {
                f,
                _marker: PhantomData,
            }
        }
    }

    impl<S, F, I> ZeroSumGame for ZeroSumGamer<S, F, I>
    where
        F: FnMut(&S) -> I,
        I: Iterator<Item = Result<i64, S>>,
    {
        type State = S;
        type Iter = I;
        fn next_state(&mut self, s: &Self::State) -> Self::Iter {
            (self.f)(s)
        }
    }

    #[derive(Debug, Clone)]
    pub struct ZeroSumGameAnalyzer<G>
    where
        G: ZeroSumGame,
        G::State: Eq + Hash,
    {
        game: G,
        scores: HashMap<G::State, i64>,
    }

    impl<G> ZeroSumGameAnalyzer<G>
    where
        G: ZeroSumGame,
        G::State: Eq + Hash + Clone,
    {
        pub fn new(game: G) -> Self {
            Self {
                game,
                scores: Default::default(),
            }
        }
        pub fn eval(&mut self, s: &G::State) -> i64 {
            if let Some(score) = self.scores.get(s).cloned() {
                score
            } else {
                let score = self
                    .game
                    .next_state(s)
                    .map(|ns| ns.unwrap_or_else(|ns| -self.eval(&ns)))
                    .max()
                    .unwrap();
                self.scores.insert(s.clone(), score);
                score
            }
        }
    }

use super::{
    ControlFlow, EvaluatedMove, GameAnalysis, GameAnalyzer, Hash, HashMap, HashSet, Neighbor,
    memoized_game,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Turn {
    Same,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreMove<S, L = ()> {
    pub next: Neighbor<S, L>,
    pub reward: i64,
    pub turn: Turn,
}

pub struct Scoring<S, F, T> {
    moves: F,
    terminal: T,
    scores: HashMap<S, i64>,
}

impl GameAnalyzer {
    /// Scores and rewards are measured for the player whose turn it is.
    /// `terminal` returns `Some(score)` at terminal states and `None` elsewhere.
    pub fn scoring<S, L, F, T, I>(self, moves: F, terminal: T) -> GameAnalyzer<Scoring<S, F, T>>
    where
        F: FnMut(&S) -> I,
        I: IntoIterator<Item = ScoreMove<S, L>>,
        T: FnMut(&S) -> Option<i64>,
    {
        GameAnalyzer {
            game: Scoring {
                moves,
                terminal,
                scores: HashMap::new(),
            },
        }
    }
}

impl<S, F, T> GameAnalyzer<Scoring<S, F, T>> {
    pub fn clear_cache(&mut self) {
        self.game.scores.clear();
    }
}
impl<S, L, F, T, I> GameAnalyzer<Scoring<S, F, T>>
where
    S: Clone + Eq + Hash,
    F: FnMut(&S) -> I,
    I: IntoIterator<Item = ScoreMove<S, L>>,
    T: FnMut(&S) -> Option<i64>,
{
    pub fn score(&mut self) -> GameAnalysis<'_, Scoring<S, F, T>, S, i64> {
        GameAnalysis::new(&mut self.game, |g, state| {
            memoized_game(
                state,
                &mut g.scores,
                &mut HashSet::new(),
                &mut |s| {
                    if let Some(value) = (g.terminal)(s) {
                        return ControlFlow::Break(value);
                    }
                    let moves = (g.moves)(s)
                        .into_iter()
                        .map(|m| (m.next.to, (m.reward, m.turn)));
                    ControlFlow::Continue((moves, None))
                },
                &mut |_, moves| moves.next(),
                &mut |best: &mut Option<i64>, (reward, turn), child| {
                    let value = match turn {
                        Turn::Same => reward + child,
                        Turn::Other => reward - child,
                    };
                    *best = Some(best.map_or(value, |b| b.max(value)));
                    ControlFlow::Continue(())
                },
                &mut |_, best| best.expect("nonterminal scoring state has no moves"),
            )
        })
    }
}
impl<S, L, F, T, I> GameAnalysis<'_, Scoring<S, F, T>, S, i64>
where
    F: FnMut(&S) -> I,
    I: IntoIterator<Item = ScoreMove<S, L>>,
    T: FnMut(&S) -> Option<i64>,
{
    /// Values include the reward and are measured for the current player.
    pub fn moves(&mut self, state: &S) -> Vec<EvaluatedMove<S, L, i64>> {
        self.evaluated_moves(state).collect()
    }

    pub fn best_moves(&mut self, state: &S) -> Vec<Neighbor<S, L>> {
        let mut result = Vec::new();
        let mut best = None;
        for m in self.evaluated_moves(state) {
            if best.is_none_or(|best| m.value > best) {
                result.clear();
                best = Some(m.value);
            }
            if best == Some(m.value) {
                result.push(m.next);
            }
        }
        result
    }

    fn evaluated_moves(&mut self, state: &S) -> impl Iterator<Item = EvaluatedMove<S, L, i64>> {
        let terminal = (self.game.terminal)(state).is_some();
        let mut moves = (!terminal)
            .then(|| (self.game.moves)(state).into_iter())
            .into_iter()
            .flatten()
            .peekable();
        assert!(
            terminal || moves.peek().is_some(),
            "nonterminal scoring state has no moves"
        );
        moves.map(|m| {
            let child = self.eval(&m.next.to);
            let value = match m.turn {
                Turn::Same => m.reward + child,
                Turn::Other => m.reward - child,
            };
            EvaluatedMove {
                next: m.next,
                value,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn minimax_and_moves() {
        let mut rng = Xorshift::default();
        let mut edges = vec![Vec::new(); 64];
        for (i, moves) in edges.iter_mut().enumerate().skip(1) {
            for to in 0..i {
                if to == 0 || rng.rand(4) == 0 {
                    moves.push(ScoreMove {
                        next: Neighbor::new(to, to),
                        reward: rng.rand(21) as i64 - 10,
                        turn: if rng.rand(2) == 0 {
                            Turn::Same
                        } else {
                            Turn::Other
                        },
                    });
                }
            }
            if i % 2 == 0 {
                // Duplicate every option so any optimum has distinct labeled ties.
                let extra: Vec<_> = moves
                    .iter()
                    .map(|m| ScoreMove {
                        next: Neighbor::new(m.next.to, m.next.label + i),
                        ..*m
                    })
                    .collect();
                moves.extend(extra);
            }
        }
        let terminal: Vec<_> = (0..edges.len())
            .map(|s| {
                if s == 0 {
                    Some(-7)
                } else {
                    (s == 1 || s > 2 && rng.rand(4) == 0).then(|| rng.rand(21) as i64 - 10)
                }
            })
            .collect();
        let mut game =
            GameAnalyzer::new().scoring(|&s: &usize| edges[s].clone(), |&s: &usize| terminal[s]);
        // Fixed-player minimax also tracks whose turn it is, instead of negamax.
        let mut scores = vec![[0i64; 2]; edges.len()];
        for i in 0..edges.len() {
            if let Some(value) = terminal[i] {
                scores[i] = [value, -value];
                continue;
            }
            for p in 0..2 {
                let candidates = edges[i].iter().map(|m| {
                    let next_player = p ^ usize::from(m.turn == Turn::Other);
                    scores[m.next.to][next_player] + if p == 0 { m.reward } else { -m.reward }
                });
                scores[i][p] = if p == 0 {
                    candidates.max()
                } else {
                    candidates.min()
                }
                .unwrap();
            }
        }
        assert_eq!(game.score().verify(0..edges.len(), |&s| scores[s][0]), None);
        for (s, expected) in scores.iter().enumerate() {
            let best = game.score().best_moves(&s);
            assert_eq!(best.is_empty(), terminal[s].is_some());
            let moves: Vec<_> = edges[s]
                .iter()
                .filter(|_| terminal[s].is_none())
                .map(|m| EvaluatedMove {
                    next: m.next,
                    value: m.reward + scores[m.next.to][usize::from(m.turn == Turn::Other)],
                })
                .collect();
            assert_eq!(game.score().moves(&s), moves.clone());
            assert_eq!(
                best,
                moves
                    .iter()
                    .filter(|m| m.value == expected[0])
                    .map(|m| m.next)
                    .collect::<Vec<_>>()
            );
        }
    }
}

use super::{
    EvaluatedMove, GameAnalysis, GameAnalyzer, GameOutcome, GameSum, GameValue, Graph, HashMap,
    HashSet, Misere, Neighbor, TopologicalSortExt, UsizeGraph, VertexMap, game_mex,
};
use std::{cmp::Reverse, collections::VecDeque};

pub struct ImpartialGraph<'g, G>
where
    G: Graph + VertexMap<Option<usize>>,
{
    graph: &'g G,
    index: Option<GraphIndex<G>>,
    grundy: Vec<Option<u64>>,
    outcomes: [GraphOutcomes; 2],
    generalized_grundy: Vec<Option<(u64, usize)>>,
    sums: GraphSums,
}

struct GraphIndex<G>
where
    G: Graph + VertexMap<Option<usize>>,
{
    indices: G::Vmap,
    edges: Vec<Vec<usize>>,
    reverse: Vec<Vec<usize>>,
}

#[derive(Default)]
struct GraphSums {
    indices: HashMap<Vec<usize>, usize>,
    outcomes: GraphOutcomes,
}

#[derive(Default)]
struct GraphOutcomes {
    values: Vec<GameOutcome>,
    winning: Vec<Option<usize>>,
}

impl GameAnalyzer {
    /// Uses the graph's shared moves for both players. Query vertices must be
    /// valid inputs to the graph's `VertexMap` operations.
    pub fn impartial_graph<G>(self, graph: &G) -> GameAnalyzer<ImpartialGraph<'_, G>>
    where
        G: Graph + VertexMap<Option<usize>>,
    {
        GameAnalyzer {
            game: ImpartialGraph {
                graph,
                index: None,
                grundy: Vec::new(),
                outcomes: std::array::from_fn(|_| GraphOutcomes::default()),
                generalized_grundy: Vec::new(),
                sums: GraphSums::default(),
            },
        }
    }
}

impl<'g, G> GameAnalyzer<ImpartialGraph<'g, G>>
where
    G: Graph + VertexMap<Option<usize>>,
{
    pub fn grundy(&mut self) -> GameAnalysis<'_, ImpartialGraph<'g, G>, G::Vertex, u64> {
        GameAnalysis::new(&mut self.game, ImpartialGraph::grundy)
    }

    pub fn value(&mut self) -> GameAnalysis<'_, ImpartialGraph<'g, G>, G::Vertex, GameValue> {
        GameAnalysis::new(&mut self.game, |g, s| GameValue::nimber(g.grundy(s)))
    }

    pub fn outcome(&mut self) -> GameAnalysis<'_, ImpartialGraph<'g, G>, G::Vertex, GameOutcome> {
        GameAnalysis::new(&mut self.game, |g, s| g.outcome(s, false))
    }

    pub fn misere(&mut self) -> Misere<'_, ImpartialGraph<'g, G>> {
        Misere {
            game: &mut self.game,
        }
    }

    pub fn sum(&mut self) -> GameSum<'_, ImpartialGraph<'g, G>> {
        GameSum {
            game: &mut self.game,
        }
    }

    pub fn clear_cache(&mut self) {
        self.game.grundy.clear();
        self.game.outcomes = std::array::from_fn(|_| GraphOutcomes::default());
        self.game.generalized_grundy.clear();
        self.game.sums = GraphSums::default();
    }
}

impl<'a, 'g, G> Misere<'a, ImpartialGraph<'g, G>>
where
    G: Graph + VertexMap<Option<usize>>,
{
    pub fn outcome(self) -> GameAnalysis<'a, ImpartialGraph<'g, G>, G::Vertex, GameOutcome, true> {
        GameAnalysis::new(self.game, |g, s| g.outcome(s, true))
    }
}
impl<'a, 'g, G> GameSum<'a, ImpartialGraph<'g, G>>
where
    G: Graph + VertexMap<Option<usize>>,
{
    pub fn grundy(self) -> GameAnalysis<'a, ImpartialGraph<'g, G>, [G::Vertex], u64> {
        GameAnalysis::new(self.game, ImpartialGraph::sum_grundy)
    }
}
impl<'a, 'g, G, const MISERE: bool> GameSum<'a, ImpartialGraph<'g, G>, MISERE>
where
    G: Graph + VertexMap<Option<usize>>,
{
    /// Normal play combines generalized Grundy values, including cyclic positions.
    /// Misere explores the sum graph, which can grow exponentially in the component count.
    pub fn outcome(
        self,
    ) -> GameAnalysis<'a, ImpartialGraph<'g, G>, [G::Vertex], GameOutcome, MISERE> {
        GameAnalysis::new(self.game, |g, s| g.sum_outcome(s, MISERE))
    }
}

impl<'g, S, L, G, V, const MISERE: bool, E> GameAnalysis<'_, ImpartialGraph<'g, G>, S, V, MISERE, E>
where
    S: Copy,
    G: Graph<Vertex = S, Label = L> + VertexMap<Option<usize>>,
    E: FnMut(&mut ImpartialGraph<'g, G>, &S) -> V,
{
    /// Values describe the destination position; outcomes are for the next player.
    pub fn moves(&mut self, state: &S) -> Vec<EvaluatedMove<S, L, V>> {
        self.game.prepare();
        self.game.index_of(*state);
        self.game
            .graph
            .neighbors(*state)
            .map(|next| {
                let value = self.eval(&next.to);
                EvaluatedMove { next, value }
            })
            .collect()
    }
}
impl<S, L, G> GameAnalysis<'_, ImpartialGraph<'_, G>, S, u64>
where
    S: Copy,
    G: Graph<Vertex = S, Label = L> + VertexMap<Option<usize>>,
{
    pub fn winning_moves(&mut self, state: &S) -> Vec<Neighbor<S, L>> {
        self.game.prepare();
        self.game.index_of(*state);
        self.game
            .graph
            .neighbors(*state)
            .filter(|next| self.eval(&next.to) == 0)
            .collect()
    }
}
impl<S, L, G> GameAnalysis<'_, ImpartialGraph<'_, G>, S, GameValue>
where
    S: Copy,
    G: Graph<Vertex = S, Label = L> + VertexMap<Option<usize>>,
{
    pub fn winning_moves(&mut self, state: &S) -> Vec<Neighbor<S, L>> {
        self.game.prepare();
        self.game.index_of(*state);
        self.game
            .graph
            .neighbors(*state)
            .filter(|next| self.eval(&next.to).as_nimber() == Some(0))
            .collect()
    }
}
impl<S, L, G, const MISERE: bool> GameAnalysis<'_, ImpartialGraph<'_, G>, S, GameOutcome, MISERE>
where
    S: Copy,
    G: Graph<Vertex = S, Label = L> + VertexMap<Option<usize>>,
{
    /// Moves to losing positions. To avoid cycling among winning positions,
    /// repeatedly use `best_move`, which follows the retrograde proof.
    pub fn winning_moves(&mut self, state: &S) -> Vec<Neighbor<S, L>> {
        self.game.prepare();
        self.game.index_of(*state);
        self.game
            .graph
            .neighbors(*state)
            .filter(|next| self.eval(&next.to) == GameOutcome::Lose)
            .collect()
    }

    /// A move that guarantees termination when winning, or preserves a draw.
    /// Returns `None` at terminal or losing positions.
    pub fn best_move(&mut self, state: &S) -> Option<Neighbor<S, L>> {
        let outcome = self.eval(state);
        if outcome == GameOutcome::Lose {
            return None;
        }
        let game = &self.game;
        let root = game.index_of(*state);
        let result = &game.outcomes[usize::from(MISERE)];
        let winner = result.winning[root];
        for next in game.graph.neighbors(*state) {
            let id = game.index_of(next.to);
            if Some(id) == winner
                || outcome == GameOutcome::Draw && result.values[id] == GameOutcome::Draw
            {
                return Some(next);
            }
        }
        None
    }
}

#[allow(clippy::type_complexity)] // Sum states and labels both carry component information.
impl<'g, S, L, G, V, const MISERE: bool, E>
    GameAnalysis<'_, ImpartialGraph<'g, G>, [S], V, MISERE, E>
where
    S: Copy,
    G: Graph<Vertex = S, Label = L> + VertexMap<Option<usize>>,
    E: FnMut(&mut ImpartialGraph<'g, G>, &[S]) -> V,
{
    pub fn moves(&mut self, state: &[S]) -> Vec<EvaluatedMove<Vec<S>, (usize, L), V>> {
        self.eval(state);
        let mut result = Vec::new();
        for (i, &component) in state.iter().enumerate() {
            for next in self.game.graph.neighbors(component) {
                let mut to = state.to_vec();
                to[i] = next.to;
                let value = self.eval(&to);
                result.push(EvaluatedMove {
                    next: Neighbor::new(to, (i, next.label)),
                    value,
                });
            }
        }
        result
    }
}
#[allow(clippy::type_complexity)] // Sum states and labels both carry component information.
impl<S, L, G> GameAnalysis<'_, ImpartialGraph<'_, G>, [S], u64>
where
    S: Copy,
    G: Graph<Vertex = S, Label = L> + VertexMap<Option<usize>>,
{
    pub fn winning_moves(&mut self, state: &[S]) -> Vec<Neighbor<Vec<S>, (usize, L)>> {
        let xor = self.eval(state);
        let mut result = Vec::new();
        if xor == 0 {
            return result;
        }
        for (i, component) in state.iter().enumerate() {
            let target = xor ^ self.game.grundy(component);
            for next in self.game.graph.neighbors(*component) {
                if self.game.grundy(&next.to) == target {
                    let mut to = state.to_vec();
                    to[i] = next.to;
                    result.push(Neighbor::new(to, (i, next.label)));
                }
            }
        }
        result
    }
}
#[allow(clippy::type_complexity)] // Sum states and labels both carry component information.
impl<S, L, G, const MISERE: bool> GameAnalysis<'_, ImpartialGraph<'_, G>, [S], GameOutcome, MISERE>
where
    S: Copy,
    G: Graph<Vertex = S, Label = L> + VertexMap<Option<usize>>,
{
    /// Moves to losing positions; use `best_move` to avoid cycling.
    pub fn winning_moves(&mut self, state: &[S]) -> Vec<Neighbor<Vec<S>, (usize, L)>> {
        if state.is_empty() {
            return Vec::new();
        }
        if let [single] = state {
            let mut query: GameAnalysis<'_, _, S, GameOutcome, MISERE> =
                GameAnalysis::new(self.game, |g: &mut ImpartialGraph<'_, G>, s| {
                    g.outcome(s, MISERE)
                });
            return query
                .winning_moves(single)
                .into_iter()
                .map(|next| Neighbor::new(vec![next.to], (0, next.label)))
                .collect();
        }
        if self.eval(state) != GameOutcome::Win {
            return Vec::new();
        }
        let mut indices = self.game.sum_indices(state);
        let game = &self.game;
        let mut xor = 0;
        let mut infinite = 0;
        if !MISERE {
            for &v in &indices {
                if let Some((value, _)) = game.generalized_grundy[v] {
                    xor ^= value;
                } else {
                    infinite += 1;
                }
            }
        }
        let mut result = Vec::new();
        for (i, &component) in state.iter().enumerate() {
            let source = indices[i];
            for next in game.graph.neighbors(component) {
                indices[i] = game.index_of(next.to);
                let wins = if MISERE {
                    let key =
                        GraphSums::canonical(indices.clone(), &game.index.as_ref().unwrap().edges);
                    game.sums.outcomes.values[game.sums.indices[&key]] == GameOutcome::Lose
                } else if let Some((value, _)) = game.generalized_grundy[indices[i]] {
                    match game.generalized_grundy[source] {
                        Some((g, _)) => infinite == 0 && xor ^ g == value,
                        None => infinite == 1 && xor == value,
                    }
                } else {
                    false
                };
                if wins {
                    let mut to = state.to_vec();
                    to[i] = next.to;
                    result.push(Neighbor::new(to, (i, next.label)));
                }
            }
            indices[i] = source;
        }
        result
    }

    /// A move that guarantees termination when winning, or preserves a draw.
    /// Returns `None` at terminal or losing positions.
    pub fn best_move(&mut self, state: &[S]) -> Option<Neighbor<Vec<S>, (usize, L)>> {
        if state.is_empty() {
            return None;
        }
        if let [single] = state {
            let mut query: GameAnalysis<'_, _, S, GameOutcome, MISERE> =
                GameAnalysis::new(self.game, |g: &mut ImpartialGraph<'_, G>, s| {
                    g.outcome(s, MISERE)
                });
            return query
                .best_move(single)
                .map(|next| Neighbor::new(vec![next.to], (0, next.label)));
        }
        let outcome = self.eval(state);
        if outcome == GameOutcome::Lose {
            return None;
        }
        let mut indices = self.game.sum_indices(state);
        let game = &self.game;
        if MISERE {
            let key = GraphSums::canonical(indices.clone(), &game.index.as_ref().unwrap().edges);
            let winner = game.sums.outcomes.winning[game.sums.indices[&key]];
            for (i, &component) in state.iter().enumerate() {
                let original = indices[i];
                for next in game.graph.neighbors(component) {
                    indices[i] = game.index_of(next.to);
                    let key =
                        GraphSums::canonical(indices.clone(), &game.index.as_ref().unwrap().edges);
                    let id = game.sums.indices[&key];
                    if Some(id) == winner
                        || outcome == GameOutcome::Draw
                            && game.sums.outcomes.values[id] == GameOutcome::Draw
                    {
                        let mut to = state.to_vec();
                        to[i] = next.to;
                        return Some(Neighbor::new(to, (i, next.label)));
                    }
                }
                indices[i] = original;
            }
            return None;
        }
        let mut xor = 0;
        let mut infinite = None;
        let mut multiple = false;
        for (i, &v) in indices.iter().enumerate() {
            if let Some((value, _)) = game.generalized_grundy[v] {
                xor ^= value;
            } else if infinite.replace(i).is_some() {
                multiple = true;
                break;
            }
        }
        if outcome == GameOutcome::Draw {
            let i = infinite.unwrap();
            let v = indices[i];
            let edges = &game.index.as_ref().unwrap().edges;
            let target = (!multiple).then(|| {
                let values = edges[v]
                    .iter()
                    .filter_map(|&u| game.generalized_grundy[u].map(|(g, _)| g))
                    .collect();
                game_mex(values)
            });
            // Preserve two infinite components, or keep the finite XOR outside
            // the remaining infinite component's finite follower values.
            if target.is_none_or(|g| g == xor) {
                let to = edges[v]
                    .iter()
                    .copied()
                    .find(|&u| {
                        game.generalized_grundy[u].is_none()
                            && target.is_none_or(|g| {
                                !edges[u].iter().any(|&w| {
                                    game.generalized_grundy[w].is_some_and(|(value, _)| value == g)
                                })
                            })
                    })
                    .unwrap();
                let next = game
                    .graph
                    .neighbors(state[i])
                    .find(|next| game.index_of(next.to) == to)
                    .unwrap();
                let mut to = state.to_vec();
                to[i] = next.to;
                return Some(Neighbor::new(to, (i, next.label)));
            }
            // The current finite XOR exceeds this mex, so a finite component
            // can decrease it to the mex without moving the infinite component.
            xor ^= target.unwrap();
            infinite = None;
        }
        // Reduce the largest (Grundy, counter) pair, then minimize the
        // destination counter so repeated winning moves terminate.
        let (i, target) = if let Some(i) = infinite {
            (i, xor)
        } else {
            let (i, (g, _)) = indices
                .iter()
                .enumerate()
                .filter_map(|(i, &v)| game.generalized_grundy[v].map(|g| (i, g)))
                .filter(|(_, (g, _))| g ^ xor < *g)
                .max_by_key(|&(i, g)| (g, Reverse(i)))?;
            (i, g ^ xor)
        };
        let next = game
            .graph
            .neighbors(state[i])
            .filter_map(|next| {
                let (g, counter) = game.generalized_grundy[game.index_of(next.to)]?;
                (g == target).then_some((next, counter))
            })
            .min_by_key(|&(_, counter)| counter)?
            .0;
        let mut to = state.to_vec();
        to[i] = next.to;
        Some(Neighbor::new(to, (i, next.label)))
    }
}

impl<G> ImpartialGraph<'_, G>
where
    G: Graph + VertexMap<Option<usize>>,
{
    fn grundy(&mut self, state: &G::Vertex) -> u64 {
        self.prepare();
        let root = self.index_of(*state);
        let index = self.index.as_ref().unwrap();
        self.grundy.resize(index.edges.len(), None);
        if let Some(value) = self.grundy[root] {
            return value;
        }
        let mut active = HashSet::from([root]);
        let mut stack = vec![(root, index.edges[root].iter())];
        while let Some((v, moves)) = stack.last_mut() {
            if let Some(&u) = moves.next() {
                if self.grundy[u].is_none() {
                    assert!(active.insert(u), "cycle in a finite game");
                    stack.push((u, index.edges[u].iter()));
                }
            } else {
                let values = index.edges[*v]
                    .iter()
                    .map(|&u| self.grundy[u].unwrap())
                    .collect();
                self.grundy[*v] = Some(game_mex(values));
                active.remove(v);
                stack.pop();
            }
        }
        self.grundy[root].unwrap()
    }

    fn outcome(&mut self, state: &G::Vertex, misere: bool) -> GameOutcome {
        self.prepare();
        let index = self.index.as_ref().unwrap();
        let outcome = &mut self.outcomes[usize::from(misere)];
        if outcome.values.is_empty() {
            outcome.extend(&index.edges, &index.reverse, misere);
        }
        self.outcomes[usize::from(misere)].values[self.index_of(*state)]
    }

    fn sum_grundy(&mut self, state: &[G::Vertex]) -> u64 {
        state.iter().fold(0, |value, s| value ^ self.grundy(s))
    }

    fn sum_outcome(&mut self, state: &[G::Vertex], misere: bool) -> GameOutcome {
        match state {
            [] => return GameOutcome::from_win(misere),
            [single] => return self.outcome(single, misere),
            _ => {}
        }
        let indices = self.sum_indices(state);
        if !misere {
            self.solve_generalized_grundy();
            return self.sum_normal_outcome(&indices);
        }
        let edges = &self.index.as_ref().unwrap().edges;
        let root = self.sums.prepare(indices, edges);
        self.sums.outcomes.values[root]
    }

    fn sum_normal_outcome(&self, state: &[usize]) -> GameOutcome {
        let mut xor = 0;
        let mut infinite = None;
        for &v in state {
            if let Some((value, _)) = self.generalized_grundy[v] {
                xor ^= value;
            } else if infinite.replace(v).is_some() {
                return GameOutcome::Draw;
            }
        }
        if let Some(v) = infinite {
            let can_win = self.index.as_ref().unwrap().edges[v]
                .iter()
                .any(|&u| self.generalized_grundy[u].is_some_and(|(value, _)| value == xor));
            if can_win {
                GameOutcome::Win
            } else {
                GameOutcome::Draw
            }
        } else if xor == 0 {
            GameOutcome::Lose
        } else {
            GameOutcome::Win
        }
    }

    fn sum_indices(&mut self, state: &[G::Vertex]) -> Vec<usize> {
        self.prepare();
        state.iter().map(|&s| self.index_of(s)).collect()
    }

    fn prepare(&mut self) {
        if self.index.is_some() {
            return;
        }
        let vertices: Vec<_> = self.graph.vertices().collect();
        let mut indices = self.graph.construct_vmap(|| None);
        for (i, &v) in vertices.iter().enumerate() {
            self.graph.vmap_set(&mut indices, v, Some(i));
        }
        let mut edges = vec![Vec::new(); vertices.len()];
        let mut reverse = vec![Vec::new(); vertices.len()];
        let mut seen = vec![None; vertices.len()];
        for (i, &v) in vertices.iter().enumerate() {
            for next in self.graph.neighbors(v) {
                let j = self
                    .graph
                    .vmap_get(&indices, next.to)
                    .expect("vertex is not enumerated by the graph");
                if seen[j] != Some(i) {
                    seen[j] = Some(i);
                    edges[i].push(j);
                    reverse[j].push(i);
                }
            }
        }
        self.index = Some(GraphIndex {
            indices,
            edges,
            reverse,
        });
    }

    fn index_of(&self, v: G::Vertex) -> usize {
        self.graph
            .vmap_get(&self.index.as_ref().unwrap().indices, v)
            .expect("vertex is not enumerated by the graph")
    }

    fn solve_generalized_grundy(&mut self) {
        let index = self.index.as_ref().unwrap();
        let n = index.edges.len();
        if self.generalized_grundy.len() == n {
            return;
        }
        let mut values: Vec<Option<(u64, usize)>> = vec![None; n];
        // Visit acyclic followers first, so DAGs need only one pass.
        let reverse = UsizeGraph::new(n, |v: usize| index.reverse[v].iter().map(|&u| (u, ())));
        let order = reverse.topological_sort();
        if order.len() == n {
            for (counter, v) in order.into_iter().enumerate() {
                let mex = self.grundy.get(v).copied().flatten().unwrap_or_else(|| {
                    let options = index.edges[v]
                        .iter()
                        .map(|&u| values[u].unwrap().0)
                        .collect();
                    game_mex(options)
                });
                values[v] = Some((mex, counter));
            }
            self.generalized_grundy = values;
            return;
        }
        let mut queue: VecDeque<_> = order.into();
        let mut queued = vec![false; n];
        for &v in &queue {
            queued[v] = true;
        }
        for (v, pending) in queued.iter_mut().enumerate() {
            if !*pending {
                *pending = true;
                queue.push_back(v);
            }
        }
        let mut counter = 0;
        let mut options = vec![HashSet::new(); n];
        let mut mexes = vec![0; n];
        while let Some(v) = queue.pop_front() {
            queued[v] = false;
            let mex = mexes[v];
            // Every unresolved option must be able to return to this mex.
            let ready = index.edges[v]
                .iter()
                .all(|&u| values[u].is_some() || options[u].contains(&mex));
            if !ready {
                continue;
            }
            values[v] = Some((mex, counter));
            counter += 1;
            for &u in &index.reverse[v] {
                if values[u].is_some() {
                    continue;
                }
                let inserted = options[u].insert(mex);
                if inserted {
                    while options[u].contains(&mexes[u]) {
                        mexes[u] += 1;
                    }
                }
                // A larger value already had a follower at this vertex's mex.
                if mex < mexes[u] && !queued[u] {
                    queued[u] = true;
                    queue.push_back(u);
                }
                if inserted {
                    for &w in &index.reverse[u] {
                        if queue.len() == n - counter {
                            break; // All unresolved vertices are already queued.
                        }
                        if values[w].is_none() && !queued[w] && mexes[w] == mex {
                            queued[w] = true;
                            queue.push_back(w);
                        }
                    }
                }
            }
        }
        self.generalized_grundy = values;
    }
}

impl GraphSums {
    fn prepare(&mut self, state: Vec<usize>, components: &[Vec<usize>]) -> usize {
        let state = Self::canonical(state, components);
        if let Some(&root) = self.indices.get(&state) {
            return root;
        }
        let root = self.outcomes.values.len();
        self.indices.insert(state.clone(), root);
        let mut edges = vec![Vec::new()];
        let mut reverse = vec![Vec::new()];
        let mut pending = VecDeque::from([(root, state)]);
        while let Some((v, state)) = pending.pop_front() {
            for (i, &component) in state.iter().enumerate() {
                if i != 0 && state[i - 1] == component {
                    continue;
                }
                for &next in &components[component] {
                    let mut to = state.clone();
                    to[i] = next;
                    let to = Self::canonical(to, components);
                    let u = *self.indices.entry(to).or_insert_with_key(|to| {
                        let u = root + edges.len();
                        pending.push_back((u, to.clone()));
                        edges.push(Vec::new());
                        reverse.push(Vec::new());
                        u
                    });
                    edges[v - root].push(u);
                }
            }
            edges[v - root].sort_unstable();
            edges[v - root].dedup();
            for &u in &edges[v - root] {
                if u >= root {
                    reverse[u - root].push(v);
                }
            }
        }
        self.outcomes.extend(&edges, &reverse, true);
        root
    }

    fn canonical(mut state: Vec<usize>, components: &[Vec<usize>]) -> Vec<usize> {
        state.retain(|&v| !components[v].is_empty());
        state.sort_unstable();
        state
    }
}

impl GraphOutcomes {
    fn extend(&mut self, edges: &[Vec<usize>], reverse: &[Vec<usize>], misere: bool) {
        let start = self.values.len();
        self.values.resize(start + edges.len(), GameOutcome::Draw);
        self.winning.resize(start + edges.len(), None);
        let mut remaining: Vec<_> = edges.iter().map(Vec::len).collect();
        let mut queue = VecDeque::new();
        for (i, next) in edges.iter().enumerate() {
            let v = start + i;
            if next.is_empty() {
                self.values[v] = GameOutcome::from_win(misere);
            } else if start != 0 {
                // The old prefix is closed under moves; only the appended states need solving.
                for &u in next {
                    if u < start {
                        match self.values[u] {
                            GameOutcome::Lose => {
                                self.values[v] = GameOutcome::Win;
                                self.winning[v] = Some(u);
                                break;
                            }
                            GameOutcome::Win => remaining[v - start] -= 1,
                            GameOutcome::Draw => {}
                        }
                    }
                }
                if self.values[v] == GameOutcome::Draw && remaining[v - start] == 0 {
                    self.values[v] = GameOutcome::Lose;
                }
            }
            if self.values[v] != GameOutcome::Draw {
                queue.push_back(v);
            }
        }
        while let Some(v) = queue.pop_front() {
            for &u in &reverse[v - start] {
                if self.values[u] != GameOutcome::Draw {
                    continue;
                }
                if self.values[v] == GameOutcome::Lose {
                    self.values[u] = GameOutcome::Win;
                    self.winning[u] = Some(v);
                    queue.push_back(u);
                } else {
                    remaining[u - start] -= 1;
                    if remaining[u - start] == 0 {
                        self.values[u] = GameOutcome::Lose;
                        queue.push_back(u);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::AdjacencyListGraph;
    use crate::tools::testutil::exhaustive_sequences;

    #[test]
    fn sums_against_play() {
        let mut graphs = Vec::new();
        // Every graph on up to three vertices, including duplicate labeled edges.
        for n in 0..=3 {
            for mask in 0..1usize << (n * n) {
                let mut graph = AdjacencyListGraph::new(n);
                for u in 0..n {
                    for v in 0..n {
                        if mask >> (u * n + v) & 1 != 0 {
                            graph.add_edge(u, v);
                            if (u + v) % 2 == 0 {
                                graph.add_edge(u, v);
                            }
                        }
                    }
                }
                graphs.push((graph, 3));
            }
        }
        // Resolving another option below the mex must wake its predecessor,
        // even when that option's value was already present.
        let mut graph = AdjacencyListGraph::new(4);
        for (u, v) in [(0, 3), (0, 1), (1, 2), (2, 1), (2, 3)] {
            graph.add_edge(u, v);
        }
        graphs.push((graph, 3));
        // In [3, 2], preserving a draw requires moving the finite component.
        let mut graph = AdjacencyListGraph::new(5);
        for (u, v) in [(1, 0), (2, 0), (2, 1), (3, 4), (4, 4), (4, 2)] {
            graph.add_edge(u, v);
        }
        graphs.push((graph, 3));
        // In [3, 0], skip repeated unsafe infinite successors before choosing 1.
        let mut graph = AdjacencyListGraph::new(4);
        for (u, v) in [(1, 1), (2, 2), (2, 0), (3, 2), (3, 2), (3, 1)] {
            graph.add_edge(u, v);
        }
        graphs.push((graph, 3));
        for copies in [1, 2, 31] {
            let mut graph = AdjacencyListGraph::new(6);
            for (u, v) in [(0, 0), (3, 4), (4, 3), (4, 5)] {
                graph.add_edge(u, v);
            }
            for _ in 0..copies {
                graph.add_edge(1, 2);
                graph.add_edge(2, 3);
            }
            graphs.push((graph, 3));
            let mut graph = AdjacencyListGraph::new(3);
            for _ in 0..copies {
                graph.add_edge(0, 2);
                graph.add_edge(1, 1);
            }
            graph.add_edge(1, 2);
            graphs.push((graph, 3));
        }
        // Normal-equivalent positions need not be interchangeable under misere.
        let mut graph = AdjacencyListGraph::new(4);
        for (u, v) in [(1, 0), (2, 0), (2, 1), (3, 2)] {
            graph.add_edge(u, v);
        }
        graphs.push((graph, 3));
        // Keep a terminal in larger samples so finite values also occur.
        let mut rng = crate::tools::Xorshift::default();
        for n in 4..=8 {
            for degree in 0..=n {
                for dag in [false, true] {
                    let mut graph = AdjacencyListGraph::new(n);
                    for v in 1..n {
                        for _ in 0..degree {
                            let bound = if dag { v } else { n };
                            graph.add_edge(v, rng.rand(bound as u64) as usize);
                        }
                    }
                    graphs.push((graph, 3));
                }
            }
        }
        let mut nim = AdjacencyListGraph::new(5);
        for s in 0..5 {
            for to in 0..s {
                nim.add_edge(s, to);
            }
        }
        graphs.push((nim, 4));
        for (graph, max_len) in &graphs {
            let n = graph.vsize;
            let mut game = GameAnalyzer::new().impartial_graph(graph);
            let mut acyclic = (0..n)
                .all(|s| graph.neighbors(s).all(|m| m.to < s))
                .then(|| GameAnalyzer::new().impartial_moves(|&s: &usize| graph.neighbors(s)));
            let mut grundy = vec![0u64; n];
            if acyclic.is_some() {
                for s in 0..n {
                    grundy[s] = (0..)
                        .find(|g| graph.neighbors(s).all(|m| grundy[m.to] != *g))
                        .unwrap();
                }
            }
            // A singleton sum also checks the single-position outcome and strategy.
            for len in 0..=*max_len {
                let states: Vec<_> = exhaustive_sequences(0..n, len..=len).collect();
                let encode = |s: &[usize]| s.iter().rev().fold(0, |id, v| id * n + v);
                let edges: Vec<Vec<_>> = states
                    .iter()
                    .map(|state| {
                        let mut edges = Vec::new();
                        for (i, &v) in state.iter().enumerate() {
                            for next in graph.neighbors(v) {
                                let mut to = state.clone();
                                to[i] = next.to;
                                edges.push(Neighbor::new(to, (i, next.label)));
                            }
                        }
                        edges
                    })
                    .collect();
                let indexed: Vec<Vec<_>> = edges
                    .iter()
                    .map(|next| next.iter().map(|e| encode(&e.to)).collect())
                    .collect();
                for misere in [false, true] {
                    let expected = outcomes(&indexed, misere);
                    let mut policy = vec![Vec::new(); 2 * states.len()];
                    // Descending queries append reachable states to an existing sum cache.
                    for (id, state) in states.iter().enumerate().rev() {
                        let (actual, moves, winning, best) = if misere {
                            let mut query = game.sum().misere().outcome();
                            (
                                query.eval(state),
                                query.moves(state),
                                query.winning_moves(state),
                                query.best_move(state),
                            )
                        } else {
                            let mut query = game.sum().outcome();
                            (
                                query.eval(state),
                                query.moves(state),
                                query.winning_moves(state),
                                query.best_move(state),
                            )
                        };
                        assert_eq!(
                            actual, expected[id],
                            "{:?}, {state:?}, {misere}",
                            graph.graph
                        );
                        let evaluated: Vec<_> = edges[id]
                            .iter()
                            .map(|next| EvaluatedMove {
                                next: next.clone(),
                                value: expected[encode(&next.to)],
                            })
                            .collect();
                        assert_eq!(moves, evaluated);
                        let expected_winning: Vec<_> = evaluated
                            .iter()
                            .filter(|m| m.value == GameOutcome::Lose)
                            .map(|m| m.next.clone())
                            .collect();
                        assert_eq!(winning, expected_winning);
                        if let Some(closure) = &mut acyclic {
                            let (value, moves, winning) = if misere {
                                let mut query = closure.misere().sum().outcome();
                                (
                                    query.eval(state),
                                    query.moves(state),
                                    query.winning_moves(state),
                                )
                            } else {
                                let mut query = closure.sum().outcome();
                                (
                                    query.eval(state),
                                    query.moves(state),
                                    query.winning_moves(state),
                                )
                            };
                            assert_eq!(value, expected[id]);
                            assert_eq!(moves, evaluated);
                            assert_eq!(winning, expected_winning);
                            if !misere {
                                let xor = state.iter().fold(0, |g, &s| g ^ grundy[s]);
                                assert_eq!(closure.sum().grundy().eval(state), xor);
                                assert_eq!(game.sum().grundy().eval(state), xor);
                                assert_eq!(
                                    closure.sum().grundy().winning_moves(state),
                                    expected_winning
                                );
                                assert_eq!(
                                    game.sum().grundy().winning_moves(state),
                                    expected_winning
                                );
                            }
                        }
                        if let Some(next) = &best {
                            assert!(edges[id].contains(next));
                        } else {
                            assert!(edges[id].is_empty() || actual == GameOutcome::Lose);
                        }
                        if let Some(next) = best.as_ref().or_else(|| edges[id].first()) {
                            policy[2 * id].push(2 * encode(&next.to) + 1);
                        }
                        policy[2 * id + 1] = indexed[id].iter().map(|u| 2 * u).collect();
                    }
                    // Fix our strategy to best_move, leaving every opponent reply available.
                    let actual = outcomes(&policy, misere);
                    for (id, &expected) in expected.iter().enumerate() {
                        if expected != GameOutcome::Lose {
                            assert_eq!(actual[2 * id], expected);
                        }
                    }
                }
            }
            if n != 0 {
                let before = game.sum().misere().outcome().eval(&[0, 0]);
                game.clear_cache();
                assert_eq!(game.misere().sum().outcome().eval(&[0, 0]), before);
            }
        }
    }

    // Independent fixed point on the full ordered product, without reverse edges or a queue.
    fn outcomes(edges: &[Vec<usize>], misere: bool) -> Vec<GameOutcome> {
        let mut values: Vec<_> = edges
            .iter()
            .map(|e| e.is_empty().then_some(misere))
            .collect();
        loop {
            let next: Vec<_> = edges
                .iter()
                .zip(&values)
                .map(|(edges, &known)| {
                    known.or_else(|| {
                        if edges.iter().any(|&v| values[v] == Some(false)) {
                            Some(true)
                        } else if edges.iter().all(|&v| values[v] == Some(true)) {
                            Some(false)
                        } else {
                            None
                        }
                    })
                })
                .collect();
            if next == values {
                break;
            }
            values = next;
        }
        values
            .into_iter()
            .map(|v| match v {
                Some(true) => GameOutcome::Win,
                Some(false) => GameOutcome::Lose,
                None => GameOutcome::Draw,
            })
            .collect()
    }

    #[test]
    fn reachable_dag_and_parallel_edges() {
        let mut graph = AdjacencyListGraph::new(6);
        for (u, v) in [
            (1, 0),
            (1, 0),
            (2, 0),
            (2, 1),
            (3, 4),
            (4, 3),
            (5, 3),
            (5, 2),
        ] {
            graph.add_edge(u, v);
        }
        let mut game = GameAnalyzer::new().impartial_graph(&graph);
        assert_eq!(game.grundy().table(0..3), vec![(0, 0), (1, 1), (2, 2)]);
        assert_eq!(game.value().eval(&2).as_nimber(), Some(2));
        assert_eq!(
            game.value()
                .winning_moves(&2)
                .iter()
                .map(|n| n.to)
                .collect::<Vec<_>>(),
            vec![0]
        );
        assert_eq!(game.grundy().eval(&2), 2);
        assert_eq!(game.outcome().eval(&5), GameOutcome::Draw);
        assert_eq!(game.outcome().winning_moves(&1).len(), 2);
        assert_eq!(
            game.grundy()
                .moves(&1)
                .iter()
                .map(|m| m.next.label)
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
        game.clear_cache();
        assert_eq!(game.misere().outcome().eval(&1), GameOutcome::Lose);
        assert_eq!(game.outcome().eval(&1), GameOutcome::Win);
    }

    #[test]
    fn sparse_vertices() {
        use crate::graph::ClosureGraph;
        let graph = ClosureGraph::new(
            || [10, 20].into_iter(),
            |v| (v == 20).then_some((10, ())).into_iter(),
        );
        let mut game = GameAnalyzer::new().impartial_graph(&graph);
        assert_eq!(game.grundy().eval(&20), 1);
        assert_eq!(game.outcome().eval(&20), GameOutcome::Win);
    }

    #[test]
    fn vertex_map_without_hash() {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        struct Vertex(usize);
        struct Label(usize);
        struct Path;
        impl Graph for Path {
            type Vertex = Vertex;
            type Label = Label;
            type Vertices<'g> = std::iter::Map<std::ops::Range<usize>, fn(usize) -> Vertex>;
            type Neighbors<'g> = std::option::IntoIter<Neighbor<Vertex, Label>>;
            fn vsize(&self) -> usize {
                3
            }
            fn vertices(&self) -> Self::Vertices<'_> {
                (0..3).map(Vertex)
            }
            fn neighbors(&self, v: Vertex) -> Self::Neighbors<'_> {
                v.0.checked_sub(1)
                    .map(|u| Neighbor::new(Vertex(u), Label(v.0)))
                    .into_iter()
            }
        }
        impl<T> VertexMap<T> for Path {
            type Vmap = Vec<T>;
            fn construct_vmap<F>(&self, mut f: F) -> Vec<T>
            where
                F: FnMut() -> T,
            {
                (0..3).map(|_| f()).collect()
            }
            fn vmap_get<'a>(&self, map: &'a Vec<T>, v: Vertex) -> &'a T {
                &map[v.0]
            }
            fn vmap_get_mut<'a>(&self, map: &'a mut Vec<T>, v: Vertex) -> &'a mut T {
                &mut map[v.0]
            }
        }
        let mut game = GameAnalyzer::new().impartial_graph(&Path);
        assert_eq!(game.grundy().eval(&Vertex(2)), 0);
        assert_eq!(game.outcome().eval(&Vertex(1)), GameOutcome::Win);
        assert_eq!(game.outcome().best_move(&Vertex(1)).unwrap().label.0, 1);
    }
}

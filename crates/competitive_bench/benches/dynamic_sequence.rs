use competitive::{
    algebra::RangeSumRangeLinear,
    data_structure::{ImplicitSplayTree, ImplicitTreap},
    num::mint_basic::MInt998244353,
    tools::Xorshift,
};
use criterion::{BatchSize, Criterion};
use std::hint::black_box;

type M = MInt998244353;

#[derive(Clone, Copy)]
enum Query {
    Insert { i: usize, x: M },
    Remove { i: usize },
    Reverse { l: usize, r: usize },
    Update { l: usize, r: usize, a: M, b: M },
    Fold { l: usize, r: usize },
}

fn gen_case(n: usize, q: usize) -> (Vec<M>, Vec<Query>) {
    let mut rng = Xorshift::default();
    let a = (0..n)
        .map(|_| M::new_unchecked(rng.random(0..998_244_353)))
        .collect::<Vec<_>>();
    let mut len = n;
    let mut queries = Vec::with_capacity(q);
    for _ in 0..q {
        match rng.random(0..5) {
            0 => {
                let i = rng.random(0..=len);
                let x = M::new_unchecked(rng.random(0..998_244_353));
                queries.push(Query::Insert { i, x });
                len += 1;
            }
            1 if len > 0 => {
                let i = rng.random(0..len);
                queries.push(Query::Remove { i });
                len -= 1;
            }
            ty => {
                if len == 0 {
                    continue;
                }
                let l = rng.random(0..len);
                let r = rng.random(l + 1..=len);
                match ty {
                    2 => queries.push(Query::Reverse { l, r }),
                    3 => {
                        let a = M::new_unchecked(rng.random(0..998_244_353));
                        let b = M::new_unchecked(rng.random(0..998_244_353));
                        queries.push(Query::Update { l, r, a, b });
                    }
                    _ => queries.push(Query::Fold { l, r }),
                }
            }
        }
    }
    (a, queries)
}

fn run_implicit_treap(a: &[M], queries: &[Query]) -> M {
    let mut seq = ImplicitTreap::<RangeSumRangeLinear<M>>::with_capacity(a.len() + queries.len());
    seq.extend(a.iter().copied());
    let mut acc = M::new_unchecked(0);
    for &query in queries {
        match query {
            Query::Insert { i, x } => seq.insert(i, x),
            Query::Remove { i } => {
                seq.remove(i);
            }
            Query::Reverse { l, r } => seq.reverse(l..r),
            Query::Update { l, r, a, b } => seq.update(l..r, (a, b)),
            Query::Fold { l, r } => acc += seq.fold(l..r).0,
        }
    }
    acc
}

fn run_implicit_splay_tree(a: &[M], queries: &[Query]) -> M {
    let mut seq =
        ImplicitSplayTree::<RangeSumRangeLinear<M>>::with_capacity(a.len() + queries.len());
    seq.extend(a.iter().copied());
    let mut acc = M::new_unchecked(0);
    for &query in queries {
        match query {
            Query::Insert { i, x } => seq.insert(i, x),
            Query::Remove { i } => {
                seq.remove(i);
            }
            Query::Reverse { l, r } => seq.reverse(l..r),
            Query::Update { l, r, a, b } => seq.update(l..r, (a, b)),
            Query::Fold { l, r } => acc += seq.fold(l..r).0,
        }
    }
    acc
}

pub fn bench_dynamic_sequence(c: &mut Criterion) {
    let (a, queries) = gen_case(5_000, 20_000);
    let mut group = c.benchmark_group("dynamic_sequence");
    group.bench_function("implicit_treap", |b| {
        b.iter_batched(
            || (a.clone(), queries.clone()),
            |(a, queries)| black_box(run_implicit_treap(&a, &queries)),
            BatchSize::LargeInput,
        )
    });
    group.bench_function("implicit_splay_tree", |b| {
        b.iter_batched(
            || (a.clone(), queries.clone()),
            |(a, queries)| black_box(run_implicit_splay_tree(&a, &queries)),
            BatchSize::LargeInput,
        )
    });
    group.finish();
}

#[cfg(verus_keep_ghost)]
use verification::*;
#[cfg(verus_keep_ghost)]
use vstd::{prelude::*, std_specs::cmp::PartialEqSpec};

#[cfg_attr(verus_keep_ghost, verus_verify)]
#[cfg_attr(verus_keep_ghost, verus_spec(result =>
    requires x.len() + y.len() < usize::MAX, T::obeys_eq_spec(),
    ensures
        result == distance(x@, y@, x.len() as nat, y.len() as nat),
        exists|script: Seq<Edit>| #[trigger] script_cost(x@, y@, x.len() as nat, y.len() as nat, script) == Some(result as nat),
        forall|script: Seq<Edit>| #[trigger] script_cost(x@, y@, x.len() as nat, y.len() as nat, script) is Some
            ==> result <= script_cost(x@, y@, x.len() as nat, y.len() as nat, script).unwrap(),
))]
pub fn levenshtein_distance<T: PartialEq>(x: &[T], y: &[T]) -> usize {
    let n = x.len();
    let m = y.len();
    let mut dp = vec![vec![0; m + 1]; n + 1];
    #[cfg_attr(verus_keep_ghost, verus_spec(iter =>
        invariant
            n == x.len(), m == y.len(), table_shape(dp@, n as nat, m as nat),
            forall|p: int| 0 <= p < iter.index() ==> #[trigger] dp@[p]@[0] == p,
    ))]
    // Verus does not model Enumerate; explicit indices keep the table invariant available.
    #[allow(clippy::needless_range_loop)]
    for i in 0..=n {
        dp[i][0] = i;
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(iter =>
        invariant
            n == x.len(), m == y.len(), table_shape(dp@, n as nat, m as nat),
            forall|p: int| 0 <= p <= n ==> #[trigger] dp@[p]@[0] == p,
            forall|q: int| 0 <= q < iter.index() ==> #[trigger] dp@[0]@[q] == q,
    ))]
    #[allow(clippy::needless_range_loop)] // Same table invariant as the row initialization.
    for j in 0..=m {
        dp[0][j] = j;
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(iter =>
        invariant
            n == x.len(), m == y.len(), n + m < usize::MAX, T::obeys_eq_spec(),
            table_shape(dp@, n as nat, m as nat),
            forall|p: int| 0 <= p <= n ==> #[trigger] dp@[p]@[0] == p,
            forall|p: int, q: int| 0 <= p <= iter.index() && 0 <= q <= m
                ==> #[trigger] dp@[p]@[q] == distance(x@, y@, p as nat, q as nat),
    ))]
    for (i, x_value) in (0..n).zip(x) {
        #[cfg_attr(verus_keep_ghost, verus_spec(iter =>
            invariant
                n == x.len(), m == y.len(), n + m < usize::MAX, T::obeys_eq_spec(),
                i < n, *x_value == x@[i as int], table_shape(dp@, n as nat, m as nat),
                forall|p: int| 0 <= p <= n ==> #[trigger] dp@[p]@[0] == p,
                forall|p: int, q: int| 0 <= p <= i && 0 <= q <= m
                    ==> #[trigger] dp@[p]@[q] == distance(x@, y@, p as nat, q as nat),
                forall|q: int| 0 <= q <= iter.index()
                    ==> #[trigger] dp@[i + 1]@[q] == distance(x@, y@, (i + 1) as nat, q as nat),
        ))]
        for (j, y_value) in (0..m).zip(y) {
            #[cfg(verus_keep_ghost)]
            proof! {
                distance_bound(x@, y@, i as nat, (j + 1) as nat);
                distance_bound(x@, y@, (i + 1) as nat, j as nat);
                distance_bound(x@, y@, i as nat, j as nat);
            }
            dp[i + 1][j + 1] =
                (dp[i][j + 1].min(dp[i + 1][j]) + 1).min(dp[i][j] + (x_value != y_value) as usize);
        }
    }
    #[cfg(verus_keep_ghost)]
    proof! {
        optimal_alignment(x@, y@, n as nat, m as nat);
        #[verifier::proof_note("For arbitrary input lengths whose sum is below usize::MAX and stable PartialEq, the result is attainable by insertions, deletions and substitutions of unit cost, with matching pairs free")]
        assert(exists|script: Seq<Edit>| #[trigger] script_cost(x@, y@, n as nat, m as nat, script) == Some(dp@[n as int]@[m as int] as nat));
        #[verifier::proof_note("Every alignment consuming both input sequences has cost at least the returned distance, so the returned edit cost is minimal, including empty inputs")]
        assert(forall|script: Seq<Edit>| #[trigger] script_cost(x@, y@, n as nat, m as nat, script) is Some
            ==> dp@[n as int]@[m as int] <= script_cost(x@, y@, n as nat, m as nat, script).unwrap());
    }
    dp[n][m]
}

#[cfg(verus_keep_ghost)]
mod verification {
    use super::*;
    use vstd::math::{max, min};

    verus! {
        pub enum Edit { Delete, Insert, Align }

        // Recursive specifications have termination checks; the lemmas below connect
        // the recurrence to the minimum cost of an edit script.
        pub open spec fn script_cost<T: PartialEq>(
            x: Seq<T>, y: Seq<T>, i: nat, j: nat, script: Seq<Edit>,
        ) -> Option<nat>
            recommends i <= x.len(), j <= y.len(),
            decreases script.len(),
        {
            if script.len() == 0 {
                if i == 0 && j == 0 { Some(0nat) } else { None }
            } else {
                let previous = match script.last() {
                    Edit::Delete => if i > 0 {
                        script_cost(x, y, (i - 1) as nat, j, script.drop_last())
                    } else { None },
                    Edit::Insert => if j > 0 {
                        script_cost(x, y, i, (j - 1) as nat, script.drop_last())
                    } else { None },
                    Edit::Align => if i > 0 && j > 0 {
                        script_cost(x, y, (i - 1) as nat, (j - 1) as nat, script.drop_last())
                    } else { None },
                };
                match previous {
                    Some(cost) => Some(cost + match script.last() {
                        Edit::Align => if x[i - 1].eq_spec(&y[j - 1]) { 0nat } else { 1nat },
                        _ => 1nat,
                    }),
                    None => None,
                }
            }
        }

        pub open spec fn distance<T: PartialEq>(x: Seq<T>, y: Seq<T>, i: nat, j: nat) -> nat
            recommends i <= x.len(), j <= y.len(),
            decreases i + j,
        {
            if i == 0 {
                j
            } else if j == 0 {
                i
            } else {
                let deletion = distance(x, y, (i - 1) as nat, j) + 1;
                let insertion = distance(x, y, i, (j - 1) as nat) + 1;
                let alignment = distance(x, y, (i - 1) as nat, (j - 1) as nat)
                    + if x[i - 1].eq_spec(&y[j - 1]) { 0nat } else { 1nat };
                min(min(deletion as int, insertion as int), alignment as int) as nat
            }
        }

        pub open spec fn table_shape(dp: Seq<Vec<usize>>, n: nat, m: nat) -> bool {
            dp.len() == n + 1
                && forall|i: int| 0 <= i <= n ==> #[trigger] dp[i]@.len() == m + 1
        }

        pub proof fn distance_bound<T: PartialEq>(x: Seq<T>, y: Seq<T>, i: nat, j: nat)
            requires i <= x.len(), j <= y.len(),
            ensures distance(x, y, i, j) <= max(i as int, j as int),
            decreases i + j,
        {
            if i > 0 && j > 0 {
                distance_bound(x, y, (i - 1) as nat, (j - 1) as nat);
            }
            #[verifier::proof_note("The unit-cost edit distance of any two prefixes is at most the larger prefix length")]
            assert(distance(x, y, i, j) <= max(i as int, j as int));
        }

        pub proof fn optimal_alignment<T: PartialEq>(x: Seq<T>, y: Seq<T>, i: nat, j: nat)
            requires i <= x.len(), j <= y.len(),
            ensures
                exists|script: Seq<Edit>| #[trigger] script_cost(x, y, i, j, script) == Some(distance(x, y, i, j)),
                forall|script: Seq<Edit>| #[trigger] script_cost(x, y, i, j, script) is Some
                    ==> script_cost(x, y, i, j, script).unwrap() >= distance(x, y, i, j),
            decreases i + j,
        {
            if i == 0 && j == 0 {
                assert(script_cost(x, y, i, j, Seq::empty()) == Some(0nat));
            } else {
                let step = if i == 0 {
                    Edit::Insert
                } else if j == 0 {
                    Edit::Delete
                } else {
                    optimal_alignment(x, y, (i - 1) as nat, j);
                    optimal_alignment(x, y, i, (j - 1) as nat);
                    optimal_alignment(x, y, (i - 1) as nat, (j - 1) as nat);
                    let deletion = distance(x, y, (i - 1) as nat, j) + 1;
                    let insertion = distance(x, y, i, (j - 1) as nat) + 1;
                    let alignment = distance(x, y, (i - 1) as nat, (j - 1) as nat)
                        + if x[i - 1].eq_spec(&y[j - 1]) { 0nat } else { 1nat };
                    if deletion <= insertion && deletion <= alignment {
                        Edit::Delete
                    } else if insertion <= alignment {
                        Edit::Insert
                    } else {
                        Edit::Align
                    }
                };
                if i == 0 { optimal_alignment(x, y, i, (j - 1) as nat); }
                if j == 0 { optimal_alignment(x, y, (i - 1) as nat, j); }
                let (previous_i, previous_j) = match step {
                    Edit::Delete => ((i - 1) as nat, j),
                    Edit::Insert => (i, (j - 1) as nat),
                    Edit::Align => ((i - 1) as nat, (j - 1) as nat),
                };
                let previous = choose|script: Seq<Edit>|
                    script_cost(x, y, previous_i, previous_j, script)
                        == Some(distance(x, y, previous_i, previous_j));
                let result = previous.push(step);
                assert(result.drop_last() == previous);
                assert(script_cost(x, y, i, j, result) == Some(distance(x, y, i, j)));
            }
            assert forall|script: Seq<Edit>| #[trigger] script_cost(x, y, i, j, script) is Some
                implies script_cost(x, y, i, j, script).unwrap() >= distance(x, y, i, j) by {
                if script.len() > 0 {
                    match script.last() {
                        Edit::Delete => {
                            assert(script_cost(x, y, (i - 1) as nat, j, script.drop_last()) is Some);
                        },
                        Edit::Insert => {
                            assert(script_cost(x, y, i, (j - 1) as nat, script.drop_last()) is Some);
                        },
                        Edit::Align => {
                            assert(script_cost(x, y, (i - 1) as nat, (j - 1) as nat, script.drop_last()) is Some);
                        },
                    }
                }
            }
            #[verifier::proof_note("The edit-distance recurrence is attainable by an alignment and is a lower bound for every valid alignment")]
            assert((exists|script: Seq<Edit>| #[trigger] script_cost(x, y, i, j, script) == Some(distance(x, y, i, j)))
                && (forall|script: Seq<Edit>| #[trigger] script_cost(x, y, i, j, script) is Some
                    ==> script_cost(x, y, i, j, script).unwrap() >= distance(x, y, i, j)));
        }
    }
}

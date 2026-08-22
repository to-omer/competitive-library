/// Returns the minimum cost and a column assigned to each row of a square cost matrix.
pub fn minimum_assignment<T>(cost: &[Vec<T>]) -> (i64, Vec<usize>)
where
    T: Copy + Into<i64>,
{
    let n = cost.len();
    assert!(cost.iter().all(|row| row.len() == n));
    if n == 0 {
        return (0, Vec::new());
    }
    let value = |row: usize, column: usize| cost[row][column].into();

    let mut monge = true;
    let mut anti_monge = true;
    'outer: for row in 0..n - 1 {
        for column in 0..n - 1 {
            let straight = i128::from(value(row, column)) + i128::from(value(row + 1, column + 1));
            let crossed = i128::from(value(row, column + 1)) + i128::from(value(row + 1, column));
            monge &= straight <= crossed;
            anti_monge &= straight >= crossed;
            if !monge && !anti_monge {
                break 'outer;
            }
        }
    }
    if monge || anti_monge {
        let assignment: Vec<_> = if monge {
            (0..n).collect()
        } else {
            (0..n).rev().collect()
        };
        let total = (0..n).map(|row| value(row, assignment[row])).sum();
        return (total, assignment);
    }

    let none = !0;
    let mut row_mate = vec![none; n];
    let mut column_mate = vec![none; n];
    let mut potential = vec![0; n];
    let mut transferable = vec![false; n];
    for column in 0..n {
        let mut row = 0;
        for next in 1..n {
            if value(next, column) < value(row, column) {
                row = next;
            }
        }
        potential[column] = value(row, column);
        if row_mate[row] == none {
            row_mate[row] = column;
            column_mate[column] = row;
            transferable[row] = true;
        } else {
            transferable[row] = false;
        }
    }
    for row in 0..n {
        if transferable[row] {
            let column = row_mate[row];
            let mut best = i64::MAX;
            for (next, &next_potential) in potential.iter().enumerate() {
                if next != column {
                    best = best.min(value(row, next) - next_potential);
                }
            }
            if best != i64::MAX {
                potential[column] -= best;
            }
        }
    }
    for _ in 0..2 {
        for row in 0..n {
            if row_mate[row] != none {
                continue;
            }
            let mut best = value(row, 0) - potential[0];
            let mut second = i64::MAX;
            let mut column = 0;
            for (next, &next_potential) in potential.iter().enumerate().skip(1) {
                let reduced = value(row, next) - next_potential;
                if reduced < best || reduced == best && column_mate[column] != none {
                    second = best;
                    best = reduced;
                    column = next;
                } else {
                    second = second.min(reduced);
                }
            }
            if best < second {
                potential[column] -= second - best;
            }
            let replaced = column_mate[column];
            if replaced != none {
                row_mate[replaced] = none;
            }
            row_mate[row] = column;
            column_mate[column] = row;
        }
    }

    let mut columns: Vec<_> = (0..n).collect();
    let mut distance = vec![0; n];
    let mut predecessor = vec![0; n];
    for start in 0..n {
        if row_mate[start] != none {
            continue;
        }
        for column in 0..n {
            distance[column] = value(start, column) - potential[column];
            predecessor[column] = start;
        }
        let mut scanned = 0;
        let mut labeled = 0;
        let mut last = 0;
        let free_column = loop {
            if scanned == labeled {
                last = scanned;
                let mut best = distance[columns[scanned]];
                for next in scanned..n {
                    let column = columns[next];
                    if distance[column] <= best {
                        if distance[column] < best {
                            best = distance[column];
                            labeled = scanned;
                        }
                        columns.swap(next, labeled);
                        labeled += 1;
                    }
                }
                if let Some(column) = columns[scanned..labeled]
                    .iter()
                    .copied()
                    .find(|&column| column_mate[column] == none)
                {
                    break column;
                }
            }
            let column = columns[scanned];
            scanned += 1;
            let row = column_mate[column];
            let base = value(row, column) - potential[column];
            let mut next = labeled;
            let mut free_column = none;
            while next < n {
                let other = columns[next];
                let edge = value(row, other) - potential[other] - base;
                let candidate = distance[column] + edge;
                if candidate < distance[other] {
                    distance[other] = candidate;
                    predecessor[other] = row;
                    if edge == 0 {
                        if column_mate[other] == none {
                            free_column = other;
                            break;
                        }
                        columns.swap(next, labeled);
                        labeled += 1;
                    }
                }
                next += 1;
            }
            if free_column != none {
                break free_column;
            }
        };
        for &column in &columns[..last] {
            potential[column] += distance[column] - distance[free_column];
        }
        let mut column = free_column;
        loop {
            let row = predecessor[column];
            column_mate[column] = row;
            let next = row_mate[row];
            row_mate[row] = column;
            if next == none {
                break;
            }
            column = next;
        }
    }

    let total = (0..n).map(|row| value(row, row_mate[row])).sum();
    (total, row_mate)
}

#[cfg(test)]
mod tests {
    use crate::{algorithm::SliceCombinationsExt, graph::minimum_assignment, tools::Xorshift};

    #[test]
    fn test_minimum_assignment() {
        let mut rng = Xorshift::default();
        for case in 0..100 {
            let n = rng.random(0..=8);
            let mut cost: Vec<Vec<i64>> = (0..n)
                .map(|_| rng.random_iter(-100..=100).take(n).collect())
                .collect();
            if case % 3 != 0 {
                let sign = if case % 3 == 1 { 1 } else { -1 };
                let row_bias: Vec<i64> = rng.random_iter(-100..=100).take(n).collect();
                let column_bias: Vec<i64> = rng.random_iter(-100..=100).take(n).collect();
                for row in 0..n {
                    for column in 0..n {
                        cost[row][column] = row_bias[row]
                            + column_bias[column]
                            + sign * (row as i64 - column as i64).pow(2);
                    }
                }
            }
            let mut permutation: Vec<_> = (0..n).collect();
            let mut expected = i64::MAX;
            loop {
                expected = expected.min(
                    cost.iter()
                        .zip(&permutation)
                        .map(|(row, &column)| row[column])
                        .sum(),
                );
                if !permutation.next_permutation() {
                    break;
                }
            }
            let (actual, assignment) = minimum_assignment(&cost);
            assert_eq!(expected, actual);
            assert_eq!(
                actual,
                cost.iter()
                    .zip(&assignment)
                    .map(|(row, &column)| row[column])
                    .sum()
            );
            let mut sorted = assignment;
            sorted.sort_unstable();
            assert!(sorted.into_iter().eq(0..n));
        }
    }
}

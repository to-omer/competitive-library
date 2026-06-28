/// Manages only prev/next links of indices.
///
/// `usize::MAX` means no previous or next index.
#[derive(Debug, Clone)]
pub struct DoublyLinkedList {
    prev: Vec<usize>,
    next: Vec<usize>,
}

impl DoublyLinkedList {
    pub fn new(n: usize) -> Self {
        Self {
            prev: vec![usize::MAX; n],
            next: vec![usize::MAX; n],
        }
    }

    pub fn len(&self) -> usize {
        self.prev.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn prev(&self, index: usize) -> usize {
        self.prev[index]
    }

    pub fn next(&self, index: usize) -> usize {
        self.next[index]
    }

    /// Links `front` immediately before `back`.
    ///
    /// Panics if `front == back`, `front` already has a next index, or `back` already has a
    /// previous index.
    pub fn link(&mut self, front: usize, back: usize) {
        assert_ne!(front, back);
        assert_eq!(self.next[front], usize::MAX);
        assert_eq!(self.prev[back], usize::MAX);
        self.next[front] = back;
        self.prev[back] = front;
    }

    pub fn cut_before(&mut self, index: usize) -> usize {
        let prev = self.prev[index];
        if prev != usize::MAX {
            self.next[prev] = usize::MAX;
            self.prev[index] = usize::MAX;
        }
        prev
    }

    pub fn cut_after(&mut self, index: usize) -> usize {
        let next = self.next[index];
        if next != usize::MAX {
            self.prev[next] = usize::MAX;
            self.next[index] = usize::MAX;
        }
        next
    }

    pub fn detach(&mut self, index: usize) -> (usize, usize) {
        let prev = self.cut_before(index);
        let next = self.cut_after(index);
        if prev != usize::MAX && next != usize::MAX {
            self.link(prev, next);
        }
        (prev, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn test_doubly_linked_list_random() {
        const CASES: usize = 200;
        const Q: usize = 200;

        let mut rng = Xorshift::default();
        for _ in 0..CASES {
            let n = rng.random(0..=30);
            let mut list = DoublyLinkedList::new(n);
            let mut lists: Vec<Vec<_>> = (0..n).map(|i| vec![i]).collect();
            let position = |lists: &[Vec<usize>], v| {
                lists
                    .iter()
                    .enumerate()
                    .find_map(|(i, list)| list.iter().position(|&u| u == v).map(|j| (i, j)))
                    .unwrap()
            };
            assert_eq!(list.is_empty(), n == 0);

            for _ in 0..Q {
                if n == 0 {
                    continue;
                }

                match rng.random(0..4) {
                    0 if lists.len() >= 2 => {
                        let mut a = rng.random(0..lists.len());
                        let mut b = rng.random(0..lists.len() - 1);
                        if a <= b {
                            b += 1;
                        }
                        let front = *lists[a].last().unwrap();
                        let back = lists[b][0];
                        list.link(front, back);

                        let other = lists.remove(b);
                        if b < a {
                            a -= 1;
                        }
                        lists[a].extend(other);
                    }
                    1 => {
                        let v = rng.random(0..n);
                        let (i, j) = position(&lists, v);
                        let prev = if j == 0 { usize::MAX } else { lists[i][j - 1] };
                        assert_eq!(list.cut_before(v), prev);
                        if j != 0 {
                            let right = lists[i].split_off(j);
                            lists.push(right);
                        }
                    }
                    2 => {
                        let v = rng.random(0..n);
                        let (i, j) = position(&lists, v);
                        let next = if j + 1 == lists[i].len() {
                            usize::MAX
                        } else {
                            lists[i][j + 1]
                        };
                        assert_eq!(list.cut_after(v), next);
                        if j + 1 != lists[i].len() {
                            let right = lists[i].split_off(j + 1);
                            lists.push(right);
                        }
                    }
                    _ => {
                        let v = rng.random(0..n);
                        let (i, j) = position(&lists, v);
                        let prev = if j == 0 { usize::MAX } else { lists[i][j - 1] };
                        let next = if j + 1 == lists[i].len() {
                            usize::MAX
                        } else {
                            lists[i][j + 1]
                        };
                        assert_eq!(list.detach(v), (prev, next));
                        lists[i].remove(j);
                        if lists[i].is_empty() {
                            lists.swap_remove(i);
                        }
                        lists.push(vec![v]);
                    }
                }

                let mut prev = vec![usize::MAX; n];
                let mut next = vec![usize::MAX; n];
                for row in &lists {
                    for (i, &v) in row.iter().enumerate() {
                        if i != 0 {
                            prev[v] = row[i - 1];
                        }
                        if i + 1 != row.len() {
                            next[v] = row[i + 1];
                        }
                    }
                }
                for i in 0..n {
                    assert_eq!(list.prev(i), prev[i]);
                    assert_eq!(list.next(i), next[i]);
                }
            }
        }
    }
}

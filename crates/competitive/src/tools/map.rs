#[cfg_attr(nightly, codesnip::entry("BTreeMapExt"))]
pub trait BTreeMapExt<K, V>
where
    K: Ord,
{
    fn first(&self) -> Option<(&K, &V)>;
    fn last(&self) -> Option<(&K, &V)>;
    fn get_next(&self, key: &K) -> Option<(&K, &V)>;
    fn get_next_excluded(&self, key: &K) -> Option<(&K, &V)>;
    fn get_next_back(&self, key: &K) -> Option<(&K, &V)>;
    fn get_next_back_excluded(&self, key: &K) -> Option<(&K, &V)>;

    fn first_mut(&mut self) -> Option<(&K, &mut V)>;
    fn last_mut(&mut self) -> Option<(&K, &mut V)>;
    fn get_next_mut(&mut self, key: &K) -> Option<(&K, &mut V)>;
    fn get_next_excluded_mut(&mut self, key: &K) -> Option<(&K, &mut V)>;
    fn get_next_back_mut(&mut self, key: &K) -> Option<(&K, &mut V)>;
    fn get_next_back_excluded_mut(&mut self, key: &K) -> Option<(&K, &mut V)>;

    fn pop_first(&mut self) -> Option<(K, V)>
    where
        K: Clone;
    fn pop_last(&mut self) -> Option<(K, V)>
    where
        K: Clone;
    fn pop_next(&mut self, key: &K) -> Option<(K, V)>
    where
        K: Clone;
    fn pop_next_excluded(&mut self, key: &K) -> Option<(K, V)>
    where
        K: Clone;
    fn pop_next_back(&mut self, key: &K) -> Option<(K, V)>
    where
        K: Clone;
    fn pop_next_back_excluded(&mut self, key: &K) -> Option<(K, V)>
    where
        K: Clone;

    fn pop_first_if<P>(&mut self, pred: P) -> Option<(K, V)>
    where
        K: Clone,
        P: FnMut(&K, &V) -> bool;
    fn pop_last_if<P>(&mut self, pred: P) -> Option<(K, V)>
    where
        K: Clone,
        P: FnMut(&K, &V) -> bool;
    fn pop_next_if<P>(&mut self, key: &K, pred: P) -> Option<(K, V)>
    where
        K: Clone,
        P: FnMut(&K, &V) -> bool;
    fn pop_next_excluded_if<P>(&mut self, key: &K, pred: P) -> Option<(K, V)>
    where
        K: Clone,
        P: FnMut(&K, &V) -> bool;
    fn pop_next_back_if<P>(&mut self, key: &K, pred: P) -> Option<(K, V)>
    where
        K: Clone,
        P: FnMut(&K, &V) -> bool;
    fn pop_next_back_excluded_if<P>(&mut self, key: &K, pred: P) -> Option<(K, V)>
    where
        K: Clone,
        P: FnMut(&K, &V) -> bool;
}
#[cfg_attr(nightly, codesnip::entry("BTreeMapExt"))]
impl<K, V> BTreeMapExt<K, V> for std::collections::BTreeMap<K, V>
where
    K: Ord,
{
    fn first(&self) -> Option<(&K, &V)> {
        self.range(..).next()
    }
    fn last(&self) -> Option<(&K, &V)> {
        self.range(..).next_back()
    }
    fn get_next(&self, key: &K) -> Option<(&K, &V)> {
        self.range(key..).next()
    }
    fn get_next_excluded(&self, key: &K) -> Option<(&K, &V)> {
        self.range((std::ops::Bound::Excluded(key), std::ops::Bound::Unbounded))
            .next()
    }
    fn get_next_back(&self, key: &K) -> Option<(&K, &V)> {
        self.range(..=key).next_back()
    }
    fn get_next_back_excluded(&self, key: &K) -> Option<(&K, &V)> {
        self.range(..key).next_back()
    }

    fn first_mut(&mut self) -> Option<(&K, &mut V)> {
        self.range_mut(..).next()
    }
    fn last_mut(&mut self) -> Option<(&K, &mut V)> {
        self.range_mut(..).next_back()
    }
    fn get_next_mut(&mut self, key: &K) -> Option<(&K, &mut V)> {
        self.range_mut(key..).next()
    }
    fn get_next_excluded_mut(&mut self, key: &K) -> Option<(&K, &mut V)> {
        self.range_mut((std::ops::Bound::Excluded(key), std::ops::Bound::Unbounded))
            .next()
    }
    fn get_next_back_mut(&mut self, key: &K) -> Option<(&K, &mut V)> {
        self.range_mut(..=key).next_back()
    }
    fn get_next_back_excluded_mut(&mut self, key: &K) -> Option<(&K, &mut V)> {
        self.range_mut(..key).next_back()
    }

    fn pop_first(&mut self) -> Option<(K, V)>
    where
        K: Clone,
    {
        self.pop_first_if(|_, _| true)
    }
    fn pop_last(&mut self) -> Option<(K, V)>
    where
        K: Clone,
    {
        self.pop_last_if(|_, _| true)
    }
    fn pop_next(&mut self, key: &K) -> Option<(K, V)>
    where
        K: Clone,
    {
        self.pop_next_if(key, |_, _| true)
    }
    fn pop_next_excluded(&mut self, key: &K) -> Option<(K, V)>
    where
        K: Clone,
    {
        self.pop_next_excluded_if(key, |_, _| true)
    }
    fn pop_next_back(&mut self, key: &K) -> Option<(K, V)>
    where
        K: Clone,
    {
        self.pop_next_back_if(key, |_, _| true)
    }
    fn pop_next_back_excluded(&mut self, key: &K) -> Option<(K, V)>
    where
        K: Clone,
    {
        self.pop_next_back_excluded_if(key, |_, _| true)
    }

    fn pop_first_if<P>(&mut self, mut pred: P) -> Option<(K, V)>
    where
        K: Clone,
        P: FnMut(&K, &V) -> bool,
    {
        match self.first().filter(|(k, v)| pred(k, v)) {
            Some((k, _)) => {
                let k = k.clone();
                let v = self.remove(&k).expect("This key must be exists.");
                Some((k, v))
            }
            None => None,
        }
    }
    fn pop_last_if<P>(&mut self, mut pred: P) -> Option<(K, V)>
    where
        K: Clone,
        P: FnMut(&K, &V) -> bool,
    {
        match self.last().filter(|(k, v)| pred(k, v)) {
            Some((k, _)) => {
                let k = k.clone();
                let v = self.remove(&k).expect("This key must be exists.");
                Some((k, v))
            }
            None => None,
        }
    }
    fn pop_next_if<P>(&mut self, key: &K, mut pred: P) -> Option<(K, V)>
    where
        K: Clone,
        P: FnMut(&K, &V) -> bool,
    {
        match self.get_next(key).filter(|(k, v)| pred(k, v)) {
            Some((k, _)) => {
                let k = k.clone();
                let v = self.remove(&k).expect("This key must be exists.");
                Some((k, v))
            }
            None => None,
        }
    }
    fn pop_next_excluded_if<P>(&mut self, key: &K, mut pred: P) -> Option<(K, V)>
    where
        K: Clone,
        P: FnMut(&K, &V) -> bool,
    {
        match self.get_next_excluded(key).filter(|(k, v)| pred(k, v)) {
            Some((k, _)) => {
                let k = k.clone();
                let v = self.remove(&k).expect("This key must be exists.");
                Some((k, v))
            }
            None => None,
        }
    }
    fn pop_next_back_if<P>(&mut self, key: &K, mut pred: P) -> Option<(K, V)>
    where
        K: Clone,
        P: FnMut(&K, &V) -> bool,
    {
        match self.get_next_back(key).filter(|(k, v)| pred(k, v)) {
            Some((k, _)) => {
                let k = k.clone();
                let v = self.remove(&k).expect("This key must be exists.");
                Some((k, v))
            }
            None => None,
        }
    }
    fn pop_next_back_excluded_if<P>(&mut self, key: &K, mut pred: P) -> Option<(K, V)>
    where
        K: Clone,
        P: FnMut(&K, &V) -> bool,
    {
        match self.get_next_back_excluded(key).filter(|(k, v)| pred(k, v)) {
            Some((k, _)) => {
                let k = k.clone();
                let v = self.remove(&k).expect("This key must be exists.");
                Some((k, v))
            }
            None => None,
        }
    }
}

#[cfg_attr(nightly, codesnip::entry("BTreeSetExt"))]
pub trait BTreeSetExt<T>
where
    T: Ord,
{
    fn first(&self) -> Option<&T>;
    fn last(&self) -> Option<&T>;
    fn get_next(&self, key: &T) -> Option<&T>;
    fn get_next_excluded(&self, key: &T) -> Option<&T>;
    fn get_next_back(&self, key: &T) -> Option<&T>;
    fn get_next_back_excluded(&self, key: &T) -> Option<&T>;

    fn pop_first(&mut self) -> Option<T>
    where
        T: Clone;
    fn pop_last(&mut self) -> Option<T>
    where
        T: Clone;
    fn pop_next(&mut self, key: &T) -> Option<T>
    where
        T: Clone;
    fn pop_next_excluded(&mut self, key: &T) -> Option<T>
    where
        T: Clone;
    fn pop_next_back(&mut self, key: &T) -> Option<T>
    where
        T: Clone;
    fn pop_next_back_excluded(&mut self, key: &T) -> Option<T>
    where
        T: Clone;

    fn pop_first_if<P>(&mut self, pred: P) -> Option<T>
    where
        T: Clone,
        P: FnMut(&T) -> bool;
    fn pop_last_if<P>(&mut self, pred: P) -> Option<T>
    where
        T: Clone,
        P: FnMut(&T) -> bool;
    fn pop_next_if<P>(&mut self, key: &T, pred: P) -> Option<T>
    where
        T: Clone,
        P: FnMut(&T) -> bool;
    fn pop_next_excluded_if<P>(&mut self, key: &T, pred: P) -> Option<T>
    where
        T: Clone,
        P: FnMut(&T) -> bool;
    fn pop_next_back_if<P>(&mut self, key: &T, pred: P) -> Option<T>
    where
        T: Clone,
        P: FnMut(&T) -> bool;
    fn pop_next_back_excluded_if<P>(&mut self, key: &T, pred: P) -> Option<T>
    where
        T: Clone,
        P: FnMut(&T) -> bool;
}
#[cfg_attr(nightly, codesnip::entry("BTreeSetExt"))]
impl<T> BTreeSetExt<T> for std::collections::BTreeSet<T>
where
    T: Ord,
{
    fn first(&self) -> Option<&T> {
        self.range(..).next()
    }
    fn last(&self) -> Option<&T> {
        self.range(..).next_back()
    }
    fn get_next(&self, key: &T) -> Option<&T> {
        self.range(key..).next()
    }
    fn get_next_excluded(&self, key: &T) -> Option<&T> {
        self.range((std::ops::Bound::Excluded(key), std::ops::Bound::Unbounded))
            .next()
    }
    fn get_next_back(&self, key: &T) -> Option<&T> {
        self.range(..=key).next_back()
    }
    fn get_next_back_excluded(&self, key: &T) -> Option<&T> {
        self.range(..key).next_back()
    }

    fn pop_first(&mut self) -> Option<T>
    where
        T: Clone,
    {
        self.pop_first_if(|_| true)
    }
    fn pop_last(&mut self) -> Option<T>
    where
        T: Clone,
    {
        self.pop_last_if(|_| true)
    }
    fn pop_next(&mut self, key: &T) -> Option<T>
    where
        T: Clone,
    {
        self.pop_next_if(key, |_| true)
    }
    fn pop_next_excluded(&mut self, key: &T) -> Option<T>
    where
        T: Clone,
    {
        self.pop_next_excluded_if(key, |_| true)
    }
    fn pop_next_back(&mut self, key: &T) -> Option<T>
    where
        T: Clone,
    {
        self.pop_next_back_if(key, |_| true)
    }
    fn pop_next_back_excluded(&mut self, key: &T) -> Option<T>
    where
        T: Clone,
    {
        self.pop_next_back_excluded_if(key, |_| true)
    }

    fn pop_first_if<P>(&mut self, mut pred: P) -> Option<T>
    where
        T: Clone,
        P: FnMut(&T) -> bool,
    {
        match <Self as BTreeSetExt<T>>::first(self).filter(|k| pred(k)) {
            Some(k) => {
                let k = k.clone();
                self.remove(&k);
                Some(k)
            }
            None => None,
        }
    }
    fn pop_last_if<P>(&mut self, mut pred: P) -> Option<T>
    where
        T: Clone,
        P: FnMut(&T) -> bool,
    {
        match <Self as BTreeSetExt<T>>::last(self).filter(|k| pred(k)) {
            Some(k) => {
                let k = k.clone();
                self.remove(&k);
                Some(k)
            }
            None => None,
        }
    }
    fn pop_next_if<P>(&mut self, key: &T, mut pred: P) -> Option<T>
    where
        T: Clone,
        P: FnMut(&T) -> bool,
    {
        match self.get_next(key).filter(|k| pred(k)) {
            Some(k) => {
                let k = k.clone();
                self.remove(&k);
                Some(k)
            }
            None => None,
        }
    }
    fn pop_next_excluded_if<P>(&mut self, key: &T, mut pred: P) -> Option<T>
    where
        T: Clone,
        P: FnMut(&T) -> bool,
    {
        match self.get_next_excluded(key).filter(|k| pred(k)) {
            Some(k) => {
                let k = k.clone();
                self.remove(&k);
                Some(k)
            }
            None => None,
        }
    }
    fn pop_next_back_if<P>(&mut self, key: &T, mut pred: P) -> Option<T>
    where
        T: Clone,
        P: FnMut(&T) -> bool,
    {
        match self.get_next_back(key).filter(|k| pred(k)) {
            Some(k) => {
                let k = k.clone();
                self.remove(&k);
                Some(k)
            }
            None => None,
        }
    }
    fn pop_next_back_excluded_if<P>(&mut self, key: &T, mut pred: P) -> Option<T>
    where
        T: Clone,
        P: FnMut(&T) -> bool,
    {
        match self.get_next_back_excluded(key).filter(|k| pred(k)) {
            Some(k) => {
                let k = k.clone();
                self.remove(&k);
                Some(k)
            }
            None => None,
        }
    }
}

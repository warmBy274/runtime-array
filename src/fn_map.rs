use std::{mem::replace, ops::{Deref, DerefMut, Index, IndexMut}};

pub struct FnMap<K, V> {
    index_fn: fn(&K) -> usize,
    buckets: Vec<Option<(usize, K, V)>>
}
impl<K, V> FnMap<K, V> {
    pub fn new(index_fn: fn(&K) -> usize) -> Self {
        Self {
            index_fn,
            buckets: Vec::new()
        }
    }
    pub fn insert(&mut self, key: K, value: V) -> () {
        if self.buckets.len() == 0 {
            self.buckets.push(Some(((self.index_fn)(&key), key, value)));
            return;
        }
        let index = (self.index_fn)(&key);
        let bucket_index = index % self.buckets.len();
        if let Some((i, _, v)) = &mut self.buckets[bucket_index] {
            if *i == index {
                *v = value;
            }
            else {
                self.resize();
                self.insert(key, value);
            }
        }
        else {
            self.buckets[bucket_index] = Some((index, key, value));
        }
    }
    pub fn get(&self, key: &K) -> Option<&V> {
        if self.buckets.len() == 0 {return None;}
        let index = (self.index_fn)(key) % self.buckets.len();
        if let Some((_, _, v)) = &self.buckets[index] {Some(v)}
        else {None}
    }
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if self.buckets.len() == 0 {return None;}
        let index = (self.index_fn)(key) % self.buckets.len();
        if let Some((_, _, v)) = &mut self.buckets[index] {Some(v)}
        else {None}
    }
    pub fn get_key_mut(&mut self, key: &K) -> Option<KeyMutGuard<'_, K, V>> {
        if self.buckets.len() == 0 {return None;}
        let index = (self.index_fn)(key) % self.buckets.len();
        if self.buckets[index].is_some() {
            Some(KeyMutGuard {
                map: self,
                index
            })
        }
        else {None}
    }
    pub fn remove(&mut self, key: &K) -> Option<V> {
        if self.buckets.len() == 0 {return None;}
        let index = (self.index_fn)(key) % self.buckets.len();
        self.buckets[index].take().map(|(_, _, v)| v)
    }
    pub fn remove_pair(&mut self, key: &K) -> Option<(K, V)> {
        if self.buckets.len() == 0 {return None;}
        let index = (self.index_fn)(key) % self.buckets.len();
        self.buckets[index].take().map(|(_, k, v)| (k, v))
    }
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            buckets: &self.buckets,
            pos: 0
        }
    }
    fn resize(&mut self) -> () {
        let new_len = self.buckets.len() * 2;
        let old_buckets = replace(&mut self.buckets, Vec::with_capacity(new_len));
        for _ in 0..new_len {
            self.buckets.push(None);
        }
        for element in old_buckets {
            if let Some((index, key, value)) = element {
                self.buckets[index % new_len] = Some((index, key, value));
            }
        }
    }
}
impl<K: Clone, V: Clone> FnMap<K, V> {
    pub fn clear(&mut self) -> () {
        self.buckets = vec![None; self.buckets.len()];
    }
}
impl<K, V> Index<K> for FnMap<K, V> {
    type Output = V;
    fn index(&self, index: K) -> &Self::Output {
        if let Some(v) = self.get(&index) {v}
        else {panic!("index not in FnMap");}
    }
}
impl<K, V> IndexMut<K> for FnMap<K, V> {
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        if let Some(v) = self.get_mut(&index) {v}
        else {panic!("index not in FnMap");}
    }
}
impl<K, V> IntoIterator for FnMap<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            buckets: self.buckets,
            pos: 0,
        }
    }
}

pub struct Iter<'a, K, V> {
    buckets: &'a [Option<(usize, K, V)>],
    pos: usize,
}
impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.buckets.len() {
            match &self.buckets[self.pos] {
                Some((_, key, value)) => {
                    self.pos += 1;
                    return Some((key, value));
                }
                None => {
                    self.pos += 1;
                }
            }
        }
        None
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.buckets.len() - self.pos;
        (0, Some(remaining))
    }
}

pub struct IntoIter<K, V> {
    buckets: Vec<Option<(usize, K, V)>>,
    pos: usize,
}
impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.buckets.len() {
            let bucket = &mut self.buckets[self.pos];
            self.pos += 1;
            if let Some((_, key, value)) = bucket.take() {
                return Some((key, value));
            }
        }
        None
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.buckets.len() - self.pos;
        (0, Some(remaining))
    }
}

pub struct KeyMutGuard<'a, K, V> {
    map: &'a mut FnMap<K, V>,
    index: usize
}
impl<'a, K, V> Deref for KeyMutGuard<'a, K, V> {
    type Target = K;
    fn deref(&self) -> &Self::Target {
        if let Some((_, k, _)) = &self.map.buckets[self.index] {k}
        else {unreachable!();}
    }
}
impl<'a, K, V> DerefMut for KeyMutGuard<'a, K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if let Some((_, k, _)) = &mut self.map.buckets[self.index] {k}
        else {unreachable!();}
    }
}
impl<'a, K, V> Drop for KeyMutGuard<'a, K, V> {
    fn drop(&mut self) {
        let pair = self.map.buckets[self.index].take().unwrap();
        self.map.insert(pair.1, pair.2);
    }
}
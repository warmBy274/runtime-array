use std::{mem::replace, ops::{Deref, DerefMut, Index}};

pub struct FnSet<V> {
    index_fn: fn(&V) -> usize,
    buckets: Vec<Option<(usize, V)>>
}
impl<V> FnSet<V> {
    pub fn new(index_fn: fn(&V) -> usize) -> Self {
        Self {
            index_fn,
            buckets: Vec::new()
        }
    }
    pub fn insert(&mut self, value: V) -> () {
        if self.buckets.len() == 0 {
            self.buckets.push(Some(((self.index_fn)(&value), value)));
            return;
        }
        let index = (self.index_fn)(&value);
        let bucket_index = index % self.buckets.len();
        if let Some((i, v)) = &mut self.buckets[bucket_index] {
            if *i == index {
                *v = value;
            }
            else {
                self.resize();
                self.insert(value);
            }
        }
        else {
            self.buckets[bucket_index] = Some((index, value));
        }
    }
    pub fn get(&self, id: usize) -> Option<&V> {
        if self.buckets.len() == 0 {return None;}
        let index = id % self.buckets.len();
        if let Some((pair_id, v)) = &self.buckets[index] {
            if *pair_id == id {Some(v)}
            else {None}
        }
        else {None}
    }
    pub fn get_mut(&mut self, id: usize) -> Option<MutGuard<'_, V>> {
        if self.buckets.len() == 0 {return None;}
        let index = id % self.buckets.len();
        if let Some((pair_id, _)) = self.buckets[index] {
            if pair_id == id {
                Some(MutGuard {
                    map: self,
                    index
                })
            }
            else {None}
        }
        else {None}
    }
    pub fn remove(&mut self, id: usize) -> Option<V> {
        if self.buckets.len() == 0 {return None;}
        let index = id % self.buckets.len();
        self.buckets[index].take().map(|(_, v)| v)
    }
    pub fn iter(&self) -> Iter<'_, V> {
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
            if let Some((index, value)) = element {
                self.buckets[index % new_len] = Some((index, value));
            }
        }
    }
}
impl<V> Index<usize> for FnSet<V> {
    type Output = V;
    fn index(&self, index: usize) -> &Self::Output {
        if let Some(v) = self.get(index) {v}
        else {panic!("index {} not in FnMap", index);}
    }
}
impl<V> IntoIterator for FnSet<V> {
    type Item = V;
    type IntoIter = IntoIter<V>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            buckets: self.buckets,
            pos: 0,
        }
    }
}

pub struct Iter<'a, V> {
    buckets: &'a [Option<(usize, V)>],
    pos: usize,
}
impl<'a, V> Iterator for Iter<'a, V> {
    type Item = &'a (usize, V);
    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.buckets.len() {
            match &self.buckets[self.pos] {
                Some(pair) => {
                    self.pos += 1;
                    return Some(pair);
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

pub struct IntoIter<V> {
    buckets: Vec<Option<(usize, V)>>,
    pos: usize,
}
impl<V> Iterator for IntoIter<V> {
    type Item = V;
    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.buckets.len() {
            let bucket = &mut self.buckets[self.pos];
            self.pos += 1;
            if let Some((_, value)) = bucket.take() {
                return Some(value);
            }
        }
        None
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.buckets.len() - self.pos;
        (0, Some(remaining))
    }
}

pub struct MutGuard<'a, T> {
    map: &'a mut FnSet<T>,
    index: usize
}
impl<'a, T> Deref for MutGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.map.buckets[self.index].as_ref().unwrap().1
    }
}
impl<'a, T> DerefMut for MutGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map.buckets[self.index].as_mut().unwrap().1
    }
}
impl<'a, T> Drop for MutGuard<'a, T> {
    fn drop(&mut self) {
        let value = self.map.buckets[self.index].take().unwrap().1;
        self.map.insert(value);
    }
}
use std::collections::hash_map::{DefaultHasher};
use std::hash::{Hash, Hasher};
use std::mem;

const INITIAL_NBUCKETS: usize = 1;

pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    items : usize,
}

impl<K, V> HashMap<K, V>
    where K: Hash + Eq,
{
    pub fn new() -> Self {
        HashMap {
            buckets: Vec::new(),
            items: 0,
        }
    }

    pub fn len(&self) -> usize{
        self.items
    }

    pub fn is_empty(&self) -> bool{
        self.items == 0
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let bucket_idx = self.bucket_idx(key);
        let bucket = &mut self.buckets[bucket_idx];
        let i = bucket.iter().position(|&(ref ekey, _)| ekey==key)?;
        self.items -=1;
        return Some(bucket.swap_remove(i).1);
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        let bucket_idx = self.bucket_idx(key);
        return self.buckets[bucket_idx]
            .iter()
            .find(|&(ref ekey,_)| { ekey == key })
            .map(|&(_, ref v)| v);
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {

        if self.buckets.is_empty() || self.items > 3 * self.buckets.len() /4 {
            self.resize();
        }

        let b_idx = self.bucket_idx(&key);
        let bucket = &mut self.buckets[b_idx];

        for &mut (ref ekey, ref mut evalue) in bucket.iter_mut() {
            if ekey == &key {
                return Some(mem::replace(evalue, value));
            }
        }
        bucket.push((key, value));
        self.items +=1;
        None
    }

    fn bucket_idx(&mut self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        return (hasher.finish() % self.buckets.len() as u64) as usize;
    }

    fn resize(&mut self) {
        let target_size = match self.buckets.len() {
            0 => INITIAL_NBUCKETS,
            n => 2 * n,
        };

        let mut new_buckets = Vec::with_capacity(target_size);
        new_buckets.extend((0..target_size).map(|_| Vec::new()));

        for (key, value) in self.buckets.iter_mut().flat_map(|bucket| bucket.drain(..)) {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            let b_idx = (hasher.finish() % new_buckets.len() as u64) as usize;
            new_buckets[b_idx].push((key, value));
        }

        mem::replace(&mut self.buckets, new_buckets);
    }
}

pub struct Iter<'a, K: 'a, V: 'a>{
    map: &'a HashMap<K,V>,
    bucket: usize,
    at: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V>{
    type Item =  (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.map.buckets.get(self.bucket) {
                Some(bucket) => {
                    match bucket.get(self.at) {
                        Some(&(ref k, ref v)) => {
                            self.at += 1;
                            break Some((k, v));
                        }
                        None => {
                            self.bucket += 1;
                            self.at = 0;
                            continue;
                        }
                    }
                }
                None => {
                    break None
                }
            }
        }
    }
}

impl<'a, K, V> IntoIterator for &'a HashMap<K,V>{
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        Iter { map: self, bucket: 0, at: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut map = HashMap::new();
        map.insert("foo", 43);
        assert_eq!(map.get(&"foo"), Some(&43));
        assert_eq!(map.remove(&"foo"), Some(43));
        assert_eq!(map.get(&"foo"), None);
        assert_eq!(map.remove(&"foo"), None);
    }

    #[test]
    fn test_size() {
        let mut map = HashMap::new();
        map.insert("foo", 43);
        assert_eq!(map.len(), 1);
        assert_eq!(map.is_empty(), false);
        map.remove(&"foo");
        assert_eq!(map.len(), 0);
        assert_eq!(map.is_empty(), true);
    }

    #[test]
    fn test_iter() {
        let mut map = HashMap::new();
        map.insert("foo", 43);
        map.insert("abc", 44);
        map.insert("mmm", 45);

        for (&k, &v) in &map {
            match k {
                "foo" => assert_eq!(v, 43),
                "abc" => assert_eq!(v, 44),
                "mmm" => assert_eq!(v, 45),
                _ => unreachable!(),
            }
        }

        assert_eq!( (&map).into_iter().count(), 3);
    }
}
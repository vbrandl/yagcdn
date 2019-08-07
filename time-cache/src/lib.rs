use std::{
    collections::HashMap,
    hash::Hash,
    time::{Duration, Instant},
};

pub struct Cache<K, V> {
    cache: HashMap<K, CacheEntry<V>>,
    duration: Duration,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash,
{
    pub fn new(duration: Duration) -> Self {
        Self {
            cache: HashMap::new(),
            duration,
        }
    }

    pub fn get(&self, key: &K) -> CacheResult<&V> {
        if let Some(entry) = self.cache.get(key) {
            if Self::is_valid(Instant::now(), entry) {
                CacheResult::Cached(&entry.1)
            } else {
                CacheResult::Invalid
            }
        } else {
            CacheResult::Empty
        }
    }

    pub fn invalidate(&mut self, key: &K) -> bool {
        self.cache.remove(key).is_some()
    }

    pub fn store(&mut self, key: K, value: V) -> Option<V> {
        self.cache
            .insert(key, CacheEntry::new(value, self.duration))
            .map(|old| old.1)
    }

    pub fn clear(&mut self) {
        let now = Instant::now();
        self.cache.retain(|_, v| Self::is_valid(now, v))
    }

    fn is_valid(when: Instant, entry: &CacheEntry<V>) -> bool {
        entry.0 >= when
    }
}

pub enum CacheResult<T> {
    Cached(T),
    Invalid,
    Empty,
}

struct CacheEntry<T>(Instant, T);

impl<T> CacheEntry<T> {
    fn new(value: T, duration: Duration) -> Self {
        CacheEntry(Instant::now() + duration, value)
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

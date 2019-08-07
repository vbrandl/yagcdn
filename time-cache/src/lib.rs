//! Simple cache structure that stores values for a specified time. The cache itself is backed by
//! a HashMap.

use std::{
    collections::HashMap,
    hash::Hash,
    time::{Duration, Instant},
};

/// Time based cache, that stores values for a defined time.
pub struct Cache<K, V> {
    cache: HashMap<K, CacheEntry<V>>,
    duration: Duration,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash,
{
    /// Creates a new cache.
    ///
    /// # Example
    /// ```
    /// use std::time::Duration;
    /// use time_cache::{Cache, CacheResult};
    ///
    /// let cache: Cache<u8, u8> = Cache::new(Duration::from_secs(0));
    /// assert_eq!(CacheResult::Empty, cache.get(&0));
    /// ```
    pub fn new(duration: Duration) -> Self {
        Self {
            cache: HashMap::new(),
            duration,
        }
    }

    /// Get an item from the cache. The item can be either valid, invalid or non existent.
    ///
    /// # Example
    /// ```
    /// use std::time::Duration;
    /// use time_cache::{Cache, CacheResult};
    ///
    /// let key = 0;
    /// let value = 1;
    /// let mut cache: Cache<u8, u8> = Cache::new(Duration::from_secs(0));
    /// assert_eq!(CacheResult::Empty, cache.get(&key));
    /// cache.store(key, value);
    /// assert_eq!(CacheResult::Invalid, cache.get(&key));
    /// ```
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

    /// Removes an item from the cache. Returns `true` if the key was present.
    ///
    /// # Example
    /// ```
    /// use std::time::Duration;
    /// use time_cache::{Cache, CacheResult};
    ///
    /// let key = 0;
    /// let value = 1;
    /// let mut cache: Cache<u8, u8> = Cache::new(Duration::from_secs(0));
    /// assert!(!cache.invalidate(&key));
    /// cache.store(key, value);
    /// assert!(cache.invalidate(&key));
    /// ```
    pub fn invalidate(&mut self, key: &K) -> bool {
        self.cache.remove(key).is_some()
    }

    /// Stores an item in the cache.
    ///
    /// # Example
    /// ```
    /// use std::time::Duration;
    /// use time_cache::{Cache, CacheResult};
    ///
    /// let key = 0;
    /// let value = 1;
    /// let dur = Duration::from_millis(500);
    /// let mut cache: Cache<u8, u8> = Cache::new(dur);
    ///
    /// assert_eq!(CacheResult::Empty, cache.get(&key));
    /// cache.store(key, value);
    /// assert_eq!(CacheResult::Cached(&value), cache.get(&key));
    /// std::thread::sleep(dur);
    /// assert_eq!(CacheResult::Invalid, cache.get(&key));
    /// ```
    pub fn store(&mut self, key: K, value: V) -> Option<V> {
        self.cache
            .insert(key, CacheEntry::new(value, self.duration))
            .map(|old| old.1)
    }

    /// Removes all invalid items from the cache.
    ///
    /// # Example
    /// ```
    /// use std::time::Duration;
    /// use time_cache::{Cache, CacheResult};
    ///
    /// let key = 0;
    /// let value = 1;
    /// let dur = Duration::from_secs(0);
    /// let mut cache: Cache<u8, u8> = Cache::new(dur);
    ///
    /// assert_eq!(CacheResult::Empty, cache.get(&key));
    /// cache.store(key, value);
    /// assert_eq!(CacheResult::Invalid, cache.get(&key));
    /// cache.clear();
    /// assert_eq!(CacheResult::Empty, cache.get(&key));
    /// ```
    pub fn clear(&mut self) {
        let now = Instant::now();
        self.cache.retain(|_, v| Self::is_valid(now, v))
    }

    fn is_valid(when: Instant, entry: &CacheEntry<V>) -> bool {
        entry.0 >= when
    }
}

/// Result when requesting a cached item.
#[derive(Debug, PartialEq)]
pub enum CacheResult<T> {
    /// Item is cached and still valid
    Cached(T),
    /// Item is cached but invalid
    Invalid,
    /// Item is not in the cache
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
    use super::{Cache, CacheResult};
    use std::time::Duration;

    #[test]
    fn always_invalid() {
        let key = 0;
        let value = 1;
        let mut cache = Cache::new(Duration::from_secs(0));
        assert_eq!(CacheResult::Empty, cache.get(&key));
        cache.store(key, value);
        assert_eq!(CacheResult::Invalid, cache.get(&key));
    }

    #[test]
    fn valid() {
        let key = 0;
        let value = 1;
        let mut cache = Cache::new(Duration::from_secs(100));
        assert_eq!(CacheResult::Empty, cache.get(&key));
        cache.store(key, value);
        assert_eq!(CacheResult::Cached(&value), cache.get(&key));
    }

    #[test]
    fn wait_for_invalidation() {
        let key = 0;
        let value = 1;
        let dur = Duration::from_millis(500);
        let mut cache = Cache::new(dur);
        assert_eq!(CacheResult::Empty, cache.get(&key));
        cache.store(key, value);
        assert_eq!(CacheResult::Cached(&value), cache.get(&key));
        std::thread::sleep(dur);
        assert_eq!(CacheResult::Invalid, cache.get(&key));
    }

    #[test]
    fn invalidate() {
        let key = 0;
        let value = 1;
        let dur = Duration::from_secs(100);
        let mut cache = Cache::new(dur);
        assert_eq!(CacheResult::Empty, cache.get(&key));
        cache.store(key, value);
        assert_eq!(CacheResult::Cached(&value), cache.get(&key));
        assert!(cache.invalidate(&key));
        assert_eq!(CacheResult::Empty, cache.get(&key));
    }

    #[test]
    fn invalidate_wait() {
        let key = 0;
        let value = 1;
        let dur = Duration::from_millis(500);
        let mut cache = Cache::new(dur);
        assert_eq!(CacheResult::Empty, cache.get(&key));
        cache.store(key, value);
        assert_eq!(CacheResult::Cached(&value), cache.get(&key));
        std::thread::sleep(dur);
        assert_eq!(CacheResult::Invalid, cache.get(&key));
        assert!(cache.invalidate(&key));
        assert_eq!(CacheResult::Empty, cache.get(&key));
    }

    #[test]
    fn clear() {
        let key = 0;
        let value = 1;
        let dur = Duration::from_secs(0);
        let mut cache = Cache::new(dur);
        assert_eq!(CacheResult::Empty, cache.get(&key));
        cache.store(key, value);
        assert_eq!(CacheResult::Invalid, cache.get(&key));
        cache.clear();
        assert_eq!(CacheResult::Empty, cache.get(&key));
    }
}

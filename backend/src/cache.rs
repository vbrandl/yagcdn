use std::{
    collections::HashMap,
    hash::Hash,
    time::{Duration, Instant},
};

pub(crate) struct Cache<K, V> {
    cache: HashMap<K, CacheEntry<V>>,
    duration: Duration,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash,
{
    pub(crate) fn new() -> Self {
        Self {
            cache: HashMap::new(),
            duration: Duration::from_secs(5 * 60),
        }
    }

    pub(crate) fn get(&self, key: &K) -> CacheResult<&V> {
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

    pub(crate) fn invalidate(&mut self, key: &K) -> bool {
        self.cache.remove(key).is_some()
    }

    pub(crate) fn store(&mut self, key: K, value: V) -> Option<V> {
        self.cache
            .insert(key, CacheEntry::new(value, self.duration))
            .map(|old| old.1)
    }

    pub(crate) fn clear(&mut self) {
        let now = Instant::now();
        self.cache.retain(|_, v| !Self::is_valid(now, v));
    }

    fn is_valid(when: Instant, entry: &CacheEntry<V>) -> bool {
        entry.0 >= when
    }
}

pub(crate) enum CacheResult<T> {
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

#[derive(Eq, PartialEq, Hash, Debug)]
pub(crate) struct Key(Service, String, String, String);

#[derive(Eq, PartialEq, Hash, Debug)]
pub(crate) enum Service {
    GitHub,
    GitLab,
    Bitbucket,
}

impl Key {
    pub(crate) fn new(service: Service, user: String, repo: String, branch: String) -> Self {
        Key(service, user, repo, branch)
    }
}

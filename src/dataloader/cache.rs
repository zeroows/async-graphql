use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

/// Factory for creating cache storage.
pub trait CacheFactory: Send + Sync + 'static {
    /// Create a cache storage.
    ///
    /// TODO: When GAT is stable, this memory allocation can be optimized away.
    fn create<K, V>(&self) -> Box<dyn CacheStorage<Key = K, Value = V>>
    where
        K: Send + Sync + Clone + Eq + Hash + 'static,
        V: Send + Sync + Clone + 'static;
}

/// Cache storage for [DataLoader].
pub trait CacheStorage: Send + Sync + 'static {
    /// The key type of the record.
    type Key: Send + Sync + Clone + Eq + Hash + 'static;

    /// The value type of the record.
    type Value: Send + Sync + Clone + 'static;

    /// Returns a reference to the value of the key in the cache or None if it is not present in the cache.
    fn get(&mut self, key: &Self::Key) -> Option<&Self::Value>;

    /// Puts a key-value pair into the cache. If the key already exists in the cache, then it updates the key's value.
    fn insert(&mut self, key: Cow<'_, Self::Key>, val: Cow<'_, Self::Value>);

    /// Removes the value corresponding to the key from the cache.
    fn remove(&mut self, key: &Self::Key);

    /// Clears the cache, removing all key-value pairs.
    fn clear(&mut self);
}

/// No cache.
pub struct NoCache;

impl CacheFactory for NoCache {
    fn create<K, V>(&self) -> Box<dyn CacheStorage<Key = K, Value = V>>
    where
        K: Send + Sync + Clone + Eq + Hash + 'static,
        V: Send + Sync + Clone + 'static,
    {
        Box::new(NoCacheImpl {
            _mark1: PhantomData,
            _mark2: PhantomData,
        })
    }
}

struct NoCacheImpl<K, V> {
    _mark1: PhantomData<K>,
    _mark2: PhantomData<V>,
}

impl<K, V> CacheStorage for NoCacheImpl<K, V>
where
    K: Send + Sync + Clone + Eq + Hash + 'static,
    V: Send + Sync + Clone + 'static,
{
    type Key = K;
    type Value = V;

    #[inline]
    fn get(&mut self, _key: &K) -> Option<&V> {
        None
    }

    #[inline]
    fn insert(&mut self, _key: Cow<'_, Self::Key>, _val: Cow<'_, Self::Value>) {}

    #[inline]
    fn remove(&mut self, _key: &K) {}

    #[inline]
    fn clear(&mut self) {}
}

/// [std::collections::HashMap] cache.
pub struct HashMapCache;

impl CacheFactory for HashMapCache {
    fn create<K, V>(&self) -> Box<dyn CacheStorage<Key = K, Value = V>>
    where
        K: Send + Sync + Clone + Eq + Hash + 'static,
        V: Send + Sync + Clone + 'static,
    {
        Box::new(HashMapCacheImpl(Default::default()))
    }
}

struct HashMapCacheImpl<K, V>(HashMap<K, V>);

impl<K, V> CacheStorage for HashMapCacheImpl<K, V>
where
    K: Send + Sync + Clone + Eq + Hash + 'static,
    V: Send + Sync + Clone + 'static,
{
    type Key = K;
    type Value = V;

    #[inline]
    fn get(&mut self, key: &Self::Key) -> Option<&Self::Value> {
        self.0.get(key)
    }

    #[inline]
    fn insert(&mut self, key: Cow<'_, Self::Key>, val: Cow<'_, Self::Value>) {
        self.0.insert(key.into_owned(), val.into_owned());
    }

    #[inline]
    fn remove(&mut self, key: &Self::Key) {
        self.0.remove(key);
    }

    #[inline]
    fn clear(&mut self) {
        self.0.clear();
    }
}

/// LRU cache.
pub struct LruCache {
    cap: usize,
}

impl LruCache {
    /// Creates a new LRU Cache that holds at most `cap` items.
    pub fn new(cap: usize) -> Self {
        Self { cap }
    }
}

impl CacheFactory for LruCache {
    fn create<K, V>(&self) -> Box<dyn CacheStorage<Key = K, Value = V>>
    where
        K: Send + Sync + Clone + Eq + Hash + 'static,
        V: Send + Sync + Clone + 'static,
    {
        Box::new(LruCacheImpl(lru::LruCache::new(self.cap)))
    }
}

struct LruCacheImpl<K, V>(lru::LruCache<K, V>);

impl<K, V> CacheStorage for LruCacheImpl<K, V>
where
    K: Send + Sync + Clone + Eq + Hash + 'static,
    V: Send + Sync + Clone + 'static,
{
    type Key = K;
    type Value = V;

    #[inline]
    fn get(&mut self, key: &Self::Key) -> Option<&Self::Value> {
        self.0.get(key)
    }

    #[inline]
    fn insert(&mut self, key: Cow<'_, Self::Key>, val: Cow<'_, Self::Value>) {
        self.0.put(key.into_owned(), val.into_owned());
    }

    #[inline]
    fn remove(&mut self, key: &Self::Key) {
        self.0.pop(key);
    }

    #[inline]
    fn clear(&mut self) {
        self.0.clear();
    }
}

use lru::LruCache;
use std::hash::Hash;
use std::sync::Mutex;

pub struct MyCache<K, V> {
    inner: Mutex<LruCache<K, V>>,
}

impl<K, V> MyCache<K, V>
where
    K: Hash + Eq + PartialEq + Clone,
    V: Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self { inner: Mutex::new(LruCache::<K, V>::new(capacity)) }
    }

    pub fn get_or_insert<F>(&self, key: K, f: F) -> V
    where
        F: FnOnce(&K) -> V,
        V: Clone,
    {
        if let Some(result) = self.get(&key) {
            return result;
        }
        let val = f(&key);
        let val_clone = val.clone();
        self.inner.lock().unwrap().put(key, val_clone);
        val
    }

    pub fn insert(&self, key: K, v: V) {
        self.inner.lock().unwrap().put(key, v);
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.inner.lock().unwrap().get(key).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache() {
        let cache = MyCache::<u64, Vec<u64>>::new(100);

        assert_eq!(cache.get(&0u64), None);
        assert_eq!(cache.get_or_insert(123u64, |key| vec![*key, 123]), vec![123u64, 123]);
        assert_eq!(cache.get(&123u64), Some(vec![123u64, 123]));
        assert_eq!(cache.get(&0u64), None);
    }
}

use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, Mutex, MutexGuard},
};

#[derive(Debug, Clone)]
struct Discovery<Candidate, Probe> {
    candidates: Vec<Candidate>,
    explored: Vec<Probe>,
}

impl<Candidate, Probe> Default for Discovery<Candidate, Probe> {
    fn default() -> Self {
        Self {
            candidates: Vec::new(),
            explored: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DiscoveryCache<Candidate, Probe> {
    candidates: Arc<Mutex<HashMap<(String, String), Discovery<Candidate, Probe>>>>,
    routes: Arc<Mutex<HashMap<(String, String), Vec<Candidate>>>>,
}

#[derive(Debug, Clone)]
pub(crate) struct ValueCache<K, V> {
    values: Arc<Mutex<HashMap<K, V>>>,
}

impl<Candidate, Probe> Default for DiscoveryCache<Candidate, Probe> {
    fn default() -> Self {
        Self {
            candidates: Arc::new(Mutex::new(HashMap::new())),
            routes: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<K, V> Default for ValueCache<K, V> {
    fn default() -> Self {
        Self {
            values: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<Candidate, Probe> DiscoveryCache<Candidate, Probe>
where
    Candidate: Clone + PartialEq,
    Probe: Clone + PartialEq,
{
    pub fn get(&self, from: &str, to: &str) -> (Vec<Candidate>, Vec<Probe>) {
        let cache = lock(&self.candidates);
        match cache.get(&Self::pool_key(from, to)) {
            Some(discovery) => (discovery.candidates.clone(), discovery.explored.clone()),
            None => (Vec::new(), Vec::new()),
        }
    }

    pub fn put(&self, from: &str, to: &str, candidates: &[Candidate], explored: &[Probe]) {
        if candidates.is_empty() && explored.is_empty() {
            return;
        }
        let mut cache = lock(&self.candidates);
        let entry = cache.entry(Self::pool_key(from, to)).or_default();
        for candidate in candidates {
            if !entry.candidates.contains(candidate) {
                entry.candidates.push(candidate.clone());
            }
        }
        for probe in explored {
            if !entry.explored.contains(probe) {
                entry.explored.push(probe.clone());
            }
        }
    }

    pub fn get_route(&self, from: &str, to: &str) -> Option<Vec<Candidate>> {
        let cache = lock(&self.routes);
        cache.get(&Self::route_key(from, to)).cloned()
    }

    pub fn put_route(&self, from: &str, to: &str, route: &[Candidate]) {
        if route.is_empty() {
            return;
        }
        let mut cache = lock(&self.routes);
        cache.insert(Self::route_key(from, to), route.to_vec());
    }

    fn pool_key(from: &str, to: &str) -> (String, String) {
        let (a, b) = if from <= to { (from, to) } else { (to, from) };
        (a.to_string(), b.to_string())
    }

    fn route_key(from: &str, to: &str) -> (String, String) {
        (from.to_string(), to.to_string())
    }
}

impl<K, V> ValueCache<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    pub fn get(&self, key: &K) -> Option<V> {
        let values = lock(&self.values);
        values.get(key).cloned()
    }

    pub fn put(&self, key: K, value: V) {
        let mut values = lock(&self.values);
        values.insert(key, value);
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool(id: &str) -> String {
        id.to_string()
    }

    #[test]
    fn test_pool_key_is_direction_insensitive() {
        assert_eq!(DiscoveryCache::<String, u32>::pool_key("0xa", "0xb"), DiscoveryCache::<String, u32>::pool_key("0xb", "0xa"));
    }

    #[test]
    fn test_route_key_is_direction_sensitive() {
        assert_ne!(
            DiscoveryCache::<String, u32>::route_key("0xa", "0xb"),
            DiscoveryCache::<String, u32>::route_key("0xb", "0xa")
        );
    }

    #[test]
    fn test_cache_merges_candidates_and_probes_across_passes() {
        let cache = DiscoveryCache::default();
        cache.put("0xa", "0xb", &[pool("0x1")], &[60, 200]);
        cache.put("0xa", "0xb", &[pool("0x2"), pool("0x1")], &[10, 2]);

        let (pools, probes) = cache.get("0xa", "0xb");
        assert_eq!(pools, vec![pool("0x1"), pool("0x2")]);
        assert_eq!(probes, vec![60, 200, 10, 2]);
    }

    #[test]
    fn test_cache_tracks_explored_probes_when_no_candidates_found() {
        let cache = DiscoveryCache::<String, u32>::default();
        cache.put("0xa", "0xb", &[], &[60, 200]);
        let (pools, probes) = cache.get("0xa", "0xb");
        assert!(pools.is_empty());
        assert_eq!(probes, vec![60, 200]);
    }

    #[test]
    fn test_route_roundtrip() {
        let cache = DiscoveryCache::<String, u32>::default();
        let route = vec![pool("0x1"), pool("0x2")];
        cache.put_route("USDC", "WAL", &route);
        assert_eq!(cache.get_route("USDC", "WAL").unwrap(), route);
        assert!(cache.get_route("WAL", "USDC").is_none());
    }

    #[test]
    fn test_put_route_skips_empty() {
        let cache = DiscoveryCache::<String, u32>::default();
        cache.put_route("USDC", "WAL", &[]);
        assert!(cache.get_route("USDC", "WAL").is_none());
    }

    #[test]
    fn test_value_cache_roundtrip() {
        let cache = ValueCache::default();
        cache.put(("router".to_string(), "jetton".to_string()), "wallet".to_string());

        assert_eq!(cache.get(&("router".to_string(), "jetton".to_string())), Some("wallet".to_string()));
        assert_eq!(cache.get(&("router".to_string(), "other".to_string())), None);
    }
}

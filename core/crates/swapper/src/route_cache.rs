use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Mutex, MutexGuard},
    time::Instant,
};

use crate::config::DiscoveryCacheConfig;

type PairKey = (String, String);

#[derive(Debug)]
struct Discovery<Candidate, Probe> {
    candidates: Vec<Candidate>,
    explored: Vec<Probe>,
}

#[derive(Debug)]
struct Cached<T> {
    value: T,
    expires_at: Instant,
}

#[derive(Debug)]
pub(crate) struct DiscoveryCache<Candidate, Probe> {
    discoveries: Mutex<HashMap<PairKey, Cached<Discovery<Candidate, Probe>>>>,
    routes: Mutex<HashMap<PairKey, Cached<Vec<Candidate>>>>,
    config: DiscoveryCacheConfig,
}

#[derive(Debug)]
pub(crate) struct ValueCache<K, V> {
    values: Mutex<HashMap<K, V>>,
}

impl<Candidate, Probe> Default for DiscoveryCache<Candidate, Probe> {
    fn default() -> Self {
        Self::new(DiscoveryCacheConfig::default())
    }
}

impl<Candidate, Probe> DiscoveryCache<Candidate, Probe> {
    pub fn new(config: DiscoveryCacheConfig) -> Self {
        Self {
            discoveries: Mutex::new(HashMap::new()),
            routes: Mutex::new(HashMap::new()),
            config,
        }
    }
}

impl<K, V> Default for ValueCache<K, V> {
    fn default() -> Self {
        Self {
            values: Mutex::new(HashMap::new()),
        }
    }
}

impl<Candidate, Probe> DiscoveryCache<Candidate, Probe>
where
    Candidate: Clone + PartialEq,
    Probe: Clone + PartialEq,
{
    pub fn get(&self, from: &str, to: &str) -> (Vec<Candidate>, Vec<Probe>) {
        let now = Instant::now();
        let key = Self::pool_key(from, to);
        let cache = lock(&self.discoveries);
        let Some(cached) = cache.get(&key) else {
            return (Vec::new(), Vec::new());
        };
        if cached.expires_at <= now {
            return (Vec::new(), Vec::new());
        }
        let discovery = &cached.value;
        (discovery.candidates.clone(), discovery.explored.clone())
    }

    pub fn contains_candidate(&self, from: &str, to: &str, candidate: &Candidate) -> bool {
        let cache = lock(&self.discoveries);
        cache
            .get(&Self::pool_key(from, to))
            .is_some_and(|cached| cached.expires_at > Instant::now() && cached.value.candidates.contains(candidate))
    }

    pub fn put(&self, from: &str, to: &str, candidates: &[Candidate], explored: &[Probe]) {
        if candidates.is_empty() && explored.is_empty() {
            return;
        }
        let now = Instant::now();
        let key = Self::pool_key(from, to);
        let expires_at = now + self.config.expiration;
        lock(&self.discoveries)
            .entry(key)
            .and_modify(|cached| {
                cached.value = if cached.expires_at <= now {
                    Discovery {
                        candidates: candidates.to_vec(),
                        explored: explored.to_vec(),
                    }
                } else {
                    Discovery {
                        candidates: Self::merged(&cached.value.candidates, candidates),
                        explored: Self::merged(&cached.value.explored, explored),
                    }
                };
                cached.expires_at = expires_at;
            })
            .or_insert_with(|| Cached {
                value: Discovery {
                    candidates: candidates.to_vec(),
                    explored: explored.to_vec(),
                },
                expires_at,
            });
    }

    pub fn get_route(&self, from: &str, to: &str) -> Option<Vec<Candidate>> {
        let now = Instant::now();
        let key = Self::route_key(from, to);
        let cache = lock(&self.routes);
        let cached = cache.get(&key)?;
        if cached.expires_at <= now {
            return None;
        }
        Some(cached.value.clone())
    }

    pub fn put_route(&self, from: &str, to: &str, route: &[Candidate]) {
        if route.is_empty() {
            return;
        }
        lock(&self.routes).insert(
            Self::route_key(from, to),
            Cached {
                value: route.to_vec(),
                expires_at: Instant::now() + self.config.expiration,
            },
        );
    }

    fn merged<T: Clone + PartialEq>(values: &[T], additions: &[T]) -> Vec<T> {
        values
            .iter()
            .cloned()
            .chain(
                additions
                    .iter()
                    .enumerate()
                    .filter(|(index, addition)| !values.contains(addition) && !additions[..*index].contains(addition))
                    .map(|(_, addition)| addition.clone()),
            )
            .collect()
    }

    fn pool_key(from: &str, to: &str) -> PairKey {
        let (a, b) = if from <= to { (from, to) } else { (to, from) };
        (a.to_string(), b.to_string())
    }

    fn route_key(from: &str, to: &str) -> PairKey {
        (from.to_string(), to.to_string())
    }
}

impl<K, V> ValueCache<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    pub fn get(&self, key: &K) -> Option<V> {
        lock(&self.values).get(key).cloned()
    }

    pub fn put(&self, key: K, value: V) {
        lock(&self.values).insert(key, value);
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
    use std::time::Duration;

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
        assert!(cache.contains_candidate("0xb", "0xa", &pool("0x2")));
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
    fn test_configured_expiration() {
        let cache = DiscoveryCache::new(DiscoveryCacheConfig { expiration: Duration::ZERO });
        cache.put("0xa", "0xb", &[pool("0x1")], &[60, 200]);

        assert_eq!(cache.get("0xa", "0xb"), (Vec::<String>::new(), Vec::<u32>::new()));
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
    fn test_route_expiration_and_empty_route() {
        let cache = DiscoveryCache::<String, u32>::new(DiscoveryCacheConfig { expiration: Duration::ZERO });
        cache.put_route("USDC", "WAL", &[pool("0x1")]);
        assert!(cache.get_route("USDC", "WAL").is_none());

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

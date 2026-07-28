use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Mutex, MutexGuard},
    time::Instant,
};

use crate::config::RouteCacheConfig;

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
pub(crate) struct RouteCache<Candidate, Probe> {
    discoveries: Mutex<HashMap<PairKey, Cached<Discovery<Candidate, Probe>>>>,
    routes: Mutex<HashMap<PairKey, Cached<Vec<Candidate>>>>,
    config: RouteCacheConfig,
}

#[derive(Debug)]
pub(crate) struct ValueCache<K, V> {
    values: Mutex<HashMap<K, Cached<V>>>,
    config: RouteCacheConfig,
}

impl<Candidate, Probe> Default for RouteCache<Candidate, Probe> {
    fn default() -> Self {
        Self::new(RouteCacheConfig::default())
    }
}

impl<Candidate, Probe> RouteCache<Candidate, Probe> {
    pub fn new(config: RouteCacheConfig) -> Self {
        Self {
            discoveries: Mutex::new(HashMap::new()),
            routes: Mutex::new(HashMap::new()),
            config,
        }
    }
}

impl<K, V> Default for ValueCache<K, V> {
    fn default() -> Self {
        Self::new(RouteCacheConfig::default())
    }
}

impl<K, V> ValueCache<K, V> {
    pub fn new(config: RouteCacheConfig) -> Self {
        Self {
            values: Mutex::new(HashMap::new()),
            config,
        }
    }
}

impl<Candidate, Probe> RouteCache<Candidate, Probe>
where
    Candidate: Clone + PartialEq,
    Probe: Clone + PartialEq,
{
    pub fn get_discovery(&self, from: &str, to: &str) -> (Vec<Candidate>, Vec<Probe>) {
        let now = Instant::now();
        let key = Self::pair_key(from, to);
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

    pub fn missing_probes(&self, from: &str, to: &str, probes: &[Probe]) -> Vec<Probe> {
        let (_, explored) = self.get_discovery(from, to);
        probes.iter().filter(|probe| !explored.contains(probe)).cloned().collect()
    }

    pub fn has_candidate(&self, from: &str, to: &str, candidate: &Candidate) -> bool {
        let cache = lock(&self.discoveries);
        cache
            .get(&Self::pair_key(from, to))
            .is_some_and(|cached| cached.expires_at > Instant::now() && cached.value.candidates.contains(candidate))
    }

    pub fn record_discovery(&self, from: &str, to: &str, discoveries: impl IntoIterator<Item = (Probe, Option<Candidate>)>) {
        let discoveries = discoveries.into_iter().collect::<Vec<_>>();
        let candidates = discoveries.iter().filter_map(|(_, candidate)| candidate.clone()).collect::<Vec<_>>();
        let explored = discoveries.into_iter().map(|(probe, _)| probe).collect::<Vec<_>>();
        if candidates.is_empty() && explored.is_empty() {
            return;
        }
        let now = Instant::now();
        let key = Self::pair_key(from, to);
        let expires_at = now + self.config.expiration;
        lock(&self.discoveries)
            .entry(key)
            .and_modify(|cached| {
                cached.value = if cached.expires_at <= now {
                    Discovery {
                        candidates: candidates.clone(),
                        explored: explored.clone(),
                    }
                } else {
                    Discovery {
                        candidates: Self::merged(&cached.value.candidates, &candidates),
                        explored: Self::merged(&cached.value.explored, &explored),
                    }
                };
                cached.expires_at = expires_at;
            })
            .or_insert_with(|| Cached {
                value: Discovery { candidates, explored },
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

    pub fn record_route(&self, from: &str, to: &str, route: &[Candidate]) {
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

    fn pair_key(from: &str, to: &str) -> PairKey {
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
        lock(&self.values)
            .get(key)
            .filter(|cached| cached.expires_at > Instant::now())
            .map(|cached| cached.value.clone())
    }

    pub fn put(&self, key: K, value: V) {
        lock(&self.values).insert(
            key,
            Cached {
                value,
                expires_at: Instant::now() + self.config.expiration,
            },
        );
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
    fn test_pair_key_is_direction_insensitive() {
        assert_eq!(RouteCache::<String, u32>::pair_key("0xa", "0xb"), RouteCache::<String, u32>::pair_key("0xb", "0xa"));
    }

    #[test]
    fn test_route_key_is_direction_sensitive() {
        assert_ne!(RouteCache::<String, u32>::route_key("0xa", "0xb"), RouteCache::<String, u32>::route_key("0xb", "0xa"));
    }

    #[test]
    fn test_cache_merges_candidates_and_probes_across_passes() {
        let cache = RouteCache::default();
        cache.record_discovery("0xa", "0xb", [(60, Some(pool("0x1"))), (200, None)]);
        cache.record_discovery("0xa", "0xb", [(10, Some(pool("0x2"))), (2, Some(pool("0x1")))]);

        let (pools, probes) = cache.get_discovery("0xa", "0xb");
        assert_eq!(pools, vec![pool("0x1"), pool("0x2")]);
        assert_eq!(probes, vec![60, 200, 10, 2]);
        assert!(cache.has_candidate("0xb", "0xa", &pool("0x2")));
        assert_eq!(cache.missing_probes("0xa", "0xb", &[2, 3, 10]), vec![3]);
    }

    #[test]
    fn test_cache_tracks_explored_probes_when_no_candidates_found() {
        let cache = RouteCache::<String, u32>::default();
        cache.record_discovery("0xa", "0xb", [(60, None), (200, None)]);
        let (pools, probes) = cache.get_discovery("0xa", "0xb");
        assert!(pools.is_empty());
        assert_eq!(probes, vec![60, 200]);
    }

    #[test]
    fn test_configured_expiration() {
        let cache = RouteCache::new(RouteCacheConfig { expiration: Duration::ZERO });
        cache.record_discovery("0xa", "0xb", [(60, Some(pool("0x1"))), (200, None)]);

        assert_eq!(cache.get_discovery("0xa", "0xb"), (Vec::<String>::new(), Vec::<u32>::new()));
    }

    #[test]
    fn test_route_roundtrip() {
        let cache = RouteCache::<String, u32>::default();
        let route = vec![pool("0x1"), pool("0x2")];
        cache.record_route("USDC", "WAL", &route);
        assert_eq!(cache.get_route("USDC", "WAL").unwrap(), route);
        assert!(cache.get_route("WAL", "USDC").is_none());
    }

    #[test]
    fn test_route_expiration_and_empty_route() {
        let cache = RouteCache::<String, u32>::new(RouteCacheConfig { expiration: Duration::ZERO });
        cache.record_route("USDC", "WAL", &[pool("0x1")]);
        assert!(cache.get_route("USDC", "WAL").is_none());

        cache.record_route("USDC", "WAL", &[]);
        assert!(cache.get_route("USDC", "WAL").is_none());
    }

    #[test]
    fn test_value_cache_roundtrip() {
        let cache = ValueCache::default();
        cache.put(("router".to_string(), "jetton".to_string()), "wallet".to_string());

        assert_eq!(cache.get(&("router".to_string(), "jetton".to_string())), Some("wallet".to_string()));
        assert_eq!(cache.get(&("router".to_string(), "other".to_string())), None);

        let expired = ValueCache::new(RouteCacheConfig { expiration: Duration::ZERO });
        expired.put("router", "wallet");
        assert_eq!(expired.get(&"router"), None);
    }
}

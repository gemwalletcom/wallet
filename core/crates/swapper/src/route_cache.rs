use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Mutex, MutexGuard},
    time::{Duration, Instant},
};

use primitives::DAY;

type PairKey<Key> = (Key, Key);

#[derive(Clone, Debug)]
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
pub(crate) struct Cache<Key, Value> {
    values: Mutex<HashMap<Key, Cached<Value>>>,
    ttl: Duration,
}

#[derive(Debug)]
pub(crate) struct RouteCache<Candidate, Probe, Key = String> {
    discoveries: Cache<PairKey<Key>, Discovery<Candidate, Probe>>,
    routes: Cache<PairKey<Key>, Vec<Candidate>>,
}

impl<Key, Value> Default for Cache<Key, Value> {
    fn default() -> Self {
        Self::new(DAY)
    }
}

impl<Key, Value> Cache<Key, Value> {
    fn new(ttl: Duration) -> Self {
        Self {
            values: Mutex::new(HashMap::new()),
            ttl,
        }
    }
}

impl<Candidate, Probe, Key> Default for RouteCache<Candidate, Probe, Key> {
    fn default() -> Self {
        Self::new(DAY)
    }
}

impl<Candidate, Probe, Key> RouteCache<Candidate, Probe, Key> {
    fn new(ttl: Duration) -> Self {
        Self {
            discoveries: Cache::new(ttl),
            routes: Cache::new(ttl),
        }
    }
}

impl<Candidate, Probe, Key> RouteCache<Candidate, Probe, Key>
where
    Candidate: Clone + PartialEq,
    Probe: Clone + PartialEq,
    Key: Eq + Hash + Ord,
{
    pub fn get_discovery(&self, from: impl Into<Key>, to: impl Into<Key>) -> (Vec<Candidate>, Vec<Probe>) {
        self.discoveries
            .get(&Self::pair_key(from, to))
            .map(|discovery| (discovery.candidates, discovery.explored))
            .unwrap_or_default()
    }

    pub fn missing_probes(&self, from: impl Into<Key>, to: impl Into<Key>, probes: &[Probe]) -> Vec<Probe> {
        let (_, explored) = self.get_discovery(from, to);
        probes.iter().filter(|probe| !explored.contains(probe)).cloned().collect()
    }

    pub fn record_discovery(&self, from: impl Into<Key>, to: impl Into<Key>, discoveries: impl IntoIterator<Item = (Probe, Option<Candidate>)>) {
        let (explored, candidates): (Vec<_>, Vec<_>) = discoveries.into_iter().unzip();
        let candidates = candidates.into_iter().flatten().collect::<Vec<_>>();
        if candidates.is_empty() && explored.is_empty() {
            return;
        }
        self.discoveries.update(Self::pair_key(from, to), |cached| match cached {
            Some(cached) => Discovery {
                candidates: Self::merged(&cached.candidates, &candidates),
                explored: Self::merged(&cached.explored, &explored),
            },
            None => Discovery { candidates, explored },
        });
    }

    pub fn get_route(&self, from: impl Into<Key>, to: impl Into<Key>) -> Option<Vec<Candidate>> {
        self.routes.get(&Self::route_key(from, to))
    }

    pub fn record_route(&self, from: impl Into<Key>, to: impl Into<Key>, route: &[Candidate]) {
        if !route.is_empty() {
            self.routes.put(Self::route_key(from, to), route.to_vec());
        }
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

    fn pair_key(from: impl Into<Key>, to: impl Into<Key>) -> PairKey<Key> {
        let from = from.into();
        let to = to.into();
        if from <= to { (from, to) } else { (to, from) }
    }

    fn route_key(from: impl Into<Key>, to: impl Into<Key>) -> PairKey<Key> {
        (from.into(), to.into())
    }
}

impl<Key, Value> Cache<Key, Value>
where
    Key: Eq + Hash,
    Value: Clone,
{
    pub fn get(&self, key: &Key) -> Option<Value> {
        let now = Instant::now();
        let mut values = lock(&self.values);
        if values.get(key).is_some_and(|cached| cached.expires_at <= now) {
            values.remove(key);
            return None;
        }
        values.get(key).map(|cached| cached.value.clone())
    }

    pub fn put(&self, key: Key, value: Value) {
        lock(&self.values).insert(
            key,
            Cached {
                value,
                expires_at: Instant::now() + self.ttl,
            },
        );
    }

    fn update(&self, key: Key, update: impl FnOnce(Option<Value>) -> Value) {
        let now = Instant::now();
        let mut values = lock(&self.values);
        let value = values.get(&key).filter(|cached| cached.expires_at > now).map(|cached| cached.value.clone());
        values.insert(
            key,
            Cached {
                value: update(value),
                expires_at: now + self.ttl,
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

    #[test]
    fn test_route_keys() {
        assert_eq!(RouteCache::<String, u32>::pair_key("0xa", "0xb"), RouteCache::<String, u32>::pair_key("0xb", "0xa"));
        assert_ne!(RouteCache::<String, u32>::route_key("0xa", "0xb"), RouteCache::<String, u32>::route_key("0xb", "0xa"));
    }

    #[test]
    fn test_discovery_cache() {
        let cache = RouteCache::<String, u32>::default();
        cache.record_discovery("0xa", "0xb", [(60, Some("0x1".to_string())), (200, None)]);
        cache.record_discovery("0xa", "0xb", [(10, Some("0x2".to_string())), (2, Some("0x1".to_string()))]);

        let (pools, probes) = cache.get_discovery("0xa", "0xb");
        assert_eq!(pools, vec!["0x1".to_string(), "0x2".to_string()]);
        assert_eq!(probes, vec![60, 200, 10, 2]);
        assert_eq!(cache.get_discovery("0xb", "0xa").0, pools);
        assert_eq!(cache.missing_probes("0xa", "0xb", &[2, 3, 10]), vec![3]);

        let cache = RouteCache::<String, u32>::default();
        cache.record_discovery("0xa", "0xb", [(60, None), (200, None)]);
        let (pools, probes) = cache.get_discovery("0xa", "0xb");
        assert!(pools.is_empty());
        assert_eq!(probes, vec![60, 200]);

        let cache = RouteCache::<String, u32>::new(Duration::ZERO);
        cache.record_discovery("0xa", "0xb", [(60, Some("0x1".to_string())), (200, None)]);

        assert_eq!(cache.get_discovery("0xa", "0xb"), (Vec::<String>::new(), Vec::<u32>::new()));
    }

    #[test]
    fn test_route_cache() {
        let cache = RouteCache::<String, u32>::default();
        let route = vec!["0x1".to_string(), "0x2".to_string()];
        cache.record_route("USDC", "WAL", &route);
        assert_eq!(cache.get_route("USDC", "WAL").unwrap(), route);
        assert!(cache.get_route("WAL", "USDC").is_none());

        let cache = RouteCache::<String, u32>::new(Duration::ZERO);
        cache.record_route("USDC", "WAL", &["0x1".to_string()]);
        assert!(cache.get_route("USDC", "WAL").is_none());
        cache.record_route("USDC", "WAL", &[]);
        assert!(cache.get_route("USDC", "WAL").is_none());
    }

    #[test]
    fn test_value_cache() {
        let cache = Cache::default();
        cache.put(("router".to_string(), "jetton".to_string()), "wallet".to_string());
        assert_eq!(cache.get(&("router".to_string(), "jetton".to_string())), Some("wallet".to_string()));
        assert_eq!(cache.get(&("router".to_string(), "other".to_string())), None);

        let expired = Cache::new(Duration::ZERO);
        expired.put("router", "wallet");
        assert_eq!(expired.get(&"router"), None);
    }
}

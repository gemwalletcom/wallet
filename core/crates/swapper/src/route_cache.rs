use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Mutex, MutexGuard},
    time::{Duration, Instant},
};

use primitives::DAY;

type PairKey<Key> = (Key, Key);

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
pub(crate) struct DiscoveryCache<Candidate, Probe, Key = String> {
    discoveries: Cache<PairKey<Key>, HashMap<Probe, Option<Candidate>>>,
}

impl<Key, Value> Default for Cache<Key, Value> {
    fn default() -> Self {
        Self::new(DAY)
    }
}

impl<Key, Value> Cache<Key, Value> {
    pub(crate) fn new(ttl: Duration) -> Self {
        Self {
            values: Mutex::new(HashMap::new()),
            ttl,
        }
    }
}

impl<Candidate, Probe, Key> Default for DiscoveryCache<Candidate, Probe, Key> {
    fn default() -> Self {
        Self::new(DAY)
    }
}

impl<Candidate, Probe, Key> DiscoveryCache<Candidate, Probe, Key> {
    fn new(ttl: Duration) -> Self {
        Self { discoveries: Cache::new(ttl) }
    }
}

impl<Candidate, Probe, Key> DiscoveryCache<Candidate, Probe, Key>
where
    Candidate: Clone + PartialEq,
    Probe: Clone + Eq + Hash,
    Key: Eq + Hash + Ord,
{
    pub fn candidates_for_probes(&self, from: impl Into<Key>, to: impl Into<Key>, probes: &[Probe]) -> Vec<Candidate> {
        let Some(discovery) = self.discoveries.get(&Self::pair_key(from, to)) else {
            return Vec::new();
        };
        let candidates = probes.iter().filter_map(|probe| discovery.get(probe).and_then(Option::as_ref)).collect::<Vec<_>>();
        candidates
            .iter()
            .enumerate()
            .filter(|(index, candidate)| !candidates[..*index].contains(candidate))
            .map(|(_, candidate)| (*candidate).clone())
            .collect()
    }

    pub fn missing_probes(&self, from: impl Into<Key>, to: impl Into<Key>, probes: &[Probe]) -> Vec<Probe> {
        let discovery = self.discoveries.get(&Self::pair_key(from, to));
        probes
            .iter()
            .filter(|probe| discovery.as_ref().is_none_or(|discovery| !discovery.contains_key(*probe)))
            .cloned()
            .collect()
    }

    pub fn record_discovery(&self, from: impl Into<Key>, to: impl Into<Key>, facts: impl IntoIterator<Item = (Probe, Option<Candidate>)>) {
        let facts = facts.into_iter().collect::<HashMap<_, _>>();
        if facts.is_empty() {
            return;
        }
        self.discoveries.update(Self::pair_key(from, to), |discovery| {
            discovery.extend(facts);
        });
    }

    fn pair_key(from: impl Into<Key>, to: impl Into<Key>) -> PairKey<Key> {
        let from = from.into();
        let to = to.into();
        if from <= to { (from, to) } else { (to, from) }
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
}

impl<Key, Value> Cache<Key, Value>
where
    Key: Eq + Hash,
    Value: Default,
{
    fn update(&self, key: Key, update: impl FnOnce(&mut Value)) {
        let now = Instant::now();
        let mut values = lock(&self.values);
        values.retain(|_, cached| cached.expires_at > now);
        let cached = values.entry(key).or_insert_with(|| Cached {
            value: Value::default(),
            expires_at: now + self.ttl,
        });
        update(&mut cached.value);
        cached.expires_at = now + self.ttl;
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
    fn test_discovery_cache() {
        let cache = DiscoveryCache::<&str, u32>::default();
        cache.record_discovery("0xa", "0xb", [(60, Some("0x1")), (200, None)]);
        cache.record_discovery("0xa", "0xb", [(10, Some("0x2")), (2, Some("0x1"))]);

        assert_eq!(cache.candidates_for_probes("0xa", "0xb", &[60]), vec!["0x1"]);
        assert_eq!(cache.candidates_for_probes("0xa", "0xb", &[10]), vec!["0x2"]);
        assert_eq!(cache.candidates_for_probes("0xa", "0xb", &[2, 10, 60, 200]), vec!["0x1", "0x2"]);
        assert_eq!(cache.candidates_for_probes("0xb", "0xa", &[10]), vec!["0x2"]);
        assert_eq!(cache.missing_probes("0xa", "0xb", &[2, 3, 10, 200]), vec![3]);

        cache.record_discovery("0xa", "0xb", [(60, None), (200, Some("0x3"))]);

        assert!(cache.candidates_for_probes("0xa", "0xb", &[60]).is_empty());
        assert_eq!(cache.candidates_for_probes("0xa", "0xb", &[200]), vec!["0x3"]);
    }

    #[test]
    fn test_discovery_cache_expiry() {
        let cache = DiscoveryCache::<&str, u32>::new(Duration::ZERO);
        cache.record_discovery("0xa", "0xb", [(60, Some("0x1")), (200, None)]);

        assert!(cache.candidates_for_probes("0xa", "0xb", &[60]).is_empty());
        assert_eq!(cache.missing_probes("0xa", "0xb", &[60, 200]), vec![60, 200]);
    }

    #[test]
    fn test_value_cache() {
        let cache = Cache::default();
        cache.put(("router", "jetton"), "wallet");
        assert_eq!(cache.get(&("router", "jetton")), Some("wallet"));
        assert_eq!(cache.get(&("router", "other")), None);

        let expired = Cache::new(Duration::ZERO);
        expired.put("router", "wallet");
        assert_eq!(expired.get(&"router"), None);
    }
}

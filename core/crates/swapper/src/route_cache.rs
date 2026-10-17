use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Mutex, MutexGuard},
    time::{Duration, Instant},
};

use primitives::{DAY, HOUR, WEEK};

use crate::SwapperError;

const OUTPRICED_BPS: u128 = 300;
const BPS: u128 = 10_000;

type PairKey<Key> = (Key, Key);
type Discoveries<Key, Probe, Candidate> = HashMap<PairKey<Key>, HashMap<Probe, Cached<Option<Candidate>>>>;

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

#[derive(Debug, Clone, Copy)]
struct DiscoveryTtl {
    found: Duration,
    missing: Duration,
    unusable: Duration,
}

#[derive(Debug)]
pub(crate) struct DiscoveryCache<Candidate, Probe, Key = String> {
    discoveries: Mutex<Discoveries<Key, Probe, Candidate>>,
    ttl: DiscoveryTtl,
}

#[derive(Debug)]
pub(crate) struct OutpricedCache<Probe, Key = String> {
    losses: Cache<(Key, Key, Probe), u128>,
}

impl<Probe, Key> Default for OutpricedCache<Probe, Key> {
    fn default() -> Self {
        Self { losses: Cache::new(HOUR) }
    }
}

impl<Probe, Key> OutpricedCache<Probe, Key>
where
    Probe: Clone + Eq + Hash,
    Key: Clone + Eq + Hash,
{
    pub fn partition<T>(&self, from: &Key, to: &Key, amount_in: u128, items: Vec<T>, probe: impl Fn(&T) -> Probe) -> (Vec<T>, Vec<T>) {
        items.into_iter().partition(|item| !self.is_outpriced(from, to, &probe(item), amount_in))
    }

    pub async fn quote_live_first<T, R, Fut>(&self, (from, to): (&Key, &Key), amount_in: u128, items: Vec<T>, probe: impl Fn(&T) -> Probe, quote: impl Fn(Vec<T>) -> Fut) -> Result<R, SwapperError>
    where
        Fut: Future<Output = Result<R, SwapperError>>,
    {
        let (live, skipped) = self.partition(from, to, amount_in, items, probe);
        if live.is_empty() {
            return Err(SwapperError::NoQuoteAvailable);
        }
        match quote(live).await {
            Err(SwapperError::NoQuoteAvailable) if !skipped.is_empty() => quote(skipped).await,
            result => result,
        }
    }

    pub fn record(&self, from: &Key, to: &Key, amount_in: u128, quotes: impl IntoIterator<Item = (Probe, u128)>) {
        let quotes = quotes.into_iter().collect::<Vec<_>>();
        let Some(best) = quotes.iter().map(|(_, amount_out)| *amount_out).max() else {
            return;
        };
        quotes
            .into_iter()
            .filter(|(_, amount_out)| *amount_out == 0 || amount_out.saturating_mul(BPS) < best.saturating_mul(BPS - OUTPRICED_BPS))
            .for_each(|(probe, _)| {
                let key = (from.clone(), to.clone(), probe);
                let lost_at = self.losses.get(&key).map_or(amount_in, |lost_at| lost_at.min(amount_in));
                self.losses.put(key, lost_at);
            });
    }

    fn is_outpriced(&self, from: &Key, to: &Key, probe: &Probe, amount_in: u128) -> bool {
        self.losses.get(&(from.clone(), to.clone(), probe.clone())).is_some_and(|lost_at| amount_in >= lost_at)
    }
}

impl<Key, Value> Default for Cache<Key, Value> {
    fn default() -> Self {
        Self::new(DAY)
    }
}

impl<Key, Value> Cache<Key, Value> {
    pub(crate) fn new(ttl: Duration) -> Self {
        Self { values: Mutex::new(HashMap::new()), ttl }
    }
}

impl<Candidate, Probe, Key> Default for DiscoveryCache<Candidate, Probe, Key> {
    fn default() -> Self {
        Self::new(DiscoveryTtl { found: WEEK, missing: DAY, unusable: HOUR })
    }
}

impl<Candidate, Probe, Key> DiscoveryCache<Candidate, Probe, Key> {
    fn new(ttl: DiscoveryTtl) -> Self {
        Self {
            discoveries: Mutex::new(HashMap::new()),
            ttl,
        }
    }
}

impl<Candidate, Probe, Key> DiscoveryCache<Candidate, Probe, Key>
where
    Candidate: Clone + PartialEq,
    Probe: Clone + Eq + Hash,
    Key: Eq + Hash + Ord,
{
    pub fn candidates_for_probes(&self, from: impl Into<Key>, to: impl Into<Key>, probes: &[Probe]) -> Vec<Candidate> {
        let discovery = self.discovery(from, to);
        let candidates = probes.iter().filter_map(|probe| discovery.get(probe).and_then(Option::as_ref)).collect::<Vec<_>>();
        candidates
            .iter()
            .enumerate()
            .filter(|(index, candidate)| !candidates[..*index].contains(candidate))
            .map(|(_, candidate)| (*candidate).clone())
            .collect()
    }

    pub fn candidates_with(&self, from: impl Into<Key>, to: impl Into<Key>, probes: &[Probe], known: Vec<Candidate>, same: impl Fn(&Candidate, &Candidate) -> bool) -> Vec<Candidate> {
        let cached = self.candidates_for_probes(from, to, probes).into_iter().filter(|candidate| known.iter().all(|pool| !same(pool, candidate))).collect::<Vec<_>>();
        [known, cached].concat()
    }

    pub fn missing_probes(&self, from: impl Into<Key>, to: impl Into<Key>, probes: &[Probe]) -> Vec<Probe> {
        let discovery = self.discovery(from, to);
        probes.iter().filter(|probe| !discovery.contains_key(*probe)).cloned().collect()
    }

    pub fn record_discovery(&self, from: impl Into<Key>, to: impl Into<Key>, facts: impl IntoIterator<Item = (Probe, Option<Candidate>)>) {
        let (found, missing) = (self.ttl.found, self.ttl.missing);
        self.record(
            from,
            to,
            facts.into_iter().map(|(probe, candidate)| {
                let ttl = if candidate.is_some() { found } else { missing };
                (probe, candidate, ttl)
            }),
        );
    }

    pub fn record_unusable(&self, from: impl Into<Key>, to: impl Into<Key>, probes: impl IntoIterator<Item = Probe>) {
        let unusable = self.ttl.unusable;
        self.record(from, to, probes.into_iter().map(|probe| (probe, None, unusable)));
    }

    fn discovery(&self, from: impl Into<Key>, to: impl Into<Key>) -> HashMap<Probe, Option<Candidate>> {
        let now = Instant::now();
        lock(&self.discoveries)
            .get(&Self::pair_key(from, to))
            .into_iter()
            .flatten()
            .filter(|(_, cached)| cached.expires_at > now)
            .map(|(probe, cached)| (probe.clone(), cached.value.clone()))
            .collect()
    }

    fn record(&self, from: impl Into<Key>, to: impl Into<Key>, facts: impl Iterator<Item = (Probe, Option<Candidate>, Duration)>) {
        let now = Instant::now();
        let facts = facts.map(|(probe, value, ttl)| (probe, Cached { value, expires_at: now + ttl })).collect::<HashMap<_, _>>();
        if facts.is_empty() {
            return;
        }
        Self::insert(&mut lock(&self.discoveries), Self::pair_key(from, to), facts, now);
    }

    fn insert(discoveries: &mut Discoveries<Key, Probe, Candidate>, key: PairKey<Key>, facts: HashMap<Probe, Cached<Option<Candidate>>>, now: Instant) {
        discoveries.values_mut().for_each(|discovery| discovery.retain(|_, cached| cached.expires_at > now));
        discoveries.retain(|_, discovery| !discovery.is_empty());
        discoveries.entry(key).or_default().extend(facts);
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
        assert_eq!(cache.candidates_with("0xa", "0xb", &[2, 10], vec!["0x9", "0x1"], |left, right| left == right), vec!["0x9", "0x1", "0x2"]);

        cache.record_discovery("0xa", "0xb", [(60, None), (200, Some("0x3"))]);

        assert!(cache.candidates_for_probes("0xa", "0xb", &[60]).is_empty());
        assert_eq!(cache.candidates_for_probes("0xa", "0xb", &[200]), vec!["0x3"]);
    }

    #[test]
    fn test_discovery_cache_expiry() {
        let ttl = |found, missing, unusable| DiscoveryTtl { found, missing, unusable };
        let expired = DiscoveryCache::<&str, u32>::new(ttl(Duration::ZERO, Duration::ZERO, Duration::ZERO));
        expired.record_discovery("0xa", "0xb", [(60, Some("0x1")), (200, None)]);

        assert!(expired.candidates_for_probes("0xa", "0xb", &[60]).is_empty());
        assert_eq!(expired.missing_probes("0xa", "0xb", &[60, 200]), vec![60, 200]);

        let cache = DiscoveryCache::<&str, u32>::new(ttl(DAY, Duration::ZERO, DAY));
        cache.record_discovery("0xa", "0xb", [(60, Some("0x1")), (10, Some("0x2")), (200, None)]);

        assert_eq!(cache.candidates_for_probes("0xa", "0xb", &[60, 10]), vec!["0x1", "0x2"]);
        assert_eq!(cache.missing_probes("0xa", "0xb", &[60, 10, 200]), vec![200]);

        cache.record_unusable("0xa", "0xb", [10]);

        assert_eq!(cache.candidates_for_probes("0xa", "0xb", &[60, 10]), vec!["0x1"]);
        assert_eq!(cache.missing_probes("0xa", "0xb", &[60, 10, 200]), vec![200]);
    }

    #[test]
    fn test_outpriced_cache() {
        let cache = OutpricedCache::<&str, &str>::default();
        let pools = || vec!["deep", "close", "shallow"];
        cache.record(&"sol", &"usdc", 100, [("deep", 1_000), ("close", 975), ("shallow", 969)]);

        assert_eq!(cache.partition(&"sol", &"usdc", 100, pools(), |pool| *pool), (vec!["deep", "close"], vec!["shallow"]));
        assert_eq!(cache.partition(&"sol", &"usdc", 500, pools(), |pool| *pool), (vec!["deep", "close"], vec!["shallow"]));
        assert_eq!(cache.partition(&"sol", &"usdc", 50, pools(), |pool| *pool), (pools(), vec![]));
        assert_eq!(cache.partition(&"usdc", &"sol", 100, pools(), |pool| *pool), (pools(), vec![]));

        cache.record(&"sol", &"usdc", 40, [("deep", 1_000), ("shallow", 500)]);
        assert_eq!(cache.partition(&"sol", &"usdc", 50, pools(), |pool| *pool), (vec!["deep", "close"], vec!["shallow"]));

        cache.record(&"usdc", &"sol", 10, [("deep", 0), ("close", 0)]);
        assert_eq!(cache.partition(&"usdc", &"sol", 10, pools(), |pool| *pool), (vec!["shallow"], vec!["deep", "close"]));
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

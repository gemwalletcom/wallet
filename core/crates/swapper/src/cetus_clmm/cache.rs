use super::model::DiscoveredPool;
use crate::route_cache::DiscoveryCache;

pub(super) type PoolCache = DiscoveryCache<DiscoveredPool, u32>;

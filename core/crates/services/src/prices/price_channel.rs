use cacher::CacheKey;
use primitives::AssetId;

pub fn price_channel(asset_id: &AssetId) -> String {
    CacheKey::Price(&asset_id.to_string()).key()
}

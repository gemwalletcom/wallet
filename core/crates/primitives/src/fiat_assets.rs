use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::currency::Currency;
use crate::{AssetId, FiatProviderName, PaymentType};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct FiatAssets {
    pub version: u32,
    pub asset_ids: Vec<String>,
}

impl FiatAssets {
    pub fn new(asset_ids: Vec<String>) -> Self {
        Self {
            version: Self::version(&asset_ids),
            asset_ids,
        }
    }

    pub fn version(asset_ids: &[String]) -> u32 {
        let mut ids: Vec<&String> = asset_ids.iter().collect();
        ids.sort();
        ids.dedup();
        let hash = ids.iter().fold(0x811c_9dc5_u32, |hash, id| {
            id.bytes()
                .chain(std::iter::once(0))
                .fold(hash, |hash, byte| (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193))
        });
        hash & 0x7fff_ffff
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatAssetLimits {
    pub currency: Currency,
    pub payment_type: PaymentType,
    pub min_amount: Option<f64>,
    pub max_amount: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatAsset {
    pub id: String,
    pub asset_id: Option<AssetId>,
    pub provider: FiatProviderName,
    pub symbol: String,
    pub network: Option<String>,
    pub token_id: Option<String>,
    pub enabled: bool,
    pub is_buy_enabled: bool,
    pub is_sell_enabled: bool,
    pub unsupported_countries: HashMap<String, Vec<String>>,
    pub buy_limits: Vec<FiatAssetLimits>,
    pub sell_limits: Vec<FiatAssetLimits>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn version_depends_on_asset_set_only() {
        let ordered = FiatAssets::version(&ids(&["bitcoin", "ethereum", "solana"]));

        assert_eq!(ordered, FiatAssets::version(&ids(&["solana", "bitcoin", "ethereum"])));
        assert_eq!(ordered, FiatAssets::version(&ids(&["solana", "bitcoin", "ethereum", "bitcoin"])));
    }

    #[test]
    fn version_changes_when_asset_set_changes() {
        let base = FiatAssets::version(&ids(&["bitcoin", "ethereum"]));

        assert_ne!(base, FiatAssets::version(&ids(&["bitcoin", "solana"])));
        assert_ne!(base, FiatAssets::version(&ids(&["bitcoin"])));
        assert_ne!(base, FiatAssets::version(&ids(&["bitcoin", "ethereum", "solana"])));
        assert_eq!(base, FiatAssets::new(ids(&["ethereum", "bitcoin"])).version);
    }
}

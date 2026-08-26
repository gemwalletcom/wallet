use std::collections::HashSet;

use chrono::Utc;
use primitives::{AssetId, AssetPrice, AssetType, Chain, PerpetualPosition, PerpetualProvider};

use crate::models::asset::wallet_default_assets;

pub fn provider(chain: Chain) -> PerpetualProvider {
    match chain {
        Chain::HyperCore => PerpetualProvider::Hypercore,
        _ => PerpetualProvider::Hypercore,
    }
}

pub fn prices_outdated(updated_at: Option<i64>, now: i64, interval_seconds: u32) -> bool {
    updated_at.is_none_or(|updated_at| now - updated_at >= i64::from(interval_seconds))
}

pub fn stale_position_ids(existing_ids: Vec<String>, positions: &[PerpetualPosition]) -> Vec<String> {
    let current: HashSet<&str> = positions.iter().map(|position| position.id.as_str()).collect();
    existing_ids.into_iter().filter(|id| !current.contains(id.as_str())).collect()
}

pub fn collateral_asset_id(chain: Chain) -> Option<AssetId> {
    wallet_default_assets(chain)
        .into_iter()
        .find(|asset| asset.asset_type == AssetType::PERPETUAL)
        .map(|asset| asset.id)
}

pub fn collateral_price(chain: Chain) -> Option<AssetPrice> {
    collateral_asset_id(chain).map(|asset_id| AssetPrice::new(asset_id, 1.0, 0.0, Utc::now()))
}

#[cfg(test)]
mod tests {
    use super::*;

    use primitives::{PerpetualDirection, PerpetualId, PerpetualMarginType};

    fn position(id: &str) -> PerpetualPosition {
        PerpetualPosition {
            id: id.into(),
            perpetual_id: PerpetualId::new(PerpetualProvider::Hypercore, "BTC"),
            asset_id: AssetId::from_chain(Chain::HyperCore),
            size: 1.0,
            size_value: 1.0,
            leverage: 1,
            entry_price: 1.0,
            liquidation_price: None,
            margin_type: PerpetualMarginType::Cross,
            direction: PerpetualDirection::Long,
            margin_amount: 1.0,
            take_profit: None,
            stop_loss: None,
            pnl: 0.0,
            funding: None,
        }
    }

    #[test]
    fn test_stale_position_ids_keeps_only_positions_that_disappeared() {
        let stale = stale_position_ids(vec!["a".into(), "b".into()], &[position("b"), position("c")]);

        assert_eq!(stale, vec!["a".to_string()]);
    }

    #[test]
    fn test_hypercore_collateral_is_priced_at_one_dollar() {
        let price = collateral_price(Chain::HyperCore).unwrap();

        assert_eq!(price.asset_id.chain, Chain::HyperCore);
        assert_eq!(price.price, 1.0);
    }

    #[test]
    fn test_prices_outdated() {
        assert!(prices_outdated(None, 100, 5));
        assert!(prices_outdated(Some(95), 100, 5));
        assert!(!prices_outdated(Some(97), 100, 5));
    }
}

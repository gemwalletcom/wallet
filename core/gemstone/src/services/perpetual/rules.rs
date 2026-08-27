use chrono::Utc;
use number_formatter::{BigNumberFormatter, NumberFormatterError};
use primitives::known_assets::HYPERCORE_PERPETUAL_USDC;
use primitives::perpetual::PerpetualBalance;
use primitives::{AssetId, AssetPrice, AssetType, Chain, PerpetualAccountMode, PerpetualPosition, PerpetualProvider};

use crate::models::asset::wallet_default_assets;
use crate::services::balance::{GemBalanceUpdate, GemBalanceUpdateType, GemBalanceValue};
use crate::services::collections::stale;

const MARKETS_REFRESH_INTERVAL_SECONDS: i64 = 60 * 60;

pub fn includes_perpetual_collateral(mode: PerpetualAccountMode) -> bool {
    match mode {
        PerpetualAccountMode::Standard => true,
        PerpetualAccountMode::Unified => false,
    }
}

pub fn is_markets_stale(updated_at: Option<i64>, now: i64) -> bool {
    updated_at.is_none_or(|updated_at| now - updated_at >= MARKETS_REFRESH_INTERVAL_SECONDS)
}

pub fn balance_update(balance: &PerpetualBalance) -> Result<GemBalanceUpdate, NumberFormatterError> {
    let asset = &*HYPERCORE_PERPETUAL_USDC;
    let value = |amount: f64| -> Result<GemBalanceValue, NumberFormatterError> {
        Ok(GemBalanceValue {
            value: BigNumberFormatter::value_from_amount(&amount.to_string(), asset.decimals as u32)?,
            amount,
        })
    };
    Ok(GemBalanceUpdate {
        asset_id: asset.id.clone(),
        update_type: GemBalanceUpdateType::Perpetual {
            available: value(balance.available)?,
            reserved: value(balance.reserved)?,
            withdrawable: value(balance.withdrawable)?,
        },
        is_active: true,
    })
}

pub fn provider(chain: Chain) -> Option<PerpetualProvider> {
    match chain {
        Chain::HyperCore | Chain::Hyperliquid => Some(PerpetualProvider::Hypercore),
        _ => None,
    }
}

pub fn prices_outdated(updated_at: Option<i64>, now: i64, interval_seconds: u32) -> bool {
    updated_at.is_none_or(|updated_at| now - updated_at >= i64::from(interval_seconds))
}

pub fn stale_position_ids(existing_ids: Vec<String>, positions: &[PerpetualPosition]) -> Vec<String> {
    stale(existing_ids, positions.iter().map(|position| position.id.clone()))
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

    #[test]
    fn test_perpetual_collateral_counts_only_in_standard_mode() {
        assert!(includes_perpetual_collateral(PerpetualAccountMode::Standard));
        assert!(!includes_perpetual_collateral(PerpetualAccountMode::Unified));
    }

    #[test]
    fn test_markets_stale_after_an_hour_or_when_never_synced() {
        assert!(is_markets_stale(None, 10_000));
        assert!(!is_markets_stale(Some(10_000 - 3_599), 10_000));
        assert!(is_markets_stale(Some(10_000 - 3_600), 10_000));
    }

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

    #[test]
    fn test_balance_update_targets_perpetual_usdc() {
        let update = balance_update(&PerpetualBalance {
            available: 1234.5,
            reserved: 0.25,
            withdrawable: 1000.0,
        })
        .unwrap();
        assert_eq!(update.asset_id, HYPERCORE_PERPETUAL_USDC.id);
        assert!(update.is_active);
        match update.update_type {
            GemBalanceUpdateType::Perpetual {
                available,
                reserved,
                withdrawable,
            } => {
                assert_eq!(available.value, "1234500000");
                assert_eq!(available.amount, 1234.5);
                assert_eq!(reserved.value, "250000");
                assert_eq!(withdrawable.value, "1000000000");
            }
            update_type => panic!("expected a perpetual update, got {update_type:?}"),
        }
    }
}

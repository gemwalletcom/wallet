use chrono::Utc;
use number_formatter::{BigNumberFormatter, NumberFormatterError};
use primitives::known_assets::HYPERCORE_PERPETUAL_USDC;
use primitives::perpetual::{PerpetualBalance, PerpetualData};
use primitives::{
    AssetBasic, AssetId, AssetPrice, AssetProperties, AssetScore, AssetType, Chain, PerpetualAccountMode, PerpetualDirection, PerpetualPosition, PerpetualProvider, Wallet,
    WalletType,
};

use super::model::{GemAutocloseSummary, GemPerpetualCloseInput, GemPerpetualOrderAction, GemPerpetualOrderInput};
use crate::perpetual::GemPerpetual;
use crate::services::error::GemServiceError;
use num_bigint::BigInt;
use primitives::{PerpetualConfirmData, PerpetualModifyConfirmData, PerpetualModifyPositionType, PerpetualReduceData, PerpetualType};
use std::collections::HashSet;
use std::str::FromStr;

use crate::models::asset::wallet_default_assets;
use crate::services::balance::{GemBalanceUpdate, GemBalanceUpdateType, GemBalanceValue};
use crate::services::collections::stale;

const MARKETS_REFRESH_INTERVAL_SECONDS: i64 = 60 * 60;

const DEFAULT_SLIPPAGE_PERCENT: f64 = 2.0;
const HOURS_PER_YEAR: f64 = 24.0 * 365.0;

pub fn perpetual_asset_basics(data: &[PerpetualData]) -> Vec<AssetBasic> {
    data.iter()
        .map(|item| {
            AssetBasic::new(
                item.asset.clone(),
                AssetProperties {
                    is_enabled: false,
                    is_buyable: false,
                    is_sellable: false,
                    is_swapable: false,
                    is_stakeable: false,
                    staking_apr: None,
                    is_earnable: false,
                    earn_apr: None,
                    has_image: false,
                    has_price: false,
                },
                AssetScore::new(0),
            )
        })
        .collect()
}

pub fn autoclose_summary(data: &PerpetualModifyConfirmData) -> Option<GemAutocloseSummary> {
    let orders = data.modify_types.iter().find_map(|modify| match modify {
        PerpetualModifyPositionType::Tpsl(orders) => Some(orders),
        PerpetualModifyPositionType::Cancel(_) => None,
    });
    let canceled: HashSet<u64> = data
        .modify_types
        .iter()
        .filter_map(|modify| match modify {
            PerpetualModifyPositionType::Cancel(orders) => Some(orders),
            PerpetualModifyPositionType::Tpsl(_) => None,
        })
        .flatten()
        .map(|order| order.order_id)
        .collect();

    let take_profit = orders.and_then(|orders| orders.take_profit.as_ref()).and_then(|value| value.parse::<f64>().ok());
    let stop_loss = orders.and_then(|orders| orders.stop_loss.as_ref()).and_then(|value| value.parse::<f64>().ok());
    let take_profit_cleared = take_profit.is_none() && data.take_profit_order_id.is_some_and(|id| canceled.contains(&id));
    let stop_loss_cleared = stop_loss.is_none() && data.stop_loss_order_id.is_some_and(|id| canceled.contains(&id));

    if take_profit.is_none() && stop_loss.is_none() && !take_profit_cleared && !stop_loss_cleared {
        return None;
    }
    Some(GemAutocloseSummary {
        take_profit,
        stop_loss,
        take_profit_cleared,
        stop_loss_cleared,
    })
}

pub fn funding_apr(funding: f64) -> f64 {
    funding * HOURS_PER_YEAR
}

pub fn slippage_percent(slippage: Option<f64>) -> f64 {
    slippage.unwrap_or(DEFAULT_SLIPPAGE_PERCENT)
}

pub fn opens_position(action: &GemPerpetualOrderAction) -> bool {
    match action {
        GemPerpetualOrderAction::Open | GemPerpetualOrderAction::Increase => true,
        GemPerpetualOrderAction::Reduce { .. } => false,
    }
}

pub fn slippage_price(market_price: f64, direction: PerpetualDirection, opens: bool, slippage: f64) -> f64 {
    let fraction = slippage / 100.0;
    let multiplier = match (direction, opens) {
        (PerpetualDirection::Long, true) | (PerpetualDirection::Short, false) => 1.0 + fraction,
        (PerpetualDirection::Long, false) | (PerpetualDirection::Short, true) => 1.0 - fraction,
    };
    market_price * multiplier
}

pub fn order_amounts(usd_amount: f64, leverage: u8, price: f64) -> (f64, f64, f64) {
    let size = (usd_amount * f64::from(leverage)) / price;
    let fiat_value = price * size;
    let margin_amount = fiat_value / f64::from(leverage);
    (size, fiat_value, margin_amount)
}

pub fn includes_perpetual_collateral(mode: PerpetualAccountMode) -> bool {
    match mode {
        PerpetualAccountMode::Standard => true,
        PerpetualAccountMode::Unified => false,
    }
}

pub fn show_perpetuals(enabled: bool, wallet: &Wallet) -> bool {
    enabled && wallet.wallet_type == WalletType::Multicoin && crate::services::stream::rules::hyperliquid_account(&wallet.accounts).is_some()
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

pub fn order(provider: PerpetualProvider, input: GemPerpetualOrderInput) -> Result<PerpetualType, GemServiceError> {
    let usdc_amount = BigInt::from_str(&input.usdc_amount).map_err(|error| GemServiceError::InvalidInput { msg: error.to_string() })?;
    let usd_amount = usdc_amount.to_string().parse::<f64>().unwrap_or_default() / 10f64.powi(input.usdc_decimals);
    let slippage = slippage_percent(input.slippage);
    let (size, fiat_value, margin_amount) = order_amounts(usd_amount, input.leverage, input.price);
    let price = slippage_price(input.price, input.direction.clone(), opens_position(&input.action), slippage);
    let formatter = GemPerpetual::new(provider);

    let data = PerpetualConfirmData {
        direction: input.direction,
        margin_type: input.margin_type,
        base_asset: input.base_asset,
        asset_index: input.asset_index,
        price: formatter.format_price(price, input.asset.decimals),
        fiat_value,
        size: formatter.format_size(size, input.asset.decimals),
        slippage,
        leverage: input.leverage,
        pnl: None,
        entry_price: None,
        market_price: input.price,
        margin_amount,
        take_profit: input.take_profit,
        stop_loss: input.stop_loss,
    };

    Ok(match input.action {
        GemPerpetualOrderAction::Open => PerpetualType::Open(data),
        GemPerpetualOrderAction::Increase => PerpetualType::Increase(data),
        GemPerpetualOrderAction::Reduce { position_direction } => PerpetualType::Reduce(PerpetualReduceData { data, position_direction }),
    })
}

pub fn close_order(provider: PerpetualProvider, input: GemPerpetualCloseInput) -> PerpetualConfirmData {
    let slippage = slippage_percent(input.slippage);
    let price = slippage_price(input.market_price, input.direction.clone(), false, slippage);
    let size = input.size.abs();
    let formatter = GemPerpetual::new(provider);

    PerpetualConfirmData {
        direction: input.direction,
        margin_type: input.margin_type,
        base_asset: input.base_asset,
        asset_index: input.asset_index,
        price: formatter.format_price(price, input.asset.decimals),
        fiat_value: size * price,
        size: formatter.format_size(size, input.asset.decimals),
        slippage,
        leverage: input.leverage,
        pnl: Some(input.pnl),
        entry_price: Some(input.entry_price),
        market_price: input.market_price,
        margin_amount: input.margin_amount,
        take_profit: None,
        stop_loss: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Asset, PerpetualDirection, PerpetualId, PerpetualMarginType};

    fn modify_data(modify_types: Vec<PerpetualModifyPositionType>, take_profit_order_id: Option<u64>, stop_loss_order_id: Option<u64>) -> PerpetualModifyConfirmData {
        PerpetualModifyConfirmData {
            base_asset: Asset::mock(),
            asset_index: 0,
            modify_types,
            take_profit_order_id,
            stop_loss_order_id,
        }
    }

    fn tpsl(take_profit: Option<&str>, stop_loss: Option<&str>) -> PerpetualModifyPositionType {
        PerpetualModifyPositionType::Tpsl(primitives::perpetual::TPSLOrderData {
            direction: PerpetualDirection::Long,
            take_profit: take_profit.map(|value| value.to_string()),
            stop_loss: stop_loss.map(|value| value.to_string()),
            size: "1".to_string(),
        })
    }

    fn cancel(order_ids: Vec<u64>) -> PerpetualModifyPositionType {
        PerpetualModifyPositionType::Cancel(
            order_ids
                .into_iter()
                .map(|order_id| primitives::perpetual::CancelOrderData { asset_index: 0, order_id })
                .collect(),
        )
    }

    #[test]
    fn test_autoclose_summary_reads_new_prices_and_cleared_orders() {
        let both = autoclose_summary(&modify_data(vec![tpsl(Some("65000"), Some("55000"))], None, None)).unwrap();
        assert_eq!(both.take_profit, Some(65000.0));
        assert_eq!(both.stop_loss, Some(55000.0));
        assert!(!both.take_profit_cleared && !both.stop_loss_cleared);

        let cleared = autoclose_summary(&modify_data(vec![cancel(vec![111, 222])], Some(111), Some(222))).unwrap();
        assert_eq!(cleared.take_profit, None);
        assert_eq!(cleared.stop_loss, None);
        assert!(cleared.take_profit_cleared && cleared.stop_loss_cleared);

        let replaced = autoclose_summary(&modify_data(vec![tpsl(Some("70000"), None), cancel(vec![111])], Some(111), None)).unwrap();
        assert_eq!(replaced.take_profit, Some(70000.0));
        assert!(!replaced.take_profit_cleared, "a replaced order is not a cleared one");

        assert!(autoclose_summary(&modify_data(vec![], None, None)).is_none());
        assert!(
            autoclose_summary(&modify_data(vec![cancel(vec![999])], Some(111), None)).is_none(),
            "cancelling an unrelated order leaves nothing to show"
        );
    }

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

    #[test]
    fn test_show_perpetuals_needs_flag_multicoin_and_hyperliquid_account() {
        let wallet = |wallet_type: WalletType, chains: &[Chain]| Wallet {
            id: primitives::WalletId::Multicoin("w".to_string()),
            external_id: None,
            name: "w".to_string(),
            index: 0,
            wallet_type,
            accounts: chains
                .iter()
                .map(|chain| primitives::Account {
                    chain: *chain,
                    address: "a".to_string(),
                    derivation_path: String::new(),
                    extended_public_key: None,
                })
                .collect(),
            is_pinned: false,
            image_url: None,
            source: primitives::WalletSource::Import,
        };

        assert!(show_perpetuals(true, &wallet(WalletType::Multicoin, &[Chain::Arbitrum])));
        assert!(!show_perpetuals(false, &wallet(WalletType::Multicoin, &[Chain::Arbitrum])));
        assert!(!show_perpetuals(true, &wallet(WalletType::Single, &[Chain::Arbitrum])));
        assert!(!show_perpetuals(true, &wallet(WalletType::Multicoin, &[Chain::Bitcoin])));
    }

    #[test]
    fn test_funding_apr_annualizes_the_hourly_rate() {
        assert_eq!(funding_apr(0.0001), 0.0001 * 8760.0);
        assert_eq!(funding_apr(0.0), 0.0);
    }

    #[test]
    fn test_slippage_price_moves_against_the_trader() {
        assert_eq!(slippage_price(100.0, PerpetualDirection::Long, true, 2.0), 102.0);
        assert_eq!(slippage_price(100.0, PerpetualDirection::Short, true, 2.0), 98.0);
        assert_eq!(slippage_price(100.0, PerpetualDirection::Long, false, 2.0), 98.0);
        assert_eq!(slippage_price(100.0, PerpetualDirection::Short, false, 2.0), 102.0);
    }

    #[test]
    fn test_order_amounts_scale_the_margin_by_leverage() {
        let (size, fiat_value, margin) = order_amounts(50.0, 4, 200.0);

        assert_eq!(size, 1.0);
        assert_eq!(fiat_value, 200.0);
        assert_eq!(margin, 50.0);
    }

    #[test]
    fn test_only_reduce_closes_a_position() {
        assert!(opens_position(&GemPerpetualOrderAction::Open));
        assert!(opens_position(&GemPerpetualOrderAction::Increase));
        assert!(!opens_position(&GemPerpetualOrderAction::Reduce {
            position_direction: PerpetualDirection::Long
        }));
    }

    #[test]
    fn test_slippage_percent_defaults_to_two() {
        assert_eq!(slippage_percent(None), 2.0);
        assert_eq!(slippage_percent(Some(0.5)), 0.5);
    }

    fn order_input(action: GemPerpetualOrderAction) -> GemPerpetualOrderInput {
        GemPerpetualOrderInput {
            action,
            direction: PerpetualDirection::Long,
            margin_type: PerpetualMarginType::Cross,
            base_asset: Asset::mock(),
            asset: Asset::mock(),
            asset_index: 1,
            price: 100.0,
            usdc_amount: "50000000".to_string(),
            usdc_decimals: 6,
            leverage: 4,
            slippage: None,
            take_profit: None,
            stop_loss: None,
        }
    }

    #[test]
    fn test_perpetual_order_keeps_the_position_action_and_prices_in_the_slippage() {
        let open = order(PerpetualProvider::Hypercore, order_input(GemPerpetualOrderAction::Open)).unwrap();
        let increase = order(PerpetualProvider::Hypercore, order_input(GemPerpetualOrderAction::Increase)).unwrap();
        let reduce = order(
            PerpetualProvider::Hypercore,
            order_input(GemPerpetualOrderAction::Reduce {
                position_direction: PerpetualDirection::Short,
            }),
        )
        .unwrap();

        let PerpetualType::Open(data) = open else { panic!("expected an open order") };
        assert_eq!(data.slippage, 2.0);
        assert_eq!(data.market_price, 100.0);
        assert_eq!(data.fiat_value, 200.0);
        assert_eq!(data.margin_amount, 50.0);
        assert!(matches!(increase, PerpetualType::Increase(_)));
        let PerpetualType::Reduce(reduce) = reduce else { panic!("expected a reduce order") };
        assert_eq!(reduce.position_direction, PerpetualDirection::Short);
    }

    #[test]
    fn test_perpetual_close_order_carries_the_position_result() {
        let data = close_order(
            PerpetualProvider::Hypercore,
            GemPerpetualCloseInput {
                asset_index: 1,
                direction: PerpetualDirection::Long,
                margin_type: PerpetualMarginType::Cross,
                base_asset: Asset::mock(),
                asset: Asset::mock(),
                market_price: 100.0,
                size: -2.0,
                leverage: 4,
                pnl: 12.5,
                entry_price: 90.0,
                margin_amount: 50.0,
                slippage: None,
            },
        );

        assert_eq!(data.pnl, Some(12.5));
        assert_eq!(data.entry_price, Some(90.0));
        assert_eq!(data.fiat_value, 196.0);
    }
}

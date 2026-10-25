use std::collections::HashMap;

use primitives::{
    Asset, AssetType, Perpetual, PerpetualAccountMode, PerpetualBalance, PerpetualDirection, PerpetualId, PerpetualMarginType, PerpetualOrderType, PerpetualPosition, PerpetualProvider, PerpetualTriggerOrder,
    chart::ChartCandleStick,
    known_assets::USDC_SYMBOL,
    perpetual::{PerpetualAddress, PerpetualData, PerpetualMetadata, PerpetualPositionsSummary},
    portfolio::{PerpetualAccountSummary, PerpetualPortfolio},
};

use crate::models::{
    balance::Balances,
    candlestick::Candlestick,
    metadata::{HypercoreMetadataResponse, perpetual_asset_id},
    order::OpenOrder,
    perp_dex::PerpDex,
    portfolio::{HypercorePortfolioResponse, HypercorePortfolioTimeframeData},
    position::{AssetPositions, LeverageType, Position},
    referral::{Referral, ReferralTokenState},
};

const USDC_TOKEN_INDEX: i32 = 0;

pub(crate) fn map_perpetual_addresses(referral: Referral) -> Vec<PerpetualAddress> {
    let addresses = referral.referrer_state.and_then(|state| state.data).and_then(|data| data.referral_states).unwrap_or_default();
    addresses
        .into_iter()
        .map(|address| {
            let totals = address
                .token_to_state
                .into_iter()
                .flatten()
                .filter(|(token, _)| *token != USDC_TOKEN_INDEX)
                .fold(address.usdc, |totals, (_, state)| ReferralTokenState {
                    cum_vlm: totals.cum_vlm + state.cum_vlm,
                    cum_fees_rewarded_to_referrer: totals.cum_fees_rewarded_to_referrer + state.cum_fees_rewarded_to_referrer,
                });
            PerpetualAddress {
                provider: PerpetualProvider::Hypercore,
                address: address.user,
                referred_at: address.time_joined,
                cumulative_volume_usd: totals.cum_vlm,
                cumulative_referral_earned_usd: totals.cum_fees_rewarded_to_referrer,
            }
        })
        .collect()
}

pub fn create_perpetual_id(coin: &str) -> PerpetualId {
    PerpetualId::new(PerpetualProvider::Hypercore, coin)
}

pub fn map_positions(positions: AssetPositions, address: String, orders: &[OpenOrder]) -> PerpetualPositionsSummary {
    let balance = map_perpetual_balance(&positions);
    let positions: Vec<PerpetualPosition> = positions.asset_positions.into_iter().map(|x| map_position(x.position, address.clone(), orders)).collect();
    PerpetualPositionsSummary { positions, balance }
}

pub fn map_perpetual_balance(positions: &AssetPositions) -> PerpetualBalance {
    let equity = positions.margin_summary.account_value.parse().unwrap_or(0.0);
    let margin_used = positions.margin_summary.total_margin_used.parse().unwrap_or(0.0);
    let reserved = f64::min(f64::max(margin_used, 0.0), f64::max(equity, 0.0));
    let available = f64::max(equity - reserved, 0.0);
    let withdrawable = positions.withdrawable.parse().unwrap_or(0.0);

    PerpetualBalance { available, reserved, withdrawable }
}

pub fn map_perpetual_dexes_balance(states: &[(PerpDex, AssetPositions)]) -> PerpetualBalance {
    total_dexes_balance(states.iter().map(|(dex, positions)| (*dex, map_perpetual_balance(positions))).collect())
}

pub fn total_dexes_balance(balances: Vec<(PerpDex, PerpetualBalance)>) -> PerpetualBalance {
    let withdrawable = balances.iter().find(|(dex, _)| *dex == PerpDex::Main).map_or(0.0, |(_, balance)| balance.withdrawable);
    PerpetualBalance {
        withdrawable,
        ..balances.into_iter().map(|(_, balance)| balance).sum()
    }
}

pub fn map_perpetual_balance_from_spot(balances: &Balances) -> PerpetualBalance {
    let Some(collateral) = balances.balances.iter().find(|balance| balance.coin == USDC_SYMBOL) else {
        return PerpetualBalance {
            available: 0.0,
            reserved: 0.0,
            withdrawable: 0.0,
        };
    };
    let total = f64::max(collateral.total.parse().unwrap_or(0.0), 0.0);
    let available = balances
        .available_after_maintenance(collateral.token)
        .and_then(|value| value.parse::<f64>().ok())
        .map(|value| value.clamp(0.0, total))
        .unwrap_or(total);

    PerpetualBalance {
        available,
        reserved: total - available,
        withdrawable: available,
    }
}

pub fn map_position(position: Position, address: String, orders: &[OpenOrder]) -> PerpetualPosition {
    let size: f64 = position.szi.parse().unwrap_or(0.0);
    let direction = if size >= 0.0 { PerpetualDirection::Long } else { PerpetualDirection::Short };

    let raw_funding = position.cum_funding.since_open.parse::<f32>().unwrap_or(0.0);
    let funding_value = Some(-raw_funding);
    let perpetual_id = create_perpetual_id(&position.coin);
    let asset_id = perpetual_asset_id(&position.coin);

    let (take_profit, stop_loss) = map_tp_sl_from_orders(orders, &position.coin);

    PerpetualPosition {
        id: format!("{}_{}", address.to_lowercase(), position.coin),
        perpetual_id,
        asset_id,
        size: size.abs(),
        size_value: position.position_value.parse::<f64>().unwrap_or(0.0).abs(),
        leverage: position.leverage.value as u8,
        entry_price: position.entry_px.parse().unwrap_or(0.0),
        liquidation_price: position.liquidation_px.and_then(|p| p.parse().ok()),
        margin_type: match position.leverage.leverage_type {
            LeverageType::Cross => PerpetualMarginType::Cross,
            LeverageType::Isolated => PerpetualMarginType::Isolated,
        },
        direction,
        margin_amount: position.margin_used.parse().unwrap_or(0.0),
        take_profit,
        stop_loss,
        pnl: position.unrealized_pnl.parse().unwrap_or(0.0),
        funding: funding_value,
    }
}

pub fn map_perpetuals_data(metadata: HypercoreMetadataResponse, dex: PerpDex) -> Vec<PerpetualData> {
    let universe = metadata.universe();
    let asset_metadata = metadata.asset_metadata();

    universe
        .universe
        .iter()
        .enumerate()
        .map(|(index, universe_asset)| {
            let metadata_item = asset_metadata.get(index);

            let asset_id = universe_asset.asset_id();
            let asset_index = dex.asset_index(index as u32);

            let current_price = metadata_item.and_then(|m| m.mid_px.as_ref().and_then(|mid| mid.parse().ok()).or_else(|| m.mark_px.parse().ok())).unwrap_or(0.0);

            let prev_price = metadata_item.and_then(|m| m.prev_day_px.parse().ok()).unwrap_or(0.0);

            let price_change_24h = if prev_price > 0.0 { ((current_price - prev_price) / prev_price) * 100.0 } else { 0.0 };

            let funding_rate = metadata_item.and_then(|m| m.funding.parse::<f64>().ok()).unwrap_or(0.0) * 100.0;

            let open_interest_coins = metadata_item.and_then(|m| m.open_interest.parse::<f64>().ok()).unwrap_or(0.0);
            let open_interest_usd = open_interest_coins * current_price;
            let volume_24h = metadata_item.and_then(|m| m.day_ntl_vlm.parse().ok()).unwrap_or(0.0);

            let perpetual_id = create_perpetual_id(&universe_asset.name);
            let perpetual = Perpetual {
                id: perpetual_id,
                name: universe_asset.name.clone(),
                provider: PerpetualProvider::Hypercore,
                asset_id: asset_id.clone(),
                identifier: asset_index.to_string(),
                price: current_price,
                price_percent_change_24h: price_change_24h,
                open_interest: open_interest_usd,
                volume_24h,
                funding: funding_rate,
                max_leverage: universe_asset.max_leverage as u8,
                is_isolated_only: dex.is_isolated_only() || universe_asset.only_isolated.unwrap_or(false),
                is_enabled: !universe_asset.is_delisted.unwrap_or(false) && volume_24h >= dex.min_volume_24h(),
            };

            let asset = Asset {
                id: asset_id,
                name: universe_asset.name.clone(),
                symbol: universe_asset.name.clone(),
                decimals: universe_asset.sz_decimals as u32,
                asset_type: AssetType::PERPETUAL,
            };

            let metadata = PerpetualMetadata { is_pinned: false };

            PerpetualData { perpetual, asset, metadata }
        })
        .collect()
}

pub fn map_candlesticks(candlesticks: Vec<Candlestick>) -> Vec<ChartCandleStick> {
    candlesticks.iter().map(ChartCandleStick::from).collect()
}

const PORTFOLIO_TIMEFRAMES: [(&str, &str); 4] = [("perpDay", "day"), ("perpWeek", "week"), ("perpMonth", "month"), ("perpAllTime", "allTime")];

pub fn map_perpetual_portfolio(response: HypercorePortfolioResponse, mode: PerpetualAccountMode, positions: &[AssetPositions], balance: &PerpetualBalance) -> PerpetualPortfolio {
    let mut timeframes: HashMap<String, HypercorePortfolioTimeframeData> = response.timeframes.into_iter().collect();
    let [day, week, month, all_time] = PORTFOLIO_TIMEFRAMES.map(|(perpetual, account)| {
        let perpetual = timeframes.remove(perpetual)?;
        let account_value_history = match mode {
            PerpetualAccountMode::Standard => perpetual.account_value_history,
            PerpetualAccountMode::Unified => timeframes.remove(account)?.account_value_history,
        };
        Some(HypercorePortfolioTimeframeData { account_value_history, ..perpetual }.into())
    });

    PerpetualPortfolio {
        day,
        week,
        month,
        all_time,
        account_summary: Some(map_account_summary(positions, balance)),
    }
}

fn map_account_summary(positions: &[AssetPositions], balance: &PerpetualBalance) -> PerpetualAccountSummary {
    let account_value = balance.available + balance.reserved;
    let total_ntl_pos: f64 = positions.iter().map(|p| p.margin_summary.total_ntl_pos.parse().unwrap_or(0.0)).sum();
    let total_margin_used: f64 = positions.iter().map(|p| p.margin_summary.total_margin_used.parse().unwrap_or(0.0)).sum();
    let unrealized_pnl: f64 = positions.iter().flat_map(|p| &p.asset_positions).map(|p| p.position.unrealized_pnl.parse().unwrap_or(0.0)).sum();

    let account_leverage = if account_value > 0.0 { total_ntl_pos / account_value } else { 0.0 };
    let margin_usage = if account_value > 0.0 { total_margin_used / account_value } else { 0.0 };

    PerpetualAccountSummary {
        account_value,
        account_leverage,
        margin_usage,
        unrealized_pnl,
    }
}

fn determine_order_type(order_type_str: &str) -> PerpetualOrderType {
    if order_type_str.to_lowercase().contains("market") { PerpetualOrderType::Market } else { PerpetualOrderType::Limit }
}

pub fn map_tp_sl_from_orders(orders: &[OpenOrder], coin: &str) -> (Option<PerpetualTriggerOrder>, Option<PerpetualTriggerOrder>) {
    orders.iter().filter(|o| o.is_position_tpsl && o.coin == coin).fold((None, None), |(tp, sl), order| match order.trigger_px {
        Some(price) if order.order_type.to_lowercase().contains("take profit") => (
            Some(PerpetualTriggerOrder {
                price,
                order_type: determine_order_type(&order.order_type),
                order_id: order.oid.to_string(),
            }),
            sl,
        ),
        Some(price) if order.order_type.to_lowercase().contains("stop") => (
            tp,
            Some(PerpetualTriggerOrder {
                price,
                order_type: determine_order_type(&order.order_type),
                order_id: order.oid.to_string(),
            }),
        ),
        _ => (tp, sl),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        balance::{Balance, Balances},
        metadata::{AssetMetadata, HypercoreUniverseResponse, UniverseAsset},
        position::{AssetPosition, AssetPositions, CumulativeFunding, Leverage, LeverageType, MarginSummary, Position, PositionType},
    };
    use chrono::DateTime;
    use primitives::{PerpetualDirection, PerpetualMarginType, perpetual_provider::PerpetualProvider};

    #[test]
    fn test_map_perpetual_addresses() {
        let referral: Referral = serde_json::from_str(include_str!("../../testdata/referral_addresses.json")).unwrap();

        assert_eq!(
            map_perpetual_addresses(referral),
            vec![
                PerpetualAddress {
                    provider: PerpetualProvider::Hypercore,
                    address: "0x0000000000000000000000000000000000000001".to_string(),
                    referred_at: DateTime::from_timestamp_millis(1753127993370).unwrap(),
                    cumulative_volume_usd: 190.0,
                    cumulative_referral_earned_usd: 19.0,
                },
                PerpetualAddress {
                    provider: PerpetualProvider::Hypercore,
                    address: "0x0000000000000000000000000000000000000002".to_string(),
                    referred_at: DateTime::from_timestamp_millis(1753127994370).unwrap(),
                    cumulative_volume_usd: 0.0,
                    cumulative_referral_earned_usd: 0.0,
                },
            ]
        );
    }

    #[test]
    fn test_map_perpetual_addresses_collects_new_tokens_and_legacy_stats() {
        let mut referral: Referral = serde_json::from_str(include_str!("../../testdata/referral_addresses.json")).unwrap();
        let addresses = referral.referrer_state.as_mut().unwrap().data.as_mut().unwrap().referral_states.as_mut().unwrap();
        let state = addresses[0].token_to_state.as_ref().unwrap()[0].1.clone();
        addresses[0].token_to_state.as_mut().unwrap().push((999, state));
        addresses[1].token_to_state = None;
        addresses[1].usdc.cum_vlm = 50.0;
        addresses[1].usdc.cum_fees_rewarded_to_referrer = 5.0;

        let addresses = map_perpetual_addresses(referral);

        assert_eq!(addresses.len(), 2);
        assert_eq!(addresses[0].cumulative_volume_usd, 290.0);
        assert_eq!(addresses[0].cumulative_referral_earned_usd, 29.0);
        assert_eq!(addresses[1].cumulative_volume_usd, 50.0);
        assert_eq!(addresses[1].cumulative_referral_earned_usd, 5.0);
    }

    #[test]
    fn test_map_perpetual_addresses_without_referrals() {
        for payload in [include_str!("../../testdata/referral_need_to_trade.json"), include_str!("../../testdata/referral_need_to_create_code.json")] {
            let referral = serde_json::from_str(payload).unwrap();
            assert_eq!(map_perpetual_addresses(referral), vec![]);
        }
    }

    #[test]
    fn test_map_positions_basic() {
        let positions = AssetPositions {
            asset_positions: vec![AssetPosition {
                position_type: PositionType::OneWay,
                position: Position {
                    coin: "BTC".to_string(),
                    szi: "1.5".to_string(),
                    leverage: Leverage {
                        leverage_type: LeverageType::Cross,
                        value: 10,
                    },
                    entry_px: "50000".to_string(),
                    position_value: "75000".to_string(),
                    unrealized_pnl: "5000".to_string(),
                    return_on_equity: "0.1".to_string(),
                    liquidation_px: Some("40000".to_string()),
                    margin_used: "7500".to_string(),
                    max_leverage: 20,
                    cum_funding: CumulativeFunding {
                        all_time: "100".to_string(),
                        since_open: "50".to_string(),
                    },
                },
            }],
            margin_summary: MarginSummary {
                account_value: "100000".to_string(),
                total_ntl_pos: "10000".to_string(),
                total_raw_usd: "10000".to_string(),
                total_margin_used: "8000".to_string(),
            },
            cross_margin_summary: MarginSummary {
                account_value: "100000".to_string(),
                total_ntl_pos: "10000".to_string(),
                total_raw_usd: "10000".to_string(),
                total_margin_used: "8000".to_string(),
            },
            cross_maintenance_margin_used: "3000".to_string(),
            withdrawable: "92000".to_string(),
        };

        let result = map_positions(positions, "test_address".to_string(), &[]);

        assert_eq!(result.positions.len(), 1);
        assert_eq!(result.positions[0].id, "test_address_BTC");
        assert_eq!(result.positions[0].size, 1.5);
        assert_eq!(result.positions[0].direction, PerpetualDirection::Long);
        assert_eq!(result.positions[0].margin_type, PerpetualMarginType::Cross);
        assert_eq!(result.positions[0].leverage, 10);
        assert_eq!(result.positions[0].pnl, 5000.0);
        assert_eq!(result.positions[0].funding, Some(-50.0));

        assert_eq!(result.balance.available, 92000.0);
        assert_eq!(result.balance.reserved, 8000.0);
        assert_eq!(result.balance.withdrawable, 92000.0);
    }

    #[test]
    fn test_map_perpetuals_data() {
        let universe_response = HypercoreUniverseResponse {
            universe: vec![UniverseAsset {
                only_isolated: Some(false),
                ..UniverseAsset::mock()
            }],
        };

        let asset_metadata = vec![AssetMetadata {
            premium: Some("1.5".to_string()),
            impact_pxs: Some(vec!["2100".to_string(), "2105".to_string()]),
            ..AssetMetadata::mock()
        }];

        let metadata_response = HypercoreMetadataResponse(universe_response, asset_metadata);
        let result = map_perpetuals_data(metadata_response, PerpDex::Main);

        assert_eq!(result.len(), 1);

        let eth_data = &result[0];
        assert_eq!(eth_data.perpetual.id, PerpetualId::new(PerpetualProvider::Hypercore, "ETH"));
        assert_eq!(eth_data.perpetual.name, "ETH");
        assert_eq!(eth_data.perpetual.provider, PerpetualProvider::Hypercore);
        assert_eq!(eth_data.perpetual.price, 2102.5);
        assert_eq!(eth_data.perpetual.funding, 0.05);
        assert_eq!(eth_data.perpetual.max_leverage, 50);
        assert_eq!(eth_data.perpetual.volume_24h, 500000.0);

        assert!(!eth_data.perpetual.is_isolated_only);

        assert_eq!(eth_data.asset.name, "ETH");
        assert_eq!(eth_data.asset.symbol, "ETH");
        assert_eq!(eth_data.asset.decimals, 4);
        assert_eq!(eth_data.asset.id.to_string(), "hypercore_perpetual::ETH");
    }

    #[test]
    fn test_map_perpetuals_data_builder_asset_index() {
        let universe_response = HypercoreUniverseResponse {
            universe: vec![UniverseAsset {
                name: "FOO".to_string(),
                ..UniverseAsset::mock()
            }],
        };

        let asset_metadata = vec![AssetMetadata::mock()];

        let metadata_response = HypercoreMetadataResponse(universe_response, asset_metadata);
        let result = map_perpetuals_data(metadata_response, PerpDex::Xyz);

        assert_eq!(result[0].perpetual.identifier, "110000");
    }

    #[test]
    fn test_map_perpetuals_data_names_an_xyz_asset_with_its_dex() {
        let universe_response = HypercoreUniverseResponse {
            universe: vec![UniverseAsset {
                name: "xyz:NVDA".to_string(),
                ..UniverseAsset::mock()
            }],
        };

        let result = map_perpetuals_data(HypercoreMetadataResponse(universe_response, vec![AssetMetadata::mock()]), PerpDex::Xyz);

        assert_eq!(
            (result[0].perpetual.name.as_str(), result[0].asset.name.as_str(), result[0].asset.symbol.as_str(), result[0].asset.id.to_string()),
            ("xyz:NVDA", "xyz:NVDA", "xyz:NVDA", "hypercore_perpetual::xyz:NVDA".to_string()),
            "an xyz market and its asset carry the dex, so users can tell it from Hyperliquid's own markets"
        );
    }

    #[test]
    fn test_map_perpetuals_data_only_isolated() {
        let universe_response = HypercoreUniverseResponse {
            universe: vec![
                UniverseAsset {
                    name: "ISOLATED_TOKEN".to_string(),
                    only_isolated: Some(true),
                    ..UniverseAsset::mock()
                },
                UniverseAsset {
                    name: "DEFAULT_TOKEN".to_string(),
                    only_isolated: None,
                    ..UniverseAsset::mock()
                },
            ],
        };

        let asset_metadata = vec![AssetMetadata::mock(), AssetMetadata::mock()];

        let metadata_response = HypercoreMetadataResponse(universe_response, asset_metadata);
        let isolated_only = |dex| map_perpetuals_data(metadata_response.clone(), dex).iter().map(|data| data.perpetual.is_isolated_only).collect::<Vec<_>>();

        assert_eq!(isolated_only(PerpDex::Main), vec![true, false]);
        assert_eq!(isolated_only(PerpDex::Xyz), vec![true, true], "every xyz market trades isolated, including the ones Hyperliquid also allows cross on");
    }

    #[test]
    fn test_map_perpetuals_data_disables_delisted() {
        let metadata: HypercoreMetadataResponse = serde_json::from_str(include_str!("../../testdata/perpetual_markets_response_meta_and_asset_ctxs.json")).unwrap();

        let result = map_perpetuals_data(metadata, PerpDex::Main);

        assert_eq!(
            result.iter().map(|data| (data.perpetual.name.as_str(), data.perpetual.identifier.as_str(), data.perpetual.is_enabled)).collect::<Vec<_>>(),
            vec![("BTC", "0", true), ("MATIC", "1", false), ("ETH", "2", true)]
        );
    }

    #[test]
    fn test_map_perpetuals_data_disables_a_thin_xyz_market() {
        let universe_response = HypercoreUniverseResponse {
            universe: ["xyz:CL", "xyz:CAMBRICON"]
                .map(|name| UniverseAsset {
                    name: name.to_string(),
                    ..UniverseAsset::mock()
                })
                .to_vec(),
        };
        let asset_metadata = vec![
            AssetMetadata {
                day_ntl_vlm: "245000000".to_string(),
                ..AssetMetadata::mock()
            },
            AssetMetadata {
                day_ntl_vlm: "2000".to_string(),
                ..AssetMetadata::mock()
            },
        ];
        let metadata_response = HypercoreMetadataResponse(universe_response, asset_metadata);
        let enabled = |dex| map_perpetuals_data(metadata_response.clone(), dex).iter().map(|data| data.perpetual.is_enabled).collect::<Vec<_>>();

        assert_eq!(enabled(PerpDex::Xyz), vec![true, false], "an xyz market trading under a million a day is kept but not offered");
        assert_eq!(enabled(PerpDex::Main), vec![true, true]);
    }

    #[test]
    fn test_map_candlesticks() {
        use crate::models::candlestick::Candlestick;

        let candlesticks = vec![
            Candlestick {
                t: 1640995200000u64,
                s: "BTC".to_string(),
                i: "1h".to_string(),
                o: "50000.0".to_string(),
                h: "51000.0".to_string(),
                l: "49000.0".to_string(),
                c: "50500.0".to_string(),
                v: "100.5".to_string(),
            },
            Candlestick {
                t: 1640998800000u64,
                s: "BTC".to_string(),
                i: "1h".to_string(),
                o: "50500.0".to_string(),
                h: "52000.0".to_string(),
                l: "50000.0".to_string(),
                c: "51000.0".to_string(),
                v: "75.2".to_string(),
            },
        ];

        let result = map_candlesticks(candlesticks);

        assert_eq!(result.len(), 2);

        let first_candle = &result[0];
        assert_eq!(first_candle.open, 50000.0);
        assert_eq!(first_candle.high, 51000.0);
        assert_eq!(first_candle.low, 49000.0);
        assert_eq!(first_candle.close, 50500.0);
        assert_eq!(first_candle.volume, 100.5);

        let second_candle = &result[1];
        assert_eq!(second_candle.open, 50500.0);
        assert_eq!(second_candle.high, 52000.0);
        assert_eq!(second_candle.low, 50000.0);
        assert_eq!(second_candle.close, 51000.0);
        assert_eq!(second_candle.volume, 75.2);
    }

    #[test]
    fn test_map_hypercore_positions_to_perpetual_positions_summary() {
        let positions = AssetPositions {
            asset_positions: vec![
                AssetPosition {
                    position_type: PositionType::OneWay,
                    position: Position {
                        coin: "SOL".to_string(),
                        szi: "-10.0".to_string(),
                        leverage: Leverage {
                            leverage_type: LeverageType::Cross,
                            value: 20,
                        },
                        entry_px: "195.39".to_string(),
                        position_value: "2029.2".to_string(),
                        unrealized_pnl: "-75.3".to_string(),
                        return_on_equity: "-0.77076616".to_string(),
                        liquidation_px: Some("558.9517436098".to_string()),
                        margin_used: "101.46".to_string(),
                        max_leverage: 20,
                        cum_funding: CumulativeFunding {
                            all_time: "-1.3358".to_string(),
                            since_open: "-1.3".to_string(),
                        },
                    },
                },
                AssetPosition {
                    position_type: PositionType::OneWay,
                    position: Position {
                        coin: "BTC".to_string(),
                        szi: "3.0".to_string(),
                        leverage: Leverage {
                            leverage_type: LeverageType::Isolated,
                            value: 10,
                        },
                        entry_px: "766.34".to_string(),
                        position_value: "2332.2".to_string(),
                        unrealized_pnl: "33.18".to_string(),
                        return_on_equity: "0.1443223634".to_string(),
                        liquidation_px: None,
                        margin_used: "233.22".to_string(),
                        max_leverage: 10,
                        cum_funding: CumulativeFunding {
                            all_time: "1.686397".to_string(),
                            since_open: "1.1".to_string(),
                        },
                    },
                },
            ],
            margin_summary: MarginSummary {
                account_value: "1000".to_string(),
                total_ntl_pos: "100".to_string(),
                total_raw_usd: "100".to_string(),
                total_margin_used: "100".to_string(),
            },
            cross_margin_summary: MarginSummary {
                account_value: "1000".to_string(),
                total_ntl_pos: "100".to_string(),
                total_raw_usd: "100".to_string(),
                total_margin_used: "100".to_string(),
            },
            cross_maintenance_margin_used: "50".to_string(),
            withdrawable: "500".to_string(),
        };

        let summary = map_positions(positions, "test_user".to_string(), &[]);

        assert_eq!(summary.positions.len(), 2);

        let sol_position = summary.positions.iter().find(|p| p.id == "test_user_SOL").unwrap();
        assert_eq!(sol_position.size, 10.0);
        assert_eq!(sol_position.size_value, 2029.2);
        assert_eq!(sol_position.leverage, 20);
        assert_eq!(sol_position.margin_type, PerpetualMarginType::Cross);
        assert_eq!(sol_position.direction, PerpetualDirection::Short);
        assert_eq!(sol_position.margin_amount, 101.46);
        assert_eq!(sol_position.pnl, -75.3);
        assert_eq!(sol_position.funding, Some(1.3));

        let btc_position = summary.positions.iter().find(|p| p.id == "test_user_BTC").unwrap();
        assert_eq!(btc_position.size, 3.0);
        assert_eq!(btc_position.size_value, 2332.2);
        assert_eq!(btc_position.leverage, 10);
        assert_eq!(btc_position.margin_type, PerpetualMarginType::Isolated);
        assert_eq!(btc_position.direction, PerpetualDirection::Long);
        assert_eq!(btc_position.margin_amount, 233.22);
        assert_eq!(btc_position.pnl, 33.18);
        assert_eq!(btc_position.funding, Some(-1.1));
    }

    #[test]
    fn test_map_position_funding_sign_reversal() {
        let position = Position {
            coin: "BTC".to_string(),
            szi: "3.0".to_string(),
            leverage: Leverage {
                leverage_type: LeverageType::Cross,
                value: 10,
            },
            entry_px: "100".to_string(),
            position_value: "300".to_string(),
            unrealized_pnl: "0".to_string(),
            return_on_equity: "0".to_string(),
            liquidation_px: None,
            margin_used: "30".to_string(),
            max_leverage: 10,
            cum_funding: CumulativeFunding {
                all_time: "1.5".to_string(),
                since_open: "1.5".to_string(),
            },
        };
        let received_long_position = Position {
            cum_funding: CumulativeFunding {
                all_time: "-1.5".to_string(),
                since_open: "-1.5".to_string(),
            },
            ..position.clone()
        };

        let perpetual_position = map_position(position, "user123".to_string(), &[]);
        assert_eq!(perpetual_position.funding, Some(-1.5));
        assert_eq!(map_position(received_long_position, "user123".to_string(), &[]).funding, Some(1.5));

        let short_position = Position {
            coin: "ETH".to_string(),
            szi: "-5.0".to_string(),
            leverage: Leverage {
                leverage_type: LeverageType::Cross,
                value: 10,
            },
            entry_px: "100".to_string(),
            position_value: "500".to_string(),
            unrealized_pnl: "0".to_string(),
            return_on_equity: "0".to_string(),
            liquidation_px: None,
            margin_used: "50".to_string(),
            max_leverage: 10,
            cum_funding: CumulativeFunding {
                all_time: "-1.5".to_string(),
                since_open: "-1.5".to_string(),
            },
        };

        let paid_short_position = Position {
            cum_funding: CumulativeFunding {
                all_time: "1.5".to_string(),
                since_open: "1.5".to_string(),
            },
            ..short_position.clone()
        };

        let short_perpetual = map_position(short_position, "user123".to_string(), &[]);
        assert_eq!(short_perpetual.size, 5.0);
        assert_eq!(short_perpetual.funding, Some(1.5));
        assert_eq!(map_position(paid_short_position, "user123".to_string(), &[]).funding, Some(-1.5), "funding a short paid is a cost, the same as for a long");
    }

    #[test]
    fn test_map_perpetual_balance() {
        let positions: AssetPositions = serde_json::from_str(include_str!("../../testdata/perpetual_positions_response_clearinghouse_state_isolated.json")).unwrap();

        assert_eq!(
            map_perpetual_balance(&positions),
            PerpetualBalance {
                available: 8.286923 - 8.286603,
                reserved: 8.286603,
                withdrawable: 0.00032,
            },
            "the margin of the isolated HYPE position is in use alongside the cross BTC margin"
        );
    }

    #[test]
    fn test_map_perpetual_dexes_balance() {
        let xyz = AssetPositions {
            withdrawable: "500".to_string(),
            ..AssetPositions::mock()
        };
        let balance = map_perpetual_dexes_balance(&[(PerpDex::Main, AssetPositions::mock()), (PerpDex::Xyz, xyz)]);

        assert_eq!(balance.available, 16000.0);
        assert_eq!(balance.withdrawable, 8000.0, "Hyperliquid pays a withdrawal from the main dex only");
    }

    #[test]
    fn test_map_perpetual_balance_with_real_data() {
        let positions = AssetPositions {
            asset_positions: vec![],
            margin_summary: MarginSummary {
                account_value: "706.364534".to_string(),
                total_ntl_pos: "12013.47849".to_string(),
                total_raw_usd: "2737.835324".to_string(),
                total_margin_used: "926.155026".to_string(),
            },
            cross_margin_summary: MarginSummary {
                account_value: "706.364534".to_string(),
                total_ntl_pos: "12013.47849".to_string(),
                total_raw_usd: "2737.835324".to_string(),
                total_margin_used: "926.155026".to_string(),
            },
            cross_maintenance_margin_used: "400.689965".to_string(),
            withdrawable: "305.674569".to_string(),
        };

        let summary = map_positions(positions, "real_data_test".to_string(), &[]);

        assert_eq!(summary.balance.reserved, 706.364534);
        assert_eq!(summary.balance.available, 0.0);
        assert_eq!(summary.balance.withdrawable, 305.674569);
    }

    #[test]
    fn test_map_position_asset_id_uses_subtoken_pattern() {
        let position = Position {
            coin: "BTC".to_string(),
            szi: "1.0".to_string(),
            leverage: Leverage {
                leverage_type: LeverageType::Cross,
                value: 10,
            },
            entry_px: "50000".to_string(),
            position_value: "50000".to_string(),
            unrealized_pnl: "0".to_string(),
            return_on_equity: "0".to_string(),
            liquidation_px: None,
            margin_used: "5000".to_string(),
            max_leverage: 10,
            cum_funding: CumulativeFunding {
                all_time: "0".to_string(),
                since_open: "0".to_string(),
            },
        };

        let perpetual_position = map_position(position, "address123".to_string(), &[]);

        assert_eq!(perpetual_position.asset_id.chain, primitives::Chain::HyperCore);
        assert_eq!(perpetual_position.asset_id.token_id, Some("perpetual::BTC".to_string()));
        assert_eq!(perpetual_position.asset_id.to_string(), "hypercore_perpetual::BTC");
    }

    #[test]
    fn test_map_tp_sl_from_orders_limit() {
        use crate::testkit::*;

        let orders = vec![
            OpenOrder::mock("HYPE", 191395165138, "Stop Limit", 35.0, Some(33.5)),
            OpenOrder::mock("HYPE", 191394991415, "Take Profit Limit", 55.0, Some(56.0)),
        ];

        let (take_profit, stop_loss) = map_tp_sl_from_orders(&orders, "HYPE");

        let tp = take_profit.unwrap();
        assert_eq!(tp.price, 55.0);
        assert_eq!(tp.order_type, PerpetualOrderType::Limit);
        assert_eq!(tp.order_id, "191394991415");

        let sl = stop_loss.unwrap();
        assert_eq!(sl.price, 35.0);
        assert_eq!(sl.order_type, PerpetualOrderType::Limit);
        assert_eq!(sl.order_id, "191395165138");
    }

    #[test]
    fn test_map_tp_sl_from_orders_market() {
        use crate::testkit::*;

        let orders = vec![OpenOrder::mock("BTC", 123456789, "Stop Market", 40000.0, None), OpenOrder::mock("BTC", 987654321, "Take Profit Market", 60000.0, None)];

        let (take_profit, stop_loss) = map_tp_sl_from_orders(&orders, "BTC");

        let tp = take_profit.unwrap();
        assert_eq!(tp.price, 60000.0);
        assert_eq!(tp.order_type, PerpetualOrderType::Market);
        assert_eq!(tp.order_id, "987654321");

        let sl = stop_loss.unwrap();
        assert_eq!(sl.price, 40000.0);
        assert_eq!(sl.order_type, PerpetualOrderType::Market);
        assert_eq!(sl.order_id, "123456789");
    }

    #[test]
    fn test_map_perpetual_portfolio() {
        use crate::testkit::*;

        let account_day = HypercorePortfolioTimeframeData {
            account_value_history: vec![HypercoreDataPoint { timestamp_ms: 1640995200000, value: 2000.0 }],
            ..HypercorePortfolioTimeframeData::mock("1")
        };
        let response = HypercorePortfolioResponse {
            timeframes: [("perpDay", "100"), ("perpWeek", "500"), ("perpMonth", "2000"), ("perpAllTime", "50000")]
                .into_iter()
                .map(|(timeframe, vlm)| (timeframe.to_string(), HypercorePortfolioTimeframeData::mock(vlm)))
                .chain([("day".to_string(), account_day)])
                .collect(),
        };
        let positions = AssetPositions::mock();
        let balance = map_perpetual_balance(&positions);

        let result = map_perpetual_portfolio(response.clone(), PerpetualAccountMode::Standard, std::slice::from_ref(&positions), &balance);

        let day = result.day.unwrap();
        assert_eq!(day.volume, 100.0);
        assert_eq!(day.account_value_history[0].value, 1000.0);
        assert_eq!(result.week.unwrap().volume, 500.0);
        assert_eq!(result.month.unwrap().volume, 2000.0);
        assert_eq!(result.all_time.unwrap().volume, 50000.0);

        let summary = result.account_summary.unwrap();
        assert_eq!(summary.account_value, 10000.0);
        assert_eq!(summary.account_leverage, 0.5);
        assert_eq!(summary.margin_usage, 0.2);
        assert_eq!(summary.unrealized_pnl, 0.0);

        let unified = map_perpetual_portfolio(response, PerpetualAccountMode::Unified, &[positions], &balance);

        let day = unified.day.unwrap();
        assert_eq!(day.account_value_history[0].value, 2000.0, "a unified account's value covers the spot USDC that backs its perpetuals");
        assert_eq!(day.volume, 100.0, "volume and PnL stay perpetual, without spot trades");
    }

    #[test]
    fn test_map_account_summary() {
        use crate::testkit::*;

        let positions = [AssetPositions::mock(), AssetPositions::mock()];
        let balance = positions.iter().map(map_perpetual_balance).sum();

        let summary = map_account_summary(&positions, &balance);

        assert_eq!(summary.account_value, 20000.0, "the main dex and a HIP-3 dex add up");
        assert_eq!(summary.account_leverage, 0.5);
        assert_eq!(summary.margin_usage, 0.2);
        assert_eq!(summary.unrealized_pnl, 0.0);
    }

    #[test]
    fn test_map_perpetual_balance_from_spot() {
        let collateral = Balance {
            coin: "USDC".to_string(),
            token: 0,
            total: "4.331289".to_string(),
            hold: "4.331289".to_string(),
        };

        let balances = Balances {
            balances: vec![collateral.clone()],
            token_to_available_after_maintenance: vec![(0, "3.797316".to_string())],
        };
        let balance = map_perpetual_balance_from_spot(&balances);

        assert_eq!(balance.available, 3.797316);
        assert_eq!(balance.reserved, 4.331289 - 3.797316);
        assert_eq!(balance.withdrawable, 3.797316);

        let balances = Balances {
            balances: vec![collateral],
            token_to_available_after_maintenance: vec![],
        };
        let balance = map_perpetual_balance_from_spot(&balances);

        assert_eq!(balance.available, 4.331289);
        assert_eq!(balance.reserved, 0.0);
    }
}

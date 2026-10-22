use std::collections::{HashMap, HashSet};

use primitives::{PerpetualMarketData, PerpetualPosition};

use crate::models::{
    metadata::perpetual_coin,
    order::OpenOrder,
    perp_dex::PerpDex,
    websocket::{ActiveAssetCtxData, HyperliquidSocketMessage, HyperliquidSubscription, PositionsDiff, RawSocketMessage},
};

use super::perpetual_mapper::{map_perpetual_balance_from_spot, map_positions, map_tp_sl_from_orders};

pub fn account_subscriptions(address: String, dexes: &[PerpDex]) -> Vec<HyperliquidSubscription> {
    dexes
        .iter()
        .flat_map(|dex| {
            [
                HyperliquidSubscription::AccountState { address: address.clone(), dex: dex.name() },
                HyperliquidSubscription::OpenOrders { address: address.clone(), dex: dex.name() },
            ]
        })
        .chain([HyperliquidSubscription::SpotState { address: address.clone() }])
        .collect()
}

pub fn parse_websocket_data(data: &[u8]) -> Result<HyperliquidSocketMessage, serde_json::Error> {
    let raw: RawSocketMessage = serde_json::from_slice(data)?;

    match raw {
        RawSocketMessage::AccountState(data) => {
            let summary = map_positions(data.clearinghouse_state, data.address, &[]);
            Ok(HyperliquidSocketMessage::AccountState {
                dex: data.dex,
                balance: summary.balance,
                positions: summary.positions,
            })
        }
        RawSocketMessage::SpotState(data) => Ok(HyperliquidSocketMessage::SpotState {
            balance: map_perpetual_balance_from_spot(&data.spot_state),
        }),
        RawSocketMessage::OpenOrders(data) => Ok(HyperliquidSocketMessage::OpenOrders { dex: data.dex, orders: data.orders }),
        RawSocketMessage::Candle(candlestick) => Ok(HyperliquidSocketMessage::Candle { candle: candlestick.into() }),
        RawSocketMessage::MarketData(data) => Ok(HyperliquidSocketMessage::MarketData { market: map_active_asset_ctx(data) }),
        RawSocketMessage::MarketPrices(data) => Ok(HyperliquidSocketMessage::MarketPrices {
            prices: data
                .mids
                .into_iter()
                .filter(|(coin, _)| is_perpetual_coin(coin))
                .filter_map(|(coin, price)| price.parse::<f64>().ok().map(|price| (coin, price)))
                .collect(),
        }),
        RawSocketMessage::SubscriptionResponse(data) => Ok(HyperliquidSocketMessage::SubscriptionResponse {
            subscription_type: data.subscription.subscription_type,
        }),
        RawSocketMessage::Error(message) => Ok(HyperliquidSocketMessage::Error { message }),
        RawSocketMessage::Unknown => Ok(HyperliquidSocketMessage::Unknown),
    }
}

fn is_perpetual_coin(coin: &str) -> bool {
    coin.chars().all(|char| char.is_ascii_alphanumeric() || char == ':')
}

fn map_active_asset_ctx(data: ActiveAssetCtxData) -> PerpetualMarketData {
    let ActiveAssetCtxData { symbol, ctx: asset_context } = data;
    let mark_price = asset_context.mark_px;
    let price = asset_context.mid_px.unwrap_or(mark_price);
    let prev_price = asset_context.prev_day_px;
    let price_percent_change_24h = if prev_price > 0.0 { ((price - prev_price) / prev_price) * 100.0 } else { 0.0 };
    let open_interest = asset_context.open_interest * price;
    let volume_24h = asset_context.day_ntl_vlm;
    let funding = asset_context.funding * 100.0;

    PerpetualMarketData {
        coin: symbol,
        price,
        price_percent_change_24h,
        open_interest,
        volume_24h,
        funding,
    }
}

pub fn diff_clearinghouse_positions(dex: PerpDex, new_positions: Vec<PerpetualPosition>, existing_positions: Vec<PerpetualPosition>) -> PositionsDiff {
    let existing_positions: Vec<PerpetualPosition> = existing_positions.into_iter().filter(|position| is_on(dex, position)).collect();
    let existing_map: HashMap<&str, &PerpetualPosition> = existing_positions.iter().map(|p| (p.id.as_str(), p)).collect();

    let positions: Vec<PerpetualPosition> = new_positions
        .into_iter()
        .map(|pos| match existing_map.get(pos.id.as_str()) {
            Some(existing) => PerpetualPosition {
                take_profit: existing.take_profit.clone(),
                stop_loss: existing.stop_loss.clone(),
                ..pos
            },
            None => pos,
        })
        .collect();

    let new_ids: HashSet<&str> = positions.iter().map(|p| p.id.as_str()).collect();
    let delete_position_ids: Vec<String> = existing_positions.iter().filter(|p| !new_ids.contains(p.id.as_str())).map(|p| p.id.clone()).collect();

    PositionsDiff { delete_position_ids, positions }
}

pub fn diff_open_orders_positions(dex: PerpDex, orders: &[OpenOrder], existing_positions: Vec<PerpetualPosition>) -> PositionsDiff {
    let positions: Vec<PerpetualPosition> = existing_positions
        .into_iter()
        .filter(|position| is_on(dex, position))
        .filter_map(|pos| {
            let coin = perpetual_coin(&pos.asset_id)?;
            let (take_profit, stop_loss) = map_tp_sl_from_orders(orders, &coin);

            if pos.take_profit != take_profit || pos.stop_loss != stop_loss {
                Some(PerpetualPosition { take_profit, stop_loss, ..pos })
            } else {
                None
            }
        })
        .collect();

    PositionsDiff { delete_position_ids: vec![], positions }
}

fn is_on(dex: PerpDex, position: &PerpetualPosition) -> bool {
    perpetual_coin(&position.asset_id).is_some_and(|coin| dex.holds(&coin))
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Chain, PerpetualDirection, PerpetualMarginType, PerpetualTriggerOrder};

    #[test]
    fn test_parse_all_mids() {
        let prices = |json: &[u8]| match parse_websocket_data(json).unwrap() {
            HyperliquidSocketMessage::MarketPrices { prices } => prices,
            message => panic!("expected MarketPrices, got {message:?}"),
        };

        assert_eq!(
            prices(include_bytes!("../../testdata/ws_all_mids.json")),
            HashMap::from([("BTC".to_string(), 84779.5), ("ETH".to_string(), 2499.55), ("kPEPE".to_string(), 0.004675)]),
            "spot (@107, PURR/USDC) and outcome (#141870) mids are not perpetuals"
        );
        assert_eq!(prices(br#"{"channel":"allMids","data":{"dex":"xyz","mids":{"xyz:NVDA":"222.165"}}}"#), HashMap::from([("xyz:NVDA".to_string(), 222.165)]));
    }

    #[test]
    fn test_parse_active_asset_ctx() {
        let json = include_bytes!("../../testdata/ws_active_asset_ctx.json");
        let HyperliquidSocketMessage::MarketData { market } = parse_websocket_data(json).unwrap() else {
            panic!("expected MarketData");
        };

        assert_eq!(
            market,
            PerpetualMarketData {
                coin: "ETH".to_string(),
                price: 2236.45,
                price_percent_change_24h: 11.822499999999991,
                open_interest: 1_118_225.0,
                volume_24h: 1_169_046.294_06,
                funding: 0.00125,
            }
        );
    }

    #[test]
    fn test_parse_active_asset_ctx_rejects_invalid_numbers() {
        let json = br#"{
          "channel": "activeAssetCtx",
          "data": {
            "coin": "ETH",
            "ctx": {
              "dayNtlVlm": "1169046.29406",
              "prevDayPx": "invalid",
              "markPx": "2236.40",
              "midPx": "2236.45",
              "funding": "0.0000125",
              "openInterest": "500"
            }
          }
        }"#;

        assert!(parse_websocket_data(json).is_err());
    }

    #[test]
    fn test_parse_candle() {
        let json = include_bytes!("../../testdata/ws_candle.json");
        let HyperliquidSocketMessage::Candle { candle: update } = parse_websocket_data(json).unwrap() else {
            panic!("expected Candle");
        };

        assert_eq!(update.coin, "ETH");
        assert_eq!(update.interval, "1h");
        assert_eq!(update.candle.open, 3300.5);
        assert_eq!(update.candle.close, 3321.1);
        assert_eq!(update.candle.high, 3345.0);
        assert_eq!(update.candle.low, 3290.2);
        assert_eq!(update.candle.volume, 12450.8);
    }

    #[test]
    fn test_parse_open_orders() {
        let json = include_bytes!("../../testdata/ws_open_orders.json");
        let HyperliquidSocketMessage::OpenOrders { dex, orders } = parse_websocket_data(json).unwrap() else {
            panic!("expected OpenOrders");
        };

        assert_eq!(dex, PerpDex::Main);
        assert_eq!(orders.len(), 2);
        assert_eq!(orders[0].coin, "BTC");
        assert_eq!(orders[0].oid, 8804521338);
        assert_eq!(orders[0].trigger_px, Some(110000.0));
        assert_eq!(orders[0].limit_px, Some(110000.0));
        assert!(orders[0].is_position_tpsl);
        assert_eq!(orders[0].order_type, "Take Profit Market");
        assert_eq!(orders[1].coin, "BTC");
        assert_eq!(orders[1].oid, 8804521339);
        assert_eq!(orders[1].trigger_px, Some(95000.0));
        assert_eq!(orders[1].limit_px, Some(95000.0));
        assert!(orders[1].is_position_tpsl);
        assert_eq!(orders[1].order_type, "Stop Market");
    }

    #[test]
    fn test_parse_clearinghouse_state() {
        let json = include_bytes!("../../testdata/ws_clearinghouse_state.json");
        let HyperliquidSocketMessage::AccountState { dex, balance, positions } = parse_websocket_data(json).unwrap() else {
            panic!("expected AccountState");
        };

        assert_eq!(dex, PerpDex::Main);
        assert_eq!(balance.available, 15230.5 - 830.5);
        assert_eq!(balance.reserved, 830.5);
        assert_eq!(balance.withdrawable, 14400.0);

        assert_eq!(positions.len(), 1);
        let pos = &positions[0];
        assert_eq!(pos.id, "0xc64cc00b46150e2681a6c0e57b4b12fd2b68fbc4_ETH");
        assert_eq!(pos.asset_id.chain, Chain::HyperCore);
        assert_eq!(pos.size, 2.5);
        assert_eq!(pos.size_value, 8305.0);
        assert_eq!(pos.leverage, 10);
        assert_eq!(pos.entry_price, 3200.0);
        assert_eq!(pos.liquidation_price, Some(2850.5));
        assert_eq!(pos.margin_type, PerpetualMarginType::Cross);
        assert_eq!(pos.direction, PerpetualDirection::Long);
        assert_eq!(pos.margin_amount, 830.5);
        assert_eq!(pos.pnl, 305.0);
        assert_eq!(pos.funding, Some(-1.82));
        assert_eq!(pos.take_profit, None);
        assert_eq!(pos.stop_loss, None);
    }

    #[test]
    fn test_parse_spot_state() {
        let json = include_bytes!("../../testdata/ws_spot_state.json");
        let HyperliquidSocketMessage::SpotState { balance } = parse_websocket_data(json).unwrap() else {
            panic!("expected SpotState");
        };

        assert_eq!(balance.available, 3.797316);
        assert_eq!(balance.reserved, 4.331289 - 3.797316);
        assert_eq!(balance.withdrawable, 3.797316);
    }

    #[test]
    fn test_account_subscriptions() {
        let address = "0x123".to_string();
        let dex_state = |dex: PerpDex| {
            [
                HyperliquidSubscription::AccountState { address: address.clone(), dex: dex.name() },
                HyperliquidSubscription::OpenOrders { address: address.clone(), dex: dex.name() },
            ]
        };

        let spot_state = HyperliquidSubscription::SpotState { address: address.clone() };

        assert_eq!(account_subscriptions(address.clone(), &[PerpDex::Main]), [dex_state(PerpDex::Main).to_vec(), vec![spot_state.clone()]].concat());
        assert_eq!(
            account_subscriptions(address.clone(), &[PerpDex::Main, PerpDex::Xyz]),
            [dex_state(PerpDex::Main).to_vec(), dex_state(PerpDex::Xyz).to_vec(), vec![spot_state]].concat(),
            "every account follows its spot balance, so a switch to unified needs no new subscription"
        );
        assert_eq!(
            serde_json::to_value(dex_state(PerpDex::Xyz)).unwrap(),
            serde_json::json!([{"type": "clearinghouseState", "user": "0x123", "dex": "xyz"}, {"type": "openOrders", "user": "0x123", "dex": "xyz"}])
        );
        assert_eq!(
            serde_json::to_value(dex_state(PerpDex::Main)).unwrap(),
            serde_json::json!([{"type": "clearinghouseState", "user": "0x123"}, {"type": "openOrders", "user": "0x123"}]),
            "the main dex subscribes as it always has"
        );
    }

    #[test]
    fn test_diff_clearinghouse_positions() {
        let btc = PerpetualPosition::mock_with_coin("BTC");
        let nvda = PerpetualPosition::mock_with_coin("xyz:NVDA");

        let diff = diff_clearinghouse_positions(PerpDex::Main, vec![], vec![btc.clone(), nvda.clone()]);

        assert_eq!(diff.delete_position_ids, vec![btc.id], "a main dex snapshot leaves the xyz positions alone");
        assert!(diff.positions.is_empty());
    }

    #[test]
    fn test_diff_open_orders_positions() {
        let nvda = PerpetualPosition {
            stop_loss: Some(PerpetualTriggerOrder::mock(90.0)),
            ..PerpetualPosition::mock_with_coin("xyz:NVDA")
        };

        let diff = diff_open_orders_positions(PerpDex::Main, &[], vec![nvda]);

        assert!(diff.positions.is_empty(), "main dex orders do not clear an xyz stop loss");
    }

    #[test]
    fn test_parse_error_channel() {
        let json = br#"{"channel":"error","data":"Already subscribed: {\"type\":\"candle\",\"interval\":\"30m\",\"coin\":\"UNI\"}"}"#;

        match parse_websocket_data(json).unwrap() {
            HyperliquidSocketMessage::Error { message } => {
                assert_eq!(message, r#"Already subscribed: {"type":"candle","interval":"30m","coin":"UNI"}"#);
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }
}

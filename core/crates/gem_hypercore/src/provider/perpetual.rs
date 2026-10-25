use std::{error::Error, future::Future};

use async_trait::async_trait;
use chain_traits::{ChainAddressStatus, ChainPerpetual};
use futures::{
    future::{join_all, try_join_all},
    try_join,
};
use gem_client::Client;
use primitives::{
    ChartPeriod, PerpetualPosition,
    chart::ChartCandleStick,
    perpetual::{PerpetualAccountMode, PerpetualAccountPositions, PerpetualAddress, PerpetualBalance, PerpetualData, PerpetualPositionsSummary},
    portfolio::PerpetualPortfolio,
};

use crate::{
    models::{perp_dex::PerpDex, position::AssetPositions},
    provider::perpetual_mapper::{map_candlesticks, map_perpetual_addresses, map_perpetual_balance_from_spot, map_perpetual_dexes_balance, map_perpetual_portfolio, map_perpetuals_data, map_positions, total_dexes_balance},
    rpc::client::HyperCoreClient,
};

pub fn candle_interval(period: &ChartPeriod) -> &'static str {
    match period {
        ChartPeriod::Hour => "1m",
        ChartPeriod::Day => "30m",
        ChartPeriod::Week => "4h",
        ChartPeriod::Month => "12h",
        ChartPeriod::Year => "1w",
        ChartPeriod::All => "1M",
    }
}

impl<C: Client> HyperCoreClient<C> {
    async fn get_clearinghouse_states(&self, address: &str) -> Result<Vec<(PerpDex, AssetPositions)>, Box<dyn Error + Sync + Send>> {
        let dexes = &self.config.perp_dexes;
        let states = try_join_all(dexes.iter().map(|dex| self.get_clearinghouse_state(address, *dex))).await?;
        Ok(dexes.iter().copied().zip(states).collect())
    }

    async fn get_dex_positions(&self, address: &str, dex: PerpDex) -> Result<PerpetualPositionsSummary, Box<dyn Error + Sync + Send>> {
        let positions = self.get_clearinghouse_state(address, dex).await?;
        let orders = match positions.asset_positions.is_empty() {
            true => Vec::new(),
            false => self.get_open_orders(address, dex).await?,
        };
        Ok(map_positions(positions, address.to_string(), &orders))
    }

    async fn answered_dexes<T, F: Future<Output = Result<T, Box<dyn Error + Sync + Send>>>>(&self, fetch: impl Fn(PerpDex) -> F) -> Result<Vec<(PerpDex, T)>, Box<dyn Error + Sync + Send>> {
        let dexes = &self.config.perp_dexes;
        let mut answered = Vec::new();
        for (dex, result) in dexes.iter().copied().zip(join_all(dexes.iter().map(|dex| fetch(*dex))).await) {
            match result {
                Ok(value) => answered.push((dex, value)),
                Err(error) if dex == PerpDex::Main => return Err(error),
                Err(_) => {}
            }
        }
        Ok(answered)
    }

    async fn get_spot_collateral(&self, address: &str, mode: PerpetualAccountMode) -> Result<Option<PerpetualBalance>, Box<dyn Error + Sync + Send>> {
        match mode {
            PerpetualAccountMode::Unified => Ok(Some(map_perpetual_balance_from_spot(&self.get_spot_balances(address).await?))),
            PerpetualAccountMode::Standard => Ok(None),
        }
    }
}

#[async_trait]
impl<C: Client> ChainPerpetual for HyperCoreClient<C> {
    async fn get_positions(&self, address: String) -> Result<PerpetualAccountPositions, Box<dyn Error + Sync + Send>> {
        let (mode, dexes) = try_join!(self.get_user_abstraction(&address), self.answered_dexes(|dex| self.get_dex_positions(&address, dex)))?;
        let mode = PerpetualAccountMode::from(mode);
        let spot_collateral = self.get_spot_collateral(&address, mode).await?;
        let is_complete = dexes.len() == self.config.perp_dexes.len();
        let (balances, positions): (Vec<_>, Vec<_>) = dexes.into_iter().map(|(dex, summary)| ((dex, summary.balance), summary.positions)).unzip();

        Ok(PerpetualAccountPositions {
            mode,
            summary: PerpetualPositionsSummary {
                positions: positions.into_iter().flatten().collect(),
                balance: spot_collateral.unwrap_or_else(|| total_dexes_balance(balances)),
            },
            is_complete,
        })
    }

    async fn get_positions_for_classification(&self, address: String) -> Result<Vec<PerpetualPosition>, Box<dyn Error + Sync + Send>> {
        let summaries = try_join_all(self.config.perp_dexes.iter().map(|dex| self.get_dex_positions(&address, *dex))).await?;
        Ok(summaries.into_iter().flat_map(|summary| summary.positions).collect())
    }

    async fn get_perpetual_account_mode(&self, address: String) -> Result<PerpetualAccountMode, Box<dyn Error + Sync + Send>> {
        Ok(self.get_user_abstraction(&address).await?.into())
    }

    async fn get_perpetuals_data(&self) -> Result<Vec<PerpetualData>, Box<dyn Error + Sync + Send>> {
        let metadata = self.answered_dexes(|dex| self.get_metadata(dex)).await?;
        Ok(metadata.into_iter().flat_map(|(dex, metadata)| map_perpetuals_data(metadata, dex)).collect())
    }

    async fn get_perpetual_candlesticks(&self, symbol: String, period: ChartPeriod) -> Result<Vec<ChartCandleStick>, Box<dyn Error + Sync + Send>> {
        let interval = candle_interval(&period);

        let end_time = chrono::Utc::now().timestamp() * 1000;
        let start_time = match period {
            ChartPeriod::Hour => end_time - 60 * 60 * 1000,
            ChartPeriod::Day => end_time - 24 * 60 * 60 * 1000,
            ChartPeriod::Week => end_time - 7 * 24 * 60 * 60 * 1000,
            ChartPeriod::Month => end_time - 30 * 24 * 60 * 60 * 1000,
            ChartPeriod::Year => end_time - 365 * 24 * 60 * 60 * 1000,
            ChartPeriod::All => 0,
        };

        let candlesticks = self.get_candlesticks(&symbol, interval, start_time, end_time).await?;
        Ok(map_candlesticks(candlesticks))
    }

    async fn get_perpetual_portfolio(&self, address: String, mode: PerpetualAccountMode) -> Result<PerpetualPortfolio, Box<dyn Error + Sync + Send>> {
        let (response, states, spot_collateral) = try_join!(self.get_perpetual_portfolio(&address), self.get_clearinghouse_states(&address), self.get_spot_collateral(&address, mode))?;
        let balance = spot_collateral.unwrap_or_else(|| map_perpetual_dexes_balance(&states));
        let positions: Vec<AssetPositions> = states.into_iter().map(|(_, positions)| positions).collect();
        Ok(map_perpetual_portfolio(response, mode, &positions, &balance))
    }

    async fn get_perpetual_addresses(&self) -> Result<Vec<PerpetualAddress>, Box<dyn Error + Sync + Send>> {
        let referral = self.get_referral(&self.config.builder_address).await?;
        Ok(map_perpetual_addresses(referral))
    }
}

#[async_trait]
impl<C: Client> ChainAddressStatus for HyperCoreClient<C> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    use gem_client::{ClientError, testkit::MockClient};
    use primitives::testkit::json::load_testdata;
    use primitives::{PerpetualId, PerpetualProvider};
    use serde_json::Value;

    #[tokio::test]
    async fn test_get_perpetual_addresses() {
        let request: Value = serde_json::from_str(r#"{"type":"referral","user":"0x0000000000000000000000000000000000000001"}"#).unwrap();
        let seen_requests = Arc::new(Mutex::new(vec![]));
        let mut client = HyperCoreClient::mock_with_responses_by_request(vec![(request.clone(), include_bytes!("../../testdata/referral_addresses.json").to_vec())], seen_requests.clone());
        client.config.builder_address = "0x0000000000000000000000000000000000000001".to_string();

        let addresses = client.get_perpetual_addresses().await.unwrap();

        assert_eq!(*seen_requests.lock().unwrap(), vec![request]);
        assert_eq!(addresses.len(), 2);
    }

    #[tokio::test]
    async fn test_get_perpetual_account_mode() {
        let unified = HyperCoreClient::mock_with_responses_by_request_type(vec![("userAbstraction", include_bytes!("../../testdata/perpetual_balance_response_user_abstraction_unified.json").to_vec())]);
        let default = HyperCoreClient::mock_with_responses_by_request_type(vec![("userAbstraction", include_bytes!("../../testdata/perpetual_positions_response_user_abstraction_default.json").to_vec())]);

        assert_eq!(unified.get_perpetual_account_mode("0x123".to_string()).await.unwrap(), PerpetualAccountMode::Unified);
        assert_eq!(default.get_perpetual_account_mode("0x123".to_string()).await.unwrap(), PerpetualAccountMode::Standard);
    }

    #[tokio::test]
    async fn test_get_positions_takes_balance_from_spot_for_unified_account() {
        let mut client = HyperCoreClient::mock_with_responses_by_request_type(vec![
            ("userAbstraction", include_bytes!("../../testdata/perpetual_balance_response_user_abstraction_unified.json").to_vec()),
            ("clearinghouseState", include_bytes!("../../testdata/perpetual_positions_response_clearinghouse_state.json").to_vec()),
            ("frontendOpenOrders", include_bytes!("../../testdata/perpetual_positions_response_open_orders.json").to_vec()),
            ("spotClearinghouseState", include_bytes!("../../testdata/perpetual_balance_response_spot_clearinghouse_state.json").to_vec()),
        ]);

        client.config.perp_dexes = vec![PerpDex::Main];
        let account = client.get_positions("0x123".to_string()).await.unwrap();
        let summary = account.summary;

        assert_eq!(account.mode, PerpetualAccountMode::Unified);
        assert_eq!(summary.balance.available, 3.797316);
        assert_eq!(summary.balance.reserved, 4.331289 - 3.797316);
        assert_eq!(summary.positions.len(), 1);
    }

    #[tokio::test]
    async fn test_get_positions_for_classification_skips_account_and_balance_requests() {
        let seen_request_types = Arc::new(Mutex::new(Vec::new()));
        let seen_request_types_clone = Arc::clone(&seen_request_types);
        let mut client = HyperCoreClient::mock_with_client(MockClient::new().with_post(move |path, body| {
            assert_eq!(path, "/info");

            let request: Value = serde_json::from_slice(body).unwrap();
            let request_type = request["type"].as_str().unwrap().to_string();
            seen_request_types_clone.lock().unwrap().push(request_type.clone());

            match request_type.as_str() {
                "clearinghouseState" => Ok(include_bytes!("../../testdata/perpetual_positions_response_clearinghouse_state.json").to_vec()),
                "frontendOpenOrders" => Ok(include_bytes!("../../testdata/perpetual_positions_response_open_orders.json").to_vec()),
                _ => Err(ClientError::Http { status: 404, body: body.to_vec() }),
            }
        }));

        client.config.perp_dexes = vec![PerpDex::Main];
        let positions = client.get_positions_for_classification("0x123".to_string()).await.unwrap();

        assert_eq!(positions.len(), 1);
        assert_eq!(seen_request_types.lock().unwrap().as_slice(), ["clearinghouseState", "frontendOpenOrders"]);
    }

    #[tokio::test]
    async fn test_get_positions() {
        let seen_requests = Arc::new(Mutex::new(Vec::new()));
        let mut client = HyperCoreClient::mock_with_responses_by_request(
            vec![
                (
                    serde_json::json!({"type": "userAbstraction", "user": "0x123"}),
                    include_bytes!("../../testdata/perpetual_positions_response_user_abstraction_default.json").to_vec(),
                ),
                (
                    load_testdata("perpetual_positions_request_clearinghouse_state.json"),
                    include_bytes!("../../testdata/perpetual_positions_response_clearinghouse_state_empty.json").to_vec(),
                ),
                (
                    load_testdata("perpetual_positions_request_clearinghouse_state_xyz.json"),
                    include_bytes!("../../testdata/perpetual_positions_response_clearinghouse_state_xyz.json").to_vec(),
                ),
                (
                    load_testdata("perpetual_positions_request_open_orders_xyz.json"),
                    include_bytes!("../../testdata/perpetual_positions_response_open_orders_xyz.json").to_vec(),
                ),
            ],
            Arc::clone(&seen_requests),
        );
        client.config.perp_dexes = vec![PerpDex::Main, PerpDex::Xyz];

        let summary = client.get_positions("0x123".to_string()).await.unwrap().summary;

        assert_eq!(
            summary
                .positions
                .iter()
                .map(|position| (position.perpetual_id.clone(), position.stop_loss.as_ref().map(|order| order.price)))
                .collect::<Vec<_>>(),
            vec![(PerpetualId::new(PerpetualProvider::Hypercore, "xyz:NVDA"), Some(90.0))]
        );
        assert_eq!(
            sorted_types(&seen_requests),
            vec!["clearinghouseState", "clearinghouseState:xyz", "frontendOpenOrders:xyz", "userAbstraction"],
            "a dex without positions asks for no open orders, and a standard account for no spot balance"
        );
    }

    #[tokio::test]
    async fn test_a_failing_dex_leaves_the_other_dexes_refreshed() {
        let user_abstraction = (
            serde_json::json!({"type": "userAbstraction", "user": "0x123"}),
            include_bytes!("../../testdata/perpetual_positions_response_user_abstraction_default.json").to_vec(),
        );
        let main_state = (
            load_testdata("perpetual_positions_request_clearinghouse_state.json"),
            include_bytes!("../../testdata/perpetual_positions_response_clearinghouse_state.json").to_vec(),
        );
        let main_orders = (
            load_testdata("perpetual_positions_request_open_orders.json"),
            include_bytes!("../../testdata/perpetual_positions_response_open_orders.json").to_vec(),
        );
        let xyz_state = (
            load_testdata("perpetual_positions_request_clearinghouse_state_xyz.json"),
            include_bytes!("../../testdata/perpetual_positions_response_clearinghouse_state_xyz.json").to_vec(),
        );
        let client = |responses: Vec<(Value, Vec<u8>)>| {
            let mut client = HyperCoreClient::mock_with_responses_by_request(responses, Arc::new(Mutex::new(Vec::new())));
            client.config.perp_dexes = vec![PerpDex::Main, PerpDex::Xyz];
            client
        };

        let without_xyz = client(vec![user_abstraction.clone(), main_state, main_orders]).get_positions("0x123".to_string()).await.unwrap();
        assert_eq!(without_xyz.summary.positions.len(), 1, "the main dex's position is refreshed while xyz fails");
        assert!(!without_xyz.is_complete);

        assert!(client(vec![user_abstraction, xyz_state]).get_positions("0x123".to_string()).await.is_err(), "without the main dex there is no account to show");

        let mut markets = HyperCoreClient::mock_with_client(MockClient::new().with_post(|_, body| {
            let request: Value = serde_json::from_slice(body).unwrap();
            match request["dex"].as_str() {
                Some(_) => Err(ClientError::Http { status: 500, body: vec![] }),
                None => Ok(include_bytes!("../../testdata/perpetual_markets_response_meta_and_asset_ctxs.json").to_vec()),
            }
        }));
        markets.config.perp_dexes = vec![PerpDex::Main, PerpDex::Xyz];
        assert!(!markets.get_perpetuals_data().await.unwrap().is_empty(), "the main dex's markets refresh while xyz fails");
    }

    #[tokio::test]
    async fn test_get_positions_requests_only_the_configured_dex() {
        let seen_requests = Arc::new(Mutex::new(Vec::new()));
        let mut client = HyperCoreClient::mock_with_responses_by_request(
            vec![
                (
                    serde_json::json!({"type": "userAbstraction", "user": "0x123"}),
                    include_bytes!("../../testdata/perpetual_positions_response_user_abstraction_default.json").to_vec(),
                ),
                (
                    load_testdata("perpetual_positions_request_clearinghouse_state.json"),
                    include_bytes!("../../testdata/perpetual_positions_response_clearinghouse_state.json").to_vec(),
                ),
                (
                    load_testdata("perpetual_positions_request_open_orders.json"),
                    include_bytes!("../../testdata/perpetual_positions_response_open_orders.json").to_vec(),
                ),
            ],
            Arc::clone(&seen_requests),
        );

        client.config.perp_dexes = vec![PerpDex::Main];
        let summary = client.get_positions("0x123".to_string()).await.unwrap().summary;

        assert_eq!(summary.positions[0].take_profit.as_ref().map(|order| order.price), Some(110.0));
        assert_eq!(sorted_types(&seen_requests), vec!["clearinghouseState", "frontendOpenOrders", "userAbstraction"]);
    }

    #[tokio::test]
    async fn test_get_perpetual_portfolio() {
        let seen_requests = Arc::new(Mutex::new(Vec::new()));
        let mut client = HyperCoreClient::mock_with_responses_by_request(
            vec![
                (serde_json::json!({"type": "portfolio", "user": "0x123"}), include_bytes!("../../testdata/perpetual_portfolio_response.json").to_vec()),
                (
                    load_testdata("perpetual_positions_request_clearinghouse_state.json"),
                    include_bytes!("../../testdata/perpetual_positions_response_clearinghouse_state.json").to_vec(),
                ),
                (
                    load_testdata("perpetual_positions_request_clearinghouse_state_xyz.json"),
                    include_bytes!("../../testdata/perpetual_positions_response_clearinghouse_state_xyz.json").to_vec(),
                ),
            ],
            Arc::clone(&seen_requests),
        );
        client.config.perp_dexes = vec![PerpDex::Main, PerpDex::Xyz];

        let portfolio = ChainPerpetual::get_perpetual_portfolio(&client, "0x123".to_string(), PerpetualAccountMode::Standard).await.unwrap();

        assert_eq!(
            sorted_types(&seen_requests),
            vec!["clearinghouseState", "clearinghouseState:xyz", "portfolio"],
            "the portfolio already covers every dex, so it is asked for once, and a standard account needs no spot balance"
        );
        assert_eq!(portfolio.all_time.unwrap().volume, 11_038_949_929.92);
        assert_eq!(portfolio.account_summary.unwrap().account_value, 10000.0 + 10000.0);
    }

    #[tokio::test]
    async fn test_get_perpetual_portfolio_unified() {
        let seen_requests = Arc::new(Mutex::new(Vec::new()));
        let mut client = HyperCoreClient::mock_with_responses_by_request(
            vec![
                (serde_json::json!({"type": "portfolio", "user": "0x123"}), include_bytes!("../../testdata/perpetual_portfolio_response.json").to_vec()),
                (
                    load_testdata("perpetual_positions_request_clearinghouse_state.json"),
                    include_bytes!("../../testdata/perpetual_positions_response_clearinghouse_state.json").to_vec(),
                ),
                (
                    serde_json::json!({"type": "spotClearinghouseState", "user": "0x123"}),
                    include_bytes!("../../testdata/perpetual_balance_response_spot_clearinghouse_state.json").to_vec(),
                ),
            ],
            Arc::clone(&seen_requests),
        );

        client.config.perp_dexes = vec![PerpDex::Main];
        let portfolio = ChainPerpetual::get_perpetual_portfolio(&client, "0x123".to_string(), PerpetualAccountMode::Unified).await.unwrap();

        assert_eq!(sorted_types(&seen_requests), vec!["clearinghouseState", "portfolio", "spotClearinghouseState"]);
        assert_eq!(portfolio.all_time.unwrap().volume, 11_038_949_929.92);
        assert_eq!(portfolio.account_summary.unwrap().account_value, 4.331289);
    }

    fn sorted_types(requests: &Mutex<Vec<Value>>) -> Vec<String> {
        let mut types: Vec<String> = requests
            .lock()
            .unwrap()
            .iter()
            .map(|request| match request["dex"].as_str() {
                Some(dex) => format!("{}:{dex}", request["type"].as_str().unwrap()),
                None => request["type"].as_str().unwrap().to_string(),
            })
            .collect();
        types.sort();
        types
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::models::perp_dex::PerpDex;
    use crate::provider::testkit::{TEST_ADDRESS, create_hypercore_test_client};
    use chain_traits::ChainPerpetual;
    use primitives::{ChartPeriod, PerpetualAccountMode};

    #[tokio::test]
    async fn test_hypercore_get_xyz_perpetuals_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut client = create_hypercore_test_client();
        client.config.perp_dexes = vec![PerpDex::Xyz];

        let data = client.get_perpetuals_data().await?;

        let xyz100 = data.iter().find(|data| data.perpetual.name == "xyz:XYZ100").unwrap();
        assert_eq!(xyz100.perpetual.identifier, "110000");
        assert!(data.iter().all(|data| data.perpetual.name.starts_with("xyz:")));
        Ok(())
    }

    #[tokio::test]
    async fn test_hypercore_get_positions() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let summary = client.get_positions(TEST_ADDRESS.to_string()).await?.summary;

        println!("Positions count: {}", summary.positions.len());
        println!("Balance: available={}, reserved={}, withdrawable={}", summary.balance.available, summary.balance.reserved, summary.balance.withdrawable);

        for pos in &summary.positions {
            println!("  {} {:?} size={} leverage={}", pos.perpetual_id, pos.direction, pos.size, pos.leverage);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_hypercore_get_perpetuals_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let data = client.get_perpetuals_data().await?;

        assert!(!data.is_empty());

        println!("Perpetuals count: {}", data.len());
        for d in data.iter().take(5) {
            println!("  {} identifier={} price={} leverage={}", d.perpetual.name, d.perpetual.identifier, d.perpetual.price, d.perpetual.max_leverage);
        }

        let btc = data.iter().find(|d| d.perpetual.name == "BTC");
        assert!(btc.is_some(), "BTC perpetual should exist");
        assert_eq!(btc.unwrap().perpetual.identifier, "0");

        let builder_assets = data.iter().filter(|d| d.perpetual.identifier.parse::<u32>().unwrap_or(0) >= 100_000).count();
        println!("Builder DEX assets: {}", builder_assets);

        Ok(())
    }

    #[tokio::test]
    async fn test_hypercore_get_perpetual_portfolio() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let portfolio = ChainPerpetual::get_perpetual_portfolio(&client, TEST_ADDRESS.to_string(), PerpetualAccountMode::Standard).await?;

        println!("Perpetual portfolio day: {:?}", portfolio.day.is_some());

        assert!(portfolio.day.is_some() || portfolio.week.is_some() || portfolio.month.is_some() || portfolio.all_time.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_hypercore_get_perpetual_candlesticks() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let candlesticks = client.get_perpetual_candlesticks("BTC".to_string(), ChartPeriod::Day).await?;

        println!("Perpetual candlesticks count: {:?}", candlesticks.len());

        assert!(!candlesticks.is_empty());
        Ok(())
    }
}

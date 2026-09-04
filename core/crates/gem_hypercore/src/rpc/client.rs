use crate::models::{
    balance::{Balances, DelegationBalance, StakeBalance, Validator},
    candlestick::Candlestick,
    metadata::HypercoreMetadataResponse,
    order::{OpenOrder, UserFill},
    perp_dex::PerpDex,
    portfolio::HypercorePortfolioResponse,
    position::AssetPositions,
    referral::Referral,
    spot::{OrderbookResponse, SpotMeta},
    user::{AgentSession, DelegatorHistoryUpdate, LedgerUpdate, UserAbstractionMode, UserFee},
};
use chain_traits::{ChainSimulation, ChainTraits};
use gem_client::{Client, ClientExt};
use primitives::InMemoryPreferences;
use serde::de::DeserializeOwned;
use std::{error::Error, sync::Arc};

use crate::config::HypercoreConfig;
use crate::models::info::{CandleSnapshotRequest, InfoRequest};
use crate::rpc::target::HyperCoreTarget;
use primitives::{Chain, Preferences};

pub(crate) const AGENT_OWNER_CACHE_PREFIX: &str = "hypercore_agent_owner_";

pub(crate) fn agent_owner_cache_key(agent_address: &str) -> String {
    format!("{AGENT_OWNER_CACHE_PREFIX}{}", agent_address.to_lowercase())
}

pub struct HyperCoreClient<C: Client> {
    client: C,
    pub chain: Chain,
    pub config: HypercoreConfig,
    pub preferences: Arc<dyn Preferences>,
    pub secure_preferences: Arc<dyn Preferences>,
}

impl<C: Client> std::fmt::Debug for HyperCoreClient<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HyperCoreClient")
            .field("chain", &self.chain)
            .field("config", &self.config)
            .field("preferences", &"<Preferences>")
            .field("secure_preferences", &"<Preferences>")
            .finish()
    }
}

impl<C: Client> HyperCoreClient<C> {
    pub fn new(client: C) -> Self {
        let preferences = Arc::new(InMemoryPreferences::new());
        let secure_preferences = Arc::new(InMemoryPreferences::new());
        Self {
            client,
            chain: Chain::HyperCore,
            config: HypercoreConfig::default(),
            preferences,
            secure_preferences,
        }
    }

    pub fn new_with_preferences(client: C, preferences: Arc<dyn Preferences>, secure_preferences: Arc<dyn Preferences>) -> Self {
        Self {
            client,
            chain: Chain::HyperCore,
            config: HypercoreConfig::default(),
            preferences,
            secure_preferences,
        }
    }

    async fn info<T>(&self, request: InfoRequest) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        T: DeserializeOwned + Send,
    {
        Ok(self.client.post(HyperCoreTarget::Info { request: request.clone() }, &request).await?)
    }

    pub async fn exchange(&self, payload: serde_json::Value) -> Result<serde_json::Value, Box<dyn Error + Send + Sync>> {
        Ok(self.client.post(HyperCoreTarget::Exchange, &payload).await?)
    }

    pub async fn get_validators(&self) -> Result<Vec<Validator>, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::ValidatorSummaries).await
    }

    pub async fn get_staking_delegations(&self, user: &str) -> Result<Vec<DelegationBalance>, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::Delegations { user: user.to_string() }).await
    }

    pub async fn get_spot_balances(&self, user: &str) -> Result<Balances, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::SpotClearinghouseState { user: user.to_string() }).await
    }

    pub async fn get_stake_balance(&self, user: &str) -> Result<StakeBalance, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::DelegatorSummary { user: user.to_string() }).await
    }

    pub async fn get_user_fills_by_time(&self, user: &str, start_time: i64) -> Result<Vec<UserFill>, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::UserFillsByTime {
            user: user.to_string(),
            start_time,
        })
        .await
    }

    pub async fn get_clearinghouse_state(&self, user: &str) -> Result<AssetPositions, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::ClearinghouseState {
            user: user.to_string(),
            dex: None,
        })
        .await
    }

    pub async fn get_clearinghouse_state_with_dex(&self, user: &str, dex: &str) -> Result<AssetPositions, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::ClearinghouseState {
            user: user.to_string(),
            dex: Some(dex.to_string()),
        })
        .await
    }

    pub async fn get_metadata(&self) -> Result<HypercoreMetadataResponse, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::MetaAndAssetCtxs { dex: None }).await
    }

    pub async fn get_metadata_with_dex(&self, dex: &str) -> Result<HypercoreMetadataResponse, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::MetaAndAssetCtxs { dex: Some(dex.to_string()) }).await
    }

    pub async fn get_perp_dexs(&self) -> Result<Vec<Option<PerpDex>>, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::PerpDexs).await
    }

    pub async fn get_spot_meta(&self) -> Result<SpotMeta, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::SpotMeta).await
    }

    pub async fn get_spot_orderbook(&self, coin: &str) -> Result<OrderbookResponse, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::L2Book { coin: coin.to_string() }).await
    }

    pub async fn get_candlesticks(&self, coin: &str, interval: &str, start_time: i64, end_time: i64) -> Result<Vec<Candlestick>, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::CandleSnapshot {
            req: CandleSnapshotRequest {
                coin: coin.to_string(),
                interval: interval.to_string(),
                start_time,
                end_time,
            },
        })
        .await
    }

    pub async fn get_user_abstraction(&self, user: &str) -> Result<UserAbstractionMode, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::UserAbstraction { user: user.to_string() }).await
    }

    pub async fn get_referral(&self, user: &str) -> Result<Referral, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::Referral { user: user.to_string() }).await
    }

    pub async fn get_extra_agents(&self, user: &str) -> Result<Vec<AgentSession>, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::ExtraAgents { user: user.to_string() }).await
    }

    pub async fn get_builder_fee(&self, user: &str, builder: &str) -> Result<u32, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::MaxBuilderFee {
            user: user.to_string(),
            builder: builder.to_string(),
        })
        .await
    }

    pub async fn get_user_fees(&self, user: &str) -> Result<UserFee, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::UserFees { user: user.to_string() }).await
    }

    pub async fn get_ledger_updates(&self, user: &str, start_time: i64) -> Result<Vec<LedgerUpdate>, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::UserNonFundingLedgerUpdates {
            user: user.to_string(),
            start_time,
        })
        .await
    }

    pub async fn get_delegator_history(&self, user: &str) -> Result<Vec<DelegatorHistoryUpdate>, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::DelegatorHistory { user: user.to_string() }).await
    }

    pub async fn get_open_orders(&self, user: &str) -> Result<Vec<OpenOrder>, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::FrontendOpenOrders {
            user: user.to_string(),
            dex: None,
        })
        .await
    }

    pub async fn get_open_orders_with_dex(&self, user: &str, dex: &str) -> Result<Vec<OpenOrder>, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::FrontendOpenOrders {
            user: user.to_string(),
            dex: Some(dex.to_string()),
        })
        .await
    }

    pub async fn get_perpetual_portfolio(&self, user: &str) -> Result<HypercorePortfolioResponse, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::Portfolio {
            user: user.to_string(),
            dex: None,
        })
        .await
    }

    pub async fn get_perpetual_portfolio_with_dex(&self, user: &str, dex: &str) -> Result<HypercorePortfolioResponse, Box<dyn Error + Send + Sync>> {
        self.info(InfoRequest::Portfolio {
            user: user.to_string(),
            dex: Some(dex.to_string()),
        })
        .await
    }
}

impl<C: Client> ChainTraits for HyperCoreClient<C> {}

impl<C: Client> ChainSimulation for HyperCoreClient<C> {}

impl<C: Client> chain_traits::ChainProvider for HyperCoreClient<C> {
    fn get_chain(&self) -> primitives::Chain {
        Chain::HyperCore
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_client::Target;
    use gem_client::testkit::MockClient;
    use serde_json::json;
    use std::sync::Mutex;

    #[tokio::test]
    async fn test_get_user_abstraction_sets_cache_ttl_header() {
        let seen_headers = Arc::new(Mutex::new(Vec::new()));
        let seen_headers_clone = Arc::clone(&seen_headers);
        let client = MockClient::new().with_post_with_headers(move |path, body, headers| {
            assert_eq!(path, "/info");
            let request: serde_json::Value = serde_json::from_slice(body).unwrap();
            assert_eq!(
                request,
                json!({
                    "type": "userAbstraction",
                    "user": "0x123"
                })
            );
            seen_headers_clone.lock().unwrap().push(headers.clone());
            Ok(br#""default""#.to_vec())
        });
        let client = HyperCoreClient::new(client);

        let mode = client.get_user_abstraction("0x123").await.unwrap();
        let recorded_headers = seen_headers.lock().unwrap().clone();

        assert_eq!(mode, UserAbstractionMode::Default);
        assert_eq!(
            recorded_headers,
            vec![
                HyperCoreTarget::Info {
                    request: InfoRequest::UserAbstraction { user: "0x123".to_string() }
                }
                .headers()
            ]
        );
    }
}

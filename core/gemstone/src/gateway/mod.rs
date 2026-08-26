mod chain_factory;
mod error;
mod preferences;

pub use chain_factory::ChainClientFactory;
pub use error::GatewayError;
use error::map_network_error;
#[cfg(test)]
pub use preferences::EmptyPreferences;
pub use preferences::GemPreferences;
pub(crate) use preferences::PreferencesWrapper;

use crate::alien::{AlienProvider, AlienProviderWrapper, coalescing_provider};
use crate::models::*;
use crate::transaction_state::StatusProvider;
use chain_traits::ChainTraits;
use std::future::Future;
use std::sync::Arc;
use swapper::swapper::GemSwapper as Swapper;
use yielder::Yielder;

use primitives::{AssetBalance, AssetId, Chain, ChartPeriod, Transaction, TransactionUpdate};

#[derive(uniffi::Object)]
pub struct GemGateway {
    chain_factory: Arc<ChainClientFactory>,
    yielder: Yielder,
    status_provider: StatusProvider,
}

impl std::fmt::Debug for GemGateway {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GemGateway").finish()
    }
}

impl GemGateway {
    pub async fn get_balance_coin(&self, chain: Chain, address: String) -> Result<AssetBalance, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_balance_coin(address).await }).await
    }

    pub async fn get_balance_tokens(&self, chain: Chain, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_balance_tokens(address, token_ids).await })
            .await
    }

    pub async fn get_balance_staking(&self, chain: Chain, address: String) -> Result<Option<AssetBalance>, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_balance_staking(address).await }).await
    }

    pub async fn get_balance_earn(&self, chain: Chain, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, GatewayError> {
        Ok(self.yielder.get_balance(chain, &address, &token_ids).await)
    }

    pub async fn get_transaction_update(&self, transaction: Transaction) -> Result<TransactionUpdate, GatewayError> {
        Ok(self.status_provider.get_update(&transaction).await?)
    }

    async fn with_provider<T, F, Fut>(&self, chain: Chain, call: F) -> Result<T, GatewayError>
    where
        F: FnOnce(Arc<dyn ChainTraits>) -> Fut,
        Fut: Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
    {
        let provider = self.chain_factory.create(chain).await?;
        call(provider).await.map_err(|e| GatewayError::NetworkError { msg: e.to_string() })
    }
}

#[uniffi::export]
impl GemGateway {
    #[uniffi::constructor]
    pub fn new(provider: Arc<dyn AlienProvider>, preferences: Arc<dyn GemPreferences>, secure_preferences: Arc<dyn GemPreferences>) -> Self {
        let provider = coalescing_provider(provider);
        let chain_factory = Arc::new(ChainClientFactory::new(provider.clone(), preferences, secure_preferences));
        let alien_wrapper = Arc::new(AlienProviderWrapper::new(provider));
        let yielder = Yielder::new(alien_wrapper.clone());
        let swapper = Swapper::new(alien_wrapper);
        let status_provider = StatusProvider::new(chain_factory.clone(), swapper);
        Self {
            chain_factory,
            yielder,
            status_provider,
        }
    }

    pub async fn get_staking_validators(&self, chain: Chain, apy: Option<f64>) -> Result<Vec<GemDelegationValidator>, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_staking_validators(apy).await }).await
    }

    pub async fn get_staking_delegation_validators(&self, chain: Chain, address: String) -> Result<Vec<GemDelegationValidator>, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_staking_delegation_validators(address).await })
            .await
    }

    pub async fn get_staking_delegations(&self, chain: Chain, address: String) -> Result<Vec<GemDelegationBase>, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_staking_delegations(address).await }).await
    }

    pub async fn transaction_broadcast(&self, chain: Chain, data: String, options: GemBroadcastOptions) -> Result<String, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.transaction_broadcast(data, options).await })
            .await
    }

    pub async fn get_chain_id(&self, chain: Chain) -> Result<String, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_chain_id().await }).await
    }

    pub async fn get_block_number(&self, chain: Chain) -> Result<u64, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_block_latest_number().await }).await
    }

    pub async fn get_fee_rates(&self, chain: Chain, input: GemTransactionInputType) -> Result<Vec<GemFeeRate>, GatewayError> {
        let fees = self
            .with_provider(chain, |provider| async move { provider.get_transaction_fee_rates(input.into()).await })
            .await?;
        Ok(fees.into_iter().map(|f| f.into()).collect())
    }

    pub async fn get_utxos(&self, chain: Chain, address: String) -> Result<Vec<GemUTXO>, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_utxos(address).await }).await
    }

    pub async fn get_transaction_preload(&self, chain: Chain, input: GemTransactionPreloadInput) -> Result<GemTransactionLoadMetadata, GatewayError> {
        let preload_input: primitives::TransactionPreloadInput = input.into();
        let metadata = self
            .with_provider(chain, |provider| async move { provider.get_transaction_preload(preload_input).await })
            .await?;
        Ok(metadata)
    }

    pub async fn get_transaction_load(&self, chain: Chain, input: GemTransactionLoadInput) -> Result<GemTransactionData, GatewayError> {
        let load_data = self
            .with_provider(chain, |chain_provider| async move { chain_provider.get_transaction_load(input.into()).await })
            .await?;

        Ok(GemTransactionData {
            fee: load_data.fee.into(),
            metadata: load_data.metadata,
        })
    }

    pub async fn get_positions(&self, chain: Chain, address: String) -> Result<GemPerpetualPositionsSummary, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_positions(address).await }).await
    }

    pub async fn get_perpetual_account_mode(&self, chain: Chain, address: String) -> Result<GemPerpetualAccountMode, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_perpetual_account_mode(address).await })
            .await
    }

    pub async fn get_perpetuals_data(&self, chain: Chain) -> Result<Vec<GemPerpetualData>, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_perpetuals_data().await }).await
    }

    pub async fn get_perpetual_candlesticks(&self, chain: Chain, symbol: String, period: String) -> Result<Vec<GemChartCandleStick>, GatewayError> {
        let chart_period = ChartPeriod::new(period).unwrap();
        self.with_provider(chain, |provider| async move { provider.get_perpetual_candlesticks(symbol, chart_period).await })
            .await
    }

    pub async fn get_perpetual_portfolio(&self, chain: Chain, address: String) -> Result<primitives::portfolio::PerpetualPortfolio, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_perpetual_portfolio(address).await }).await
    }

    pub async fn get_token_data(&self, chain: Chain, token_id: String) -> Result<GemAsset, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_token_data(token_id).await }).await
    }

    pub async fn get_is_token_address(&self, chain: Chain, token_id: String) -> Result<bool, GatewayError> {
        Ok(self.chain_factory.create(chain).await?.get_is_token_address(&token_id))
    }

    pub async fn get_earn_data(&self, asset_id: AssetId, address: String, value: String, earn_type: GemEarnType) -> Result<GemContractCallData, GatewayError> {
        self.yielder
            .get_data(&asset_id, &address, &value, &earn_type)
            .await
            .map_err(|e| GatewayError::NetworkError { msg: e.to_string() })
    }

    pub fn get_earn_providers(&self, asset_id: AssetId) -> Vec<GemDelegationValidator> {
        self.yielder.get_providers(&asset_id)
    }

    pub async fn get_earn_positions(&self, address: String, asset_id: AssetId) -> Vec<GemDelegationBase> {
        self.yielder.get_positions(&address, &asset_id).await
    }

    pub async fn get_node_status(&self, chain: Chain, url: &str) -> Result<GemNodeStatus, GatewayError> {
        let provider = self.chain_factory.create_with_url(chain, url.to_string()).await?;
        provider.get_nodes_status().await.map_err(map_network_error)
    }
}

#[cfg(all(test, feature = "reqwest_provider"))]
mod tests {
    use super::*;
    use crate::testkit::TestAlienProvider;

    #[test]
    fn test_get_node_status_http_404_error() {
        let provider: Arc<dyn AlienProvider> = Arc::new(TestAlienProvider::with_status(404));
        let preferences: Arc<dyn GemPreferences> = Arc::new(EmptyPreferences {});
        let gateway = GemGateway::new(provider, preferences.clone(), preferences.clone());

        let result = futures::executor::block_on(gateway.get_node_status(Chain::Bitcoin, "https://httpbin.org/status/404"));

        match result {
            Ok(status) => panic!("expected network error for 404 response, got {:?}", status),
            Err(GatewayError::NetworkError { msg }) => assert_eq!(msg, "HTTP error: status 404"),
            Err(GatewayError::PlatformError { .. }) => panic!("expected NetworkError, got PlatformError"),
        }
    }
}

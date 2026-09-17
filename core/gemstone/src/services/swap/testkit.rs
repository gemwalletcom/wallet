use std::sync::{Arc, Mutex};

use num_bigint::{BigInt, BigUint};
use primitives::{AssetId, Chain, WalletId};
use swapper::{Quote, SwapperProvider};

use super::GemSwapService;
use super::model::{GemSwapButtonInput, GemSwapPair};
use super::session::{GemSwapQuotesResult, GemSwapRequest, GemSwapSession};
use super::store::GemSwapStore;
use crate::gem_swapper::GemSwapper;
use crate::keystore::GemKeystore;
use crate::services::error::GemServiceError;
use crate::services::node::GemNodeService;
use crate::services::wallet::testkit::MemoryKeystorePassword;
use crate::testkit::TestAlienProvider;

#[derive(Default)]
pub struct MemorySwapStore {
    pub pairs: Mutex<Vec<GemSwapPair>>,
    pub recent_asset_ids: Mutex<Vec<AssetId>>,
    pub pay_asset_ids: Mutex<Vec<AssetId>>,
    pub receive_asset_ids: Mutex<Vec<AssetId>>,
    pub receive_requests: Mutex<Vec<(Vec<Chain>, Vec<AssetId>)>>,
}

#[async_trait::async_trait]
impl GemSwapStore for MemorySwapStore {
    async fn get_swap_pairs(&self, _wallet_id: WalletId) -> Result<Vec<GemSwapPair>, GemServiceError> {
        Ok(self.pairs.lock().unwrap().clone())
    }

    async fn get_recent_asset_ids(&self, _wallet_id: WalletId) -> Result<Vec<AssetId>, GemServiceError> {
        Ok(self.recent_asset_ids.lock().unwrap().clone())
    }

    async fn get_pay_asset_ids(&self, _wallet_id: WalletId) -> Result<Vec<AssetId>, GemServiceError> {
        Ok(self.pay_asset_ids.lock().unwrap().clone())
    }

    async fn get_receive_asset_ids(&self, _wallet_id: WalletId, chains: Vec<Chain>, asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, GemServiceError> {
        self.receive_requests.lock().unwrap().push((chains, asset_ids));
        Ok(self.receive_asset_ids.lock().unwrap().clone())
    }
}

impl GemSwapService {
    pub fn mock(store: Arc<MemorySwapStore>) -> Self {
        Self::new(
            Arc::new(GemSwapper::new(Arc::new(TestAlienProvider::with_status(200)), Arc::new(GemNodeService::mock()))),
            GemKeystore::new(std::env::temp_dir().to_string_lossy().to_string()).unwrap(),
            Arc::new(MemoryKeystorePassword::default()),
            store,
        )
    }
}

impl GemSwapPair {
    pub fn mock(from: Chain, to: Chain) -> Self {
        Self {
            from_asset_id: AssetId::from_chain(from),
            to_asset_id: AssetId::from_chain(to),
        }
    }
}

impl GemSwapRequest {
    pub fn mock() -> Self {
        Self {
            pay_asset_id: AssetId::from_chain(Chain::Ethereum),
            receive_asset_id: AssetId::from_chain(Chain::Solana),
            value: BigUint::from(100u32),
            slippage_bps: None,
        }
    }
}

impl GemSwapButtonInput {
    pub fn mock(value: u64, available_balance: u64) -> Self {
        Self {
            value: BigInt::from(value),
            available_balance: BigInt::from(available_balance),
            quote_error: None,
            transfer_error: None,
        }
    }
}

impl GemSwapQuotesResult {
    pub fn mock(quotes: Vec<Quote>) -> Self {
        Self {
            request: GemSwapRequest::mock(),
            quotes,
            error: None,
        }
    }
}

impl GemSwapSession {
    pub fn mock_ready() -> Self {
        Self::default()
            .on_request_changed(Some(GemSwapRequest::mock()))
            .on_quote_results(GemSwapQuotesResult::mock(vec![
                Quote::mock_with_provider(SwapperProvider::Okx, "10"),
                Quote::mock_with_provider(SwapperProvider::Jupiter, "9"),
            ]))
    }
}

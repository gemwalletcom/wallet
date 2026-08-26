pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{Chain, WalletId};

pub use store::GemPerpetualStore;

use crate::gateway::GemGateway;
use crate::services::price::GemPriceService;

#[derive(uniffi::Object)]
pub struct GemPerpetualService {
    gateway: Arc<GemGateway>,
    price: Arc<GemPriceService>,
    store: Arc<dyn GemPerpetualStore>,
}

#[uniffi::export]
impl GemPerpetualService {
    #[uniffi::constructor]
    pub fn new(gateway: Arc<GemGateway>, price: Arc<GemPriceService>, store: Arc<dyn GemPerpetualStore>) -> Self {
        Self { gateway, price, store }
    }

    pub async fn sync_markets(&self, chain: Chain) -> Result<(), GemServiceError> {
        let data = self.gateway.get_perpetuals_data(chain).await?;
        self.store.save_perpetuals(data).await?;
        if let Some(price) = rules::collateral_price(chain) {
            self.price.update_prices(vec![price], Currency::USD).await?;
        }
        Ok(())
    }

    pub async fn sync_positions(&self, wallet_id: WalletId, chain: Chain, address: String) -> Result<(), GemServiceError> {
        let summary = self.gateway.get_positions(chain, address).await?;
        let existing_ids = self.store.get_position_ids(wallet_id.clone(), rules::provider(chain)).await?;
        let delete_ids = rules::stale_position_ids(existing_ids, &summary.positions);
        self.store.update_positions(wallet_id.clone(), summary.positions, delete_ids).await?;
        self.store.update_balance(wallet_id, summary.balance).await
    }
}

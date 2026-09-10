use async_trait::async_trait;
use primitives::{AssetId, Chain, ChainAddress};

use crate::{PaymentError, PaymentLoad};

#[async_trait]
pub(crate) trait PaymentProvider: Send + Sync {
    fn supported_chains(&self) -> &'static [Chain];

    async fn load(&self, addresses: &[ChainAddress]) -> Result<PaymentLoad, PaymentError>;

    async fn select_asset(&self, _addresses: &[ChainAddress], _asset_id: AssetId) -> Result<PaymentLoad, PaymentError> {
        Err(PaymentError::InvalidRequest {
            reason: "Payment offers no choice of asset".to_string(),
        })
    }

    async fn confirm(&self, quote_id: &str, transaction_hash: String) -> Result<(), PaymentError>;
}

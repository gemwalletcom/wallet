use async_trait::async_trait;
use primitives::{AssetId, Chain, ChainAddress};

use crate::{PaymentError, PaymentLoad, PaymentUpdate};

#[async_trait]
pub(crate) trait PaymentProvider: Send + Sync {
    fn supported_chains(&self) -> &'static [Chain];

    async fn load(&self, addresses: &[ChainAddress]) -> Result<PaymentLoad, PaymentError>;

    async fn select_asset(&self, _addresses: &[ChainAddress], _asset_id: AssetId) -> Result<PaymentLoad, PaymentError> {
        Err(PaymentError::invalid_request("Payment offers no choice of asset"))
    }

    async fn confirm(&self, quote_id: &str, action_results: Vec<String>) -> Result<(), PaymentError>;

    async fn status(&self) -> Result<PaymentUpdate, PaymentError> {
        Err(PaymentError::invalid_request("Payment reports no status"))
    }
}

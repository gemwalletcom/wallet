use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::{AddressName, Chain};

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemAddressStore: Send + Sync {
    async fn get_address_name(&self, chain: Chain, address: String) -> Result<Option<AddressName>, GemServiceError>;
    async fn save_address_names(&self, names: Vec<AddressName>) -> Result<(), GemServiceError>;
}

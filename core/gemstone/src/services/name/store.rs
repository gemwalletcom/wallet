use async_trait::async_trait;
use primitives::{AddressName, Chain};

use super::error::GemNameError;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemAddressStore: Send + Sync {
    async fn get_address_name(&self, chain: Chain, address: String) -> Result<Option<AddressName>, GemNameError>;
    async fn save_address_names(&self, names: Vec<AddressName>) -> Result<(), GemNameError>;
}

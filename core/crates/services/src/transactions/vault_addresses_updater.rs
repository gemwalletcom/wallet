use std::error::Error;
use std::sync::Arc;

use cacher::{SwapVaultAddressCacher, SwapVaultAddressKind};
use primitives::SwapProvider;
use swapper::swapper::GemSwapper;

pub struct VaultAddressesUpdater {
    swapper: Arc<GemSwapper>,
    cacher: Arc<dyn SwapVaultAddressCacher>,
}

impl VaultAddressesUpdater {
    pub fn new(swapper: Arc<GemSwapper>, cacher: Arc<dyn SwapVaultAddressCacher>) -> Self {
        Self { swapper, cacher }
    }

    pub async fn update(&self, provider: SwapProvider, from_timestamp: Option<u64>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let vault_addresses = self.swapper.get_vault_addresses(&provider, from_timestamp).await?;
        let deposit_count = self.cacher.add_vault_addresses(provider, SwapVaultAddressKind::Deposit, &vault_addresses.deposit).await?;
        let send_count = self.cacher.add_vault_addresses(provider, SwapVaultAddressKind::Send, &vault_addresses.send).await?;
        Ok(deposit_count + send_count)
    }
}

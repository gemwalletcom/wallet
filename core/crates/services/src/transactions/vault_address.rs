use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::sync::Arc;

use cacher::{SwapVaultAddressCacher, SwapVaultAddressKind};
use primitives::SwapProvider;
use swapper::SwapperProvider;

type AddressMap = HashMap<String, SwapperProvider>;

#[derive(Clone)]
pub struct SwapVaultAddressClient {
    cacher: Arc<dyn SwapVaultAddressCacher>,
}

impl SwapVaultAddressClient {
    pub fn new(cacher: Arc<dyn SwapVaultAddressCacher>) -> Self {
        Self { cacher }
    }

    pub async fn get_deposit_address_map(&self) -> Result<AddressMap, Box<dyn Error + Send + Sync>> {
        self.get_address_map(SwapVaultAddressKind::Deposit).await
    }

    pub async fn get_send_address_map(&self) -> Result<AddressMap, Box<dyn Error + Send + Sync>> {
        self.get_address_map(SwapVaultAddressKind::Send).await
    }

    pub async fn add_deposit_ids(&self, deposit_ids: BTreeMap<SwapperProvider, Vec<String>>) -> Result<(), Box<dyn Error + Send + Sync>> {
        for (provider, ids) in deposit_ids {
            self.cacher.add_vault_addresses(provider, SwapVaultAddressKind::Deposit, &ids).await?;
        }
        Ok(())
    }

    async fn get_address_map(&self, kind: SwapVaultAddressKind) -> Result<AddressMap, Box<dyn Error + Send + Sync>> {
        let providers = SwapProvider::cross_chain_providers();
        let results = self.cacher.vault_addresses(&providers, kind).await?;
        Ok(providers.into_iter().zip(results).flat_map(|(provider, members)| members.into_iter().map(move |address| (address, provider))).collect())
    }
}

use std::collections::{BTreeMap, HashMap, HashSet};
use std::error::Error;

use cacher::{CacherClient, SwapVaultAddressKind};
use primitives::{SwapProvider, Transaction};
use swapper::{SwapperProvider, cross_chain};

type AddressMap = HashMap<String, SwapperProvider>;

pub trait DepositAddressLookup {
    fn get_deposit_address_map(&self, transactions: &[Transaction]) -> impl Future<Output = Result<AddressMap, Box<dyn Error + Send + Sync>>> + Send;
}

#[derive(Clone)]
pub struct SwapVaultAddressClient {
    cacher: CacherClient,
}

impl SwapVaultAddressClient {
    pub fn new(cacher: CacherClient) -> Self {
        Self { cacher }
    }

    pub async fn get_deposit_address_map(&self, transactions: &[Transaction]) -> Result<AddressMap, Box<dyn Error + Send + Sync>> {
        let candidates = transactions.iter().flat_map(cross_chain::deposit_candidates).collect::<HashSet<_>>().into_iter().collect::<Vec<_>>();
        let providers = SwapProvider::cross_chain_providers();
        let results = self.cacher.matching_deposit_ids(&providers, &candidates).await?;
        Ok(address_map(providers, results))
    }

    pub async fn get_send_address_map(&self) -> Result<AddressMap, Box<dyn Error + Send + Sync>> {
        let providers = SwapProvider::cross_chain_providers();
        let results = self.cacher.send_addresses(&providers).await?;
        Ok(address_map(providers, results))
    }

    pub async fn add_deposit_ids(&self, deposit_ids: BTreeMap<SwapperProvider, Vec<String>>) -> Result<(), Box<dyn Error + Send + Sync>> {
        for (provider, ids) in deposit_ids {
            self.cacher.add_vault_addresses(provider, SwapVaultAddressKind::Deposit, &ids).await?;
        }
        Ok(())
    }
}

fn address_map(providers: Vec<SwapProvider>, results: Vec<Vec<String>>) -> AddressMap {
    providers.into_iter().zip(results).flat_map(|(provider, members)| members.into_iter().map(move |address| (address, provider))).collect()
}

impl DepositAddressLookup for SwapVaultAddressClient {
    async fn get_deposit_address_map(&self, transactions: &[Transaction]) -> Result<AddressMap, Box<dyn Error + Send + Sync>> {
        SwapVaultAddressClient::get_deposit_address_map(self, transactions).await
    }
}

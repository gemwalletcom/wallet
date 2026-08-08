use std::collections::HashMap;
use std::error::Error;

use crate::models::{Account, AssetDetails, TransactionBroadcast, TransactionStatus, TransactionsParams};
use gem_client::{CONTENT_TYPE, ContentType};

#[cfg(feature = "rpc")]
use gem_client::{Client, ClientExt};
#[cfg(feature = "rpc")]
use primitives::Chain;

#[derive(Debug)]
pub struct AlgorandClient<C: Client> {
    client: C,
    pub chain: Chain,
}

impl<C: Client> AlgorandClient<C> {
    pub fn new(client: C) -> Self {
        Self { client, chain: Chain::Algorand }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain
    }

    pub async fn get_account(&self, address: &str) -> Result<Account, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v2/accounts/{}", address)).await?)
    }

    pub async fn get_asset(&self, asset_id: &str) -> Result<AssetDetails, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v2/assets/{}", asset_id)).await?)
    }

    pub async fn get_transactions_params(&self) -> Result<TransactionsParams, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/v2/transactions/params").await?)
    }

    pub async fn broadcast_transaction(&self, data: &str) -> Result<TransactionBroadcast, Box<dyn Error + Send + Sync>> {
        let headers = HashMap::from([(CONTENT_TYPE.to_string(), ContentType::ApplicationXBinary.as_str().to_string())]);

        Ok(self.client.post_with_headers("/v2/transactions", &data, headers).await?)
    }

    pub async fn get_pending_transaction(&self, transaction_id: &str) -> Result<TransactionStatus, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v2/transactions/pending/{}", transaction_id)).await?)
    }
}

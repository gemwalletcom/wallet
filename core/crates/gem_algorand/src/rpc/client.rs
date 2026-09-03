use std::error::Error;

use gem_client::{Client, ClientError, ClientExt};
use primitives::Chain;
use serde::de::DeserializeOwned;

use crate::models::{Account, AssetDetails, TransactionBroadcast, TransactionStatus, TransactionsParams};
use crate::rpc::target::AlgorandTarget;

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

    async fn send<R: DeserializeOwned + Send>(&self, target: AlgorandTarget) -> Result<R, ClientError> {
        let path = target.path();
        let headers = target.headers();
        match target.body() {
            Some(body) => self.client.post(&path, body).headers(headers).await,
            None => self.client.get(&path).headers(headers).await,
        }
    }

    pub async fn get_account(&self, address: &str) -> Result<Account, Box<dyn Error + Send + Sync>> {
        Ok(self.send(AlgorandTarget::GetAccount { address: address.to_string() }).await?)
    }

    pub async fn get_asset(&self, asset_id: &str) -> Result<AssetDetails, Box<dyn Error + Send + Sync>> {
        Ok(self.send(AlgorandTarget::GetAsset { asset_id: asset_id.to_string() }).await?)
    }

    pub async fn get_transactions_params(&self) -> Result<TransactionsParams, Box<dyn Error + Send + Sync>> {
        Ok(self.send(AlgorandTarget::GetTransactionsParams).await?)
    }

    pub async fn broadcast_transaction(&self, data: &str) -> Result<TransactionBroadcast, Box<dyn Error + Send + Sync>> {
        Ok(self.send(AlgorandTarget::SendTransaction { transaction: data.to_string() }).await?)
    }

    pub async fn get_pending_transaction(&self, transaction_id: &str) -> Result<TransactionStatus, Box<dyn Error + Send + Sync>> {
        Ok(self
            .send(AlgorandTarget::GetPendingTransaction {
                transaction_id: transaction_id.to_string(),
            })
            .await?)
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;
    use gem_client::{CONTENT_TYPE, ContentType};

    use super::*;

    #[tokio::test]
    async fn test_broadcast_transaction() {
        let client = AlgorandClient::new(MockClient::new().with_post_with_headers(|path, body, headers| {
            assert_eq!(path, "/v2/transactions");
            assert_eq!(body, br#""deadbeef""#);
            assert_eq!(headers.get(CONTENT_TYPE).map(String::as_str), Some(ContentType::ApplicationXBinary.as_str()));
            Ok(br#"{"txId":"TXID"}"#.to_vec())
        }));

        let broadcast = client.broadcast_transaction("deadbeef").await.unwrap();

        assert_eq!(
            broadcast,
            TransactionBroadcast {
                tx_id: Some("TXID".to_string()),
                message: None,
            }
        );
    }
}

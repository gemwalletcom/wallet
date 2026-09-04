use std::error::Error;

use gem_client::{Client, ClientExt};

use crate::models::account::PolkadotAccountBalance;
use crate::models::block::PolkadotNodeVersion;
use crate::models::fee::PolkadotEstimateFee;
use crate::models::rpc::{Block, BlockHeader};
use crate::models::transaction::{PolkadotTransactionBroadcastResponse, PolkadotTransactionMaterial, PolkadotTransactionPayload};
use crate::rpc::target::PolkadotTarget;

pub struct PolkadotClient<C: Client> {
    pub client: C,
}

impl<C: Client> PolkadotClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_balance(&self, address: String) -> Result<PolkadotAccountBalance, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(PolkadotTarget::GetBalance { address }).await?)
    }

    pub async fn get_transaction_material(&self) -> Result<PolkadotTransactionMaterial, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(PolkadotTarget::GetTransactionMaterial).await?)
    }

    pub async fn estimate_fee(&self, transaction: &str) -> Result<PolkadotEstimateFee, Box<dyn Error + Send + Sync>> {
        let payload = PolkadotTransactionPayload { tx: transaction.to_string() };
        Ok(self.client.post(PolkadotTarget::EstimateFee, &payload).await?)
    }

    pub async fn get_node_version(&self) -> Result<PolkadotNodeVersion, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(PolkadotTarget::GetNodeVersion).await?)
    }

    pub async fn get_block_head(&self) -> Result<Block, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(PolkadotTarget::GetBlockHead).await?)
    }

    pub async fn get_blocks(&self, from: &str, to: &str) -> Result<Vec<Block>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(PolkadotTarget::GetBlocks {
                from: from.to_string(),
                to: to.to_string(),
            })
            .await?)
    }

    pub async fn broadcast_transaction(&self, transaction: String) -> Result<PolkadotTransactionBroadcastResponse, Box<dyn Error + Send + Sync>> {
        let payload = PolkadotTransactionPayload { tx: transaction };
        Ok(self.client.post(PolkadotTarget::SendTransaction, &payload).await?)
    }

    pub async fn get_block_header(&self, block: &str) -> Result<BlockHeader, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(PolkadotTarget::GetBlockHeader { block: block.to_string() }).await?)
    }

    pub async fn get_block(&self, block_number: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(PolkadotTarget::GetBlock { number: block_number }).await?)
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;

    use super::*;

    #[tokio::test]
    async fn test_broadcast_transaction() {
        let client = PolkadotClient::new(MockClient::new().with_post(|path, body| {
            assert_eq!(path, "/v1/transaction");
            assert_eq!(body, br#"{"tx":"0xsigned"}"#);
            Ok(br#"{"hash":"0xhash"}"#.to_vec())
        }));

        let broadcast = client.broadcast_transaction("0xsigned".to_string()).await.unwrap();

        assert_eq!(broadcast.hash.as_deref(), Some("0xhash"));
    }
}

use std::error::Error;

use gem_client::{Client, ClientExt};

use crate::models::account::PolkadotAccountBalance;
use crate::models::block::PolkadotNodeVersion;
use crate::models::fee::PolkadotEstimateFee;
use crate::models::rpc::{Block, BlockHeader};
use crate::models::transaction::{PolkadotTransactionBroadcastResponse, PolkadotTransactionMaterial, PolkadotTransactionPayload};

pub struct PolkadotClient<C: Client> {
    pub client: C,
}

impl<C: Client> PolkadotClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_balance(&self, address: String) -> Result<PolkadotAccountBalance, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v1/accounts/{}/balance-info", address)).await?)
    }

    pub async fn get_transaction_material(&self) -> Result<PolkadotTransactionMaterial, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/v1/transaction/material").await?)
    }

    pub async fn estimate_fee(&self, transaction: &str) -> Result<PolkadotEstimateFee, Box<dyn Error + Send + Sync>> {
        let payload = PolkadotTransactionPayload { tx: transaction.to_string() };
        Ok(self.client.post("/v1/transaction/fee-estimate", &payload).await?)
    }

    pub async fn get_node_version(&self) -> Result<PolkadotNodeVersion, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/v1/node/version").await?)
    }

    pub async fn get_block_head(&self) -> Result<Block, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/v1/blocks/head").await?)
    }

    pub async fn get_blocks(&self, from: &str, to: &str) -> Result<Vec<Block>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v1/blocks?range={}-{}&noFees=true", from, to)).await?)
    }

    pub async fn broadcast_transaction(&self, transaction: String) -> Result<PolkadotTransactionBroadcastResponse, Box<dyn Error + Send + Sync>> {
        let payload = PolkadotTransactionPayload { tx: transaction };
        Ok(self.client.post("/v1/transaction", &payload).await?)
    }

    pub async fn get_block_header(&self, block: &str) -> Result<BlockHeader, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v1/blocks/{}/header", block)).await?)
    }

    pub async fn get_block(&self, block_number: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v1/blocks/{}", block_number)).await?)
    }
}

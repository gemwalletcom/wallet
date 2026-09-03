use std::error::Error;

use chain_traits::{ChainAddressStatus, ChainPerpetual, ChainProvider, ChainSimulation, ChainStaking, ChainTraits};
use gem_client::{Client, ClientError, ClientExt};
use primitives::chain::Chain;
use serde::de::DeserializeOwned;

use crate::models::{
    account::BalanceResponse,
    block::{Block, BlockData, GenesisData},
    rpc::{AddressTransaction, AddressTransactions, Block as RpcBlock, Blocks, Data},
    transaction::TransactionBroadcast,
    utxo::{UTXO, UTXOS},
};
use crate::rpc::target::CardanoTarget;
use primitives::graphql::GraphqlData;

#[derive(Debug)]
pub struct CardanoClient<C: Client> {
    client: C,
    chain: Chain,
}

impl<C: Client> CardanoClient<C> {
    pub fn new(client: C) -> Self {
        Self { client, chain: Chain::Cardano }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain
    }

    async fn query<R: DeserializeOwned + Send>(&self, target: CardanoTarget) -> Result<R, ClientError> {
        let body = target.body();
        self.client.post(target, &body).await
    }

    pub(crate) async fn get_tip(&self) -> Result<Block, Box<dyn Error + Send + Sync>> {
        Ok(self.query::<Data<BlockData>>(CardanoTarget::Tip).await?.data.cardano.tip)
    }

    pub async fn get_block(&self, block_number: u64) -> Result<RpcBlock, Box<dyn Error + Send + Sync>> {
        let response: Data<Blocks> = self.query(CardanoTarget::Block { number: block_number }).await?;
        response.data.blocks.first().cloned().ok_or_else(|| "Block not found".into())
    }

    pub async fn get_address_transactions(&self, address: &str, limit: usize) -> Result<Vec<AddressTransaction>, Box<dyn Error + Send + Sync>> {
        let target = CardanoTarget::AddressTransactions {
            address: address.to_string(),
            limit,
        };
        Ok(self.query::<Data<AddressTransactions>>(target).await?.data.transactions)
    }

    pub async fn get_balance(&self, address: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let response: GraphqlData<BalanceResponse> = self.query(CardanoTarget::Balance { address: address.to_string() }).await?;

        if let Some(errors) = response.errors
            && let Some(error) = errors.first()
        {
            return Err(error.message.clone().into());
        }

        if let Some(data) = response.data {
            Ok(data.utxos.aggregate.sum.value.unwrap_or_else(|| "0".to_string()))
        } else {
            Ok("0".to_string())
        }
    }

    pub async fn get_utxos(&self, address: &str) -> Result<Vec<UTXO>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .query::<Data<UTXOS<Vec<UTXO>>>>(CardanoTarget::Utxos { address: address.to_string() })
            .await?
            .data
            .utxos)
    }

    pub async fn get_network_magic(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(self
            .query::<Data<GenesisData>>(CardanoTarget::NetworkMagic)
            .await?
            .data
            .genesis
            .shelley
            .network_magic
            .to_string())
    }

    pub async fn broadcast_transaction(&self, data: String) -> Result<String, Box<dyn Error + Send + Sync>> {
        let response: GraphqlData<TransactionBroadcast> = self.query(CardanoTarget::SubmitTransaction { transaction: data }).await?;

        if let Some(errors) = response.errors
            && let Some(error) = errors.first()
        {
            return Err(error.message.clone().into());
        }

        if let Some(data) = response.data
            && let Some(submit_transaction) = data.submit_transaction
        {
            return Ok(submit_transaction.hash);
        }

        Err("Failed to broadcast transaction - no data or hash returned".into())
    }

    pub async fn get_latest_block(&self) -> Result<u64, Box<dyn Error + Send + Sync>> {
        Ok(self.get_tip().await?.number)
    }
}

impl<C: Client> ChainStaking for CardanoClient<C> {}

impl<C: Client> ChainPerpetual for CardanoClient<C> {}

impl<C: Client> ChainAddressStatus for CardanoClient<C> {}

impl<C: Client> ChainSimulation for CardanoClient<C> {}

impl<C: Client> ChainTraits for CardanoClient<C> {}

impl<C: Client> ChainProvider for CardanoClient<C> {
    fn get_chain(&self) -> Chain {
        self.chain
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;

    use super::*;

    #[tokio::test]
    async fn test_get_tip() {
        let client = MockClient::new().with_post(|path, body| {
            assert_eq!(path, "/");
            let request: serde_json::Value = serde_json::from_slice(body).unwrap();
            assert_eq!(request["query"], "{ cardano { tip { number slotNo } } }");
            Ok(br#"{"data":{"cardano":{"tip":{"number":13427226,"slotNo":"187400452"}}}}"#.to_vec())
        });

        let cardano = CardanoClient::new(client);
        let tip = cardano.get_tip().await.unwrap();
        assert_eq!(tip.number, 13_427_226);
        assert_eq!(tip.slot_no, 187_400_452);
    }

    #[tokio::test]
    async fn test_get_address_transactions() {
        let address = "addr1test";
        let client = MockClient::new().with_post(move |path, body| {
            assert_eq!(path, "/");
            let request: serde_json::Value = serde_json::from_slice(body).unwrap();
            assert_eq!(request["operationName"], "GetTransactionsByAddress");
            assert_eq!(request["variables"], serde_json::json!({ "address": address, "limit": 25 }));
            assert_eq!(
                request["query"],
                CardanoTarget::AddressTransactions {
                    address: address.to_string(),
                    limit: 25
                }
                .query()
            );
            Ok(br#"{"data":{"transactions":[{"hash":"tx_hash","includedAt":"2023-01-01T00:00:00Z","inputs":[{"address":"addr1","value":"1000"}],"outputs":[{"address":"addr2","value":"900"}],"fee":"100"}]}}"#.to_vec())
        });

        let transactions = CardanoClient::new(client).get_address_transactions(address, 25).await.unwrap();

        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].transaction.hash, "tx_hash");
        assert_eq!(transactions[0].included_at, "2023-01-01T00:00:00Z");
    }
}

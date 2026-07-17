use std::error::Error;

use chain_traits::{ChainAddressStatus, ChainPerpetual, ChainProvider, ChainSimulation, ChainStaking, ChainTraits};
use gem_client::{Client, ClientExt};
use primitives::chain::Chain;

use crate::models::{
    account::BalanceResponse,
    block::{Block, BlockData, GenesisData},
    rpc::{AddressTransaction, AddressTransactions, Block as RpcBlock, Blocks, Data},
    transaction::TransactionBroadcast,
    utxo::{UTXO, UTXOS},
};
use primitives::graphql::GraphqlData;

const TIP_QUERY: &str = "{ cardano { tip { number slotNo } } }";
const TRANSACTIONS_BY_ADDRESS_QUERY: &str = "query GetTransactionsByAddress($address: String!, $limit: Int!) { transactions(limit: $limit, order_by: { includedAt: desc }, where: { outputs: { address: { _eq: $address } } }) { hash includedAt inputs { address value } outputs { address value } fee } }";

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

    pub(crate) async fn get_tip(&self) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let json = serde_json::json!({
            "query": TIP_QUERY
        });
        let response: Data<BlockData> = self.client.post("/", &json).await?;
        Ok(response.data.cardano.tip)
    }

    pub async fn get_block(&self, block_number: u64) -> Result<RpcBlock, Box<dyn Error + Send + Sync>> {
        let json = serde_json::json!({
            "query": "query GetBlockByNumber($blockNumber: Int!) { blocks(where: { number: { _eq: $blockNumber } }) { number hash forgedAt transactions { hash inputs { address value } outputs { address value } fee } } }",
            "variables": {
                "blockNumber": block_number
            },
            "operationName": "GetBlockByNumber"
        });
        let response: Data<Blocks> = self.client.post("/", &json).await?;
        response.data.blocks.first().cloned().ok_or_else(|| "Block not found".into())
    }

    pub async fn get_address_transactions(&self, address: &str, limit: usize) -> Result<Vec<AddressTransaction>, Box<dyn Error + Send + Sync>> {
        let request = serde_json::json!({
            "operationName": "GetTransactionsByAddress",
            "variables": {
                "address": address,
                "limit": limit,
            },
            "query": TRANSACTIONS_BY_ADDRESS_QUERY,
        });
        let response: Data<AddressTransactions> = self.client.post("/", &request).await?;
        Ok(response.data.transactions)
    }

    pub async fn get_balance(&self, address: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let json = serde_json::json!({
            "operationName": "GetBalance",
            "variables": {"address": address},
            "query": "query GetBalance($address: String!) { utxos: utxos_aggregate(where: { address: { _eq: $address }  } ) { aggregate { sum { value } } } }"
        });
        let response: GraphqlData<BalanceResponse> = self.client.post("/", &json).await?;

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
        let json = serde_json::json!({
            "operationName": "UtxoSetForAddress",
            "variables": {"address": address},
            "query": "query UtxoSetForAddress($address: String!) { utxos(order_by: { value: desc } , where: { address: { _eq: $address }  } ) { address value txHash index tokens { quantity asset { fingerprint policyId assetName } } } }"
        });
        let response: Data<UTXOS<Vec<UTXO>>> = self.client.post("/", &json).await?;
        Ok(response.data.utxos)
    }

    pub async fn get_network_magic(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let json = serde_json::json!({
            "operationName": "GetNetworkMagic",
            "variables": {},
            "query": "query GetNetworkMagic { genesis { shelley { networkMagic } } }"
        });
        let response: Data<GenesisData> = self.client.post("/", &json).await?;
        Ok(response.data.genesis.shelley.network_magic.to_string())
    }

    pub async fn broadcast_transaction(&self, data: String) -> Result<String, Box<dyn Error + Send + Sync>> {
        let json = serde_json::json!({
            "operationName": "SubmitTransaction",
            "variables": {"transaction": data},
            "query": "mutation SubmitTransaction($transaction: String!) { submitTransaction(transaction: $transaction) { hash } }"
        });
        let response: GraphqlData<TransactionBroadcast> = self.client.post("/", &json).await?;

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
            assert_eq!(request["query"], TRANSACTIONS_BY_ADDRESS_QUERY);
            Ok(br#"{"data":{"transactions":[{"hash":"tx_hash","includedAt":"2023-01-01T00:00:00Z","inputs":[{"address":"addr1","value":"1000"}],"outputs":[{"address":"addr2","value":"900"}],"fee":"100"}]}}"#.to_vec())
        });

        let transactions = CardanoClient::new(client).get_address_transactions(address, 25).await.unwrap();

        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].transaction.hash, "tx_hash");
        assert_eq!(transactions[0].included_at, "2023-01-01T00:00:00Z");
    }
}

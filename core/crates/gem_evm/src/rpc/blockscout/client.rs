use std::cmp::Reverse;
use std::collections::BTreeSet;
use std::error::Error;

use gem_client::{Client, ClientExt, build_path_with_query};
use num_bigint::BigUint;

use super::model::{Items, TokenBalance, TokenTransfer, Transaction};
use crate::rpc::EVMIndexerClient;

#[derive(Debug, Clone)]
pub(crate) struct BlockscoutClient<C: Client + Clone> {
    client: C,
    chain_id: u64,
    api_key: String,
}

impl<C: Client + Clone> BlockscoutClient<C> {
    pub(crate) fn new(client: C, chain_id: u64, api_key: String) -> Self {
        Self { client, chain_id, api_key }
    }

    fn address_path(&self, address: &str, endpoint: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(build_path_with_query(
            &format!("/{}/api/v2/addresses/{address}/{endpoint}", self.chain_id),
            &[("apikey", self.api_key.as_str())],
        )?)
    }
}

impl<C: Client + Clone> EVMIndexerClient for BlockscoutClient<C> {
    async fn get_transaction_ids_by_address(&self, address: &str, limit: usize) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let transactions: Items<Transaction> = self.client.get(&self.address_path(address, "transactions")?).await?;
        let token_transfers: Items<TokenTransfer> = self.client.get(&self.address_path(address, "token-transfers")?).await?;
        Ok(transactions
            .items
            .into_iter()
            .map(|transaction| (transaction.block_number, transaction.hash))
            .chain(token_transfers.items.into_iter().map(|transfer| (transfer.block_number, transfer.transaction_hash)))
            .map(|(block_number, hash)| (Reverse(block_number), hash))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|(_, hash)| hash)
            .take(limit)
            .collect())
    }

    async fn get_token_balances(&self, address: &str) -> Result<Vec<(String, BigUint)>, Box<dyn Error + Send + Sync>> {
        let balances: Vec<TokenBalance> = self.client.get(&self.address_path(address, "token-balances")?).await?;
        Ok(balances
            .into_iter()
            .filter(|balance| balance.token.token_type == "ERC-20" && balance.token.reputation.as_deref() == Some("ok"))
            .map(|balance| (balance.token.address_hash, balance.value))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;

    use super::*;

    #[tokio::test]
    async fn test_get_transaction_ids_by_address() {
        let client = MockClient::new().with_get(|path| {
            let response = match path {
                "/1/api/v2/addresses/0x123/transactions?apikey=key" => include_str!("../../../testdata/blockscout_transactions.json"),
                "/1/api/v2/addresses/0x123/token-transfers?apikey=key" => include_str!("../../../testdata/blockscout_token_transfers.json"),
                _ => panic!("unexpected path: {path}"),
            };
            Ok(response.as_bytes().to_vec())
        });
        let client = BlockscoutClient::new(client, 1, "key".to_string());

        let transaction_ids = client.get_transaction_ids_by_address("0x123", 3).await.unwrap();

        assert_eq!(transaction_ids, vec!["0xtoken", "0xnormal", "0xshared"]);
    }

    #[tokio::test]
    async fn test_get_token_balances() {
        let client = MockClient::new().with_get(|path| {
            assert_eq!(path, "/1/api/v2/addresses/0x123/token-balances?apikey=key");
            Ok(include_str!("../../../testdata/blockscout_token_balances.json").as_bytes().to_vec())
        });
        let client = BlockscoutClient::new(client, 1, "key".to_string());

        let balances = client.get_token_balances("0x123").await.unwrap();

        assert_eq!(
            balances,
            vec![("0xtoken".to_string(), BigUint::from(42u8)), ("0xunpriced".to_string(), BigUint::from(10u8)),]
        );
    }
}

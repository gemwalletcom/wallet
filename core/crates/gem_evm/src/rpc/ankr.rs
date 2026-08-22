use std::collections::HashSet;
use std::error::Error;

use gem_ankr::Client as AnkrClient;
use gem_client::Client as Transport;
use num_bigint::BigUint;

use super::{EVMIndexerClient, TransactionReference};

impl<C: Transport + Clone> EVMIndexerClient for AnkrClient<C> {
    async fn get_transactions_by_address(&self, address: &str, limit: usize) -> Result<Vec<TransactionReference>, Box<dyn Error + Send + Sync>> {
        let (transactions, token_transfers) = futures::try_join!(self.get_transactions(address, limit), self.get_token_transfers(address, limit))?;
        let hashes = transactions
            .into_iter()
            .map(|transaction| transaction.hash)
            .chain(token_transfers.into_iter().map(|transfer| transfer.transaction_hash));
        let mut seen = HashSet::new();
        Ok(hashes.filter(|hash| seen.insert(hash.clone())).map(|hash| TransactionReference::new(hash, None)).collect())
    }

    async fn get_token_balances(&self, address: &str) -> Result<Vec<(String, BigUint)>, Box<dyn Error + Send + Sync>> {
        Ok(AnkrClient::get_token_balances(self, address)
            .await?
            .into_iter()
            .filter_map(|balance| balance.contract_address.map(|address| (address, balance.balance_raw_integer)))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use gem_ankr::testkit::{ACCOUNT_BALANCE, TOKEN_TRANSFERS, TRANSACTIONS};
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use serde_json::{Value, from_str};

    use super::*;

    #[tokio::test]
    async fn test_indexer_mapping() {
        let client = AnkrClient::new(
            mock_jsonrpc_client(|method, _| {
                let response: Value = match method {
                    "ankr_getTransactionsByAddress" => from_str(TRANSACTIONS).unwrap(),
                    "ankr_getTokenTransfers" => from_str(TOKEN_TRANSFERS).unwrap(),
                    "ankr_getAccountBalance" => from_str(ACCOUNT_BALANCE).unwrap(),
                    _ => panic!("unexpected method: {method}"),
                };
                Ok(response)
            }),
            "bsc",
        );

        let transactions = client.get_transactions_by_address("0x123", 2).await.unwrap();
        let balances = EVMIndexerClient::get_token_balances(&client, "0x123").await.unwrap();

        assert_eq!(
            transactions,
            vec![
                TransactionReference::new("0xcee2abf4d8cc0ea0b9ecc9d21d81b7579f614a27a8740210856b199e5521f6f7".to_string(), None),
                TransactionReference::new("0x1111111111111111111111111111111111111111111111111111111111111111".to_string(), None)
            ]
        );
        assert_eq!(
            balances,
            vec![("0x227D920e20eBAc8A40E7D6431B7d724Bb64D7245".to_string(), BigUint::from(3_371_908_000_000_000_000u64))]
        );
    }
}

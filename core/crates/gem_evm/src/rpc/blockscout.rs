use std::cmp::Reverse;
use std::collections::BTreeSet;
use std::error::Error;

use gem_blockscout::Client as BlockscoutClient;
use gem_client::Client as Transport;
use num_bigint::BigUint;

use super::{EVMIndexerClient, TransactionReference};

impl<C: Transport> EVMIndexerClient for BlockscoutClient<C> {
    async fn get_transactions_by_address(&self, address: &str, limit: usize) -> Result<Vec<TransactionReference>, Box<dyn Error + Send + Sync>> {
        let transactions = self.get_transactions(address, limit).await?;
        let token_transfers = self.get_token_transfers(address, limit).await?;
        Ok(transactions
            .into_iter()
            .map(|transaction| (transaction.block_number, transaction.hash))
            .chain(token_transfers.into_iter().map(|transfer| (transfer.block_number, transfer.transaction_hash)))
            .map(|(block_number, hash)| (Reverse(block_number), hash))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|(Reverse(block_number), hash)| TransactionReference::new(hash, Some(block_number)))
            .collect())
    }

    async fn get_token_balances(&self, address: &str) -> Result<Vec<(String, BigUint)>, Box<dyn Error + Send + Sync>> {
        Ok(BlockscoutClient::get_token_balances(self, address)
            .await?
            .into_iter()
            .filter(|balance| balance.token.token_type == "ERC-20" && balance.token.reputation.as_deref() == Some("ok"))
            .map(|balance| (balance.token.address_hash, balance.value))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use gem_blockscout::testkit::{TOKEN_BALANCES, TOKEN_TRANSFERS, TRANSACTIONS};
    use gem_client::testkit::MockClient;

    use super::*;

    #[tokio::test]
    async fn test_indexer_mapping() {
        let client = BlockscoutClient::new(
            MockClient::new().with_get(|path| {
                let response = match path {
                    "/1/api/v2/addresses/0x123/transactions" => TRANSACTIONS,
                    "/1/api/v2/addresses/0x123/token-transfers" => TOKEN_TRANSFERS,
                    "/1/api/v2/addresses/0x123/token-balances" => TOKEN_BALANCES,
                    _ => panic!("unexpected path: {path}"),
                };
                Ok(response.as_bytes().to_vec())
            }),
            1,
            "key".to_string(),
        );

        let transactions = client.get_transactions_by_address("0x123", 3).await.unwrap();
        let balances = EVMIndexerClient::get_token_balances(&client, "0x123").await.unwrap();

        assert_eq!(
            transactions,
            vec![
                TransactionReference::new("0xtoken".to_string(), Some(11)),
                TransactionReference::new("0xnormal".to_string(), Some(10)),
                TransactionReference::new("0xshared".to_string(), Some(8))
            ]
        );
        assert_eq!(
            balances,
            vec![("0xtoken".to_string(), BigUint::from(42u8)), ("0xunpriced".to_string(), BigUint::from(10u8))]
        );
    }
}

use gem_client::{Client as Transport, ClientError, ClientExt};

use crate::model::{Items, PageQuery};
use crate::target::BlockscoutTarget;
use crate::{TokenBalance, TokenTransfer, Transaction};

pub struct Client<C: Transport> {
    client: C,
    chain_id: u64,
    api_key: String,
}

impl<C: Transport> Client<C> {
    pub fn new(client: C, chain_id: u64, api_key: String) -> Self {
        Self { client, chain_id, api_key }
    }

    pub async fn get_transactions(&self, address: &str, limit: usize) -> Result<Vec<Transaction>, ClientError> {
        let target = BlockscoutTarget::Transactions {
            chain_id: self.chain_id,
            address: address.to_string(),
            query: PageQuery::newest(limit),
        };
        Ok(self.client.get::<Items<Transaction>>(target).query(&self.api_key_query()).await?.items)
    }

    pub async fn get_token_transfers(&self, address: &str, limit: usize) -> Result<Vec<TokenTransfer>, ClientError> {
        let target = BlockscoutTarget::TokenTransfers {
            chain_id: self.chain_id,
            address: address.to_string(),
            query: PageQuery::newest(limit),
        };
        Ok(self.client.get::<Items<TokenTransfer>>(target).query(&self.api_key_query()).await?.items)
    }

    pub async fn get_token_balances(&self, address: &str) -> Result<Vec<TokenBalance>, ClientError> {
        let target = BlockscoutTarget::TokenBalances {
            chain_id: self.chain_id,
            address: address.to_string(),
        };
        self.client.get(target).query(&self.api_key_query()).await
    }

    fn api_key_query(&self) -> [(&'static str, &str); 1] {
        [("apikey", self.api_key.as_str())]
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;
    use num_bigint::BigUint;

    use super::*;
    use crate::testkit::{TOKEN_BALANCES, TOKEN_TRANSFERS, TRANSACTIONS};

    #[tokio::test]
    async fn test_get_transactions() {
        let client = Client::new(
            MockClient::new().with_get(|path| {
                let response = match path {
                    "/1/api/v2/addresses/0x123/transactions?sort=block_number&order=desc&items_count=3&apikey=key" => TRANSACTIONS,
                    "/1/api/v2/addresses/0x123/token-transfers?sort=block_number&order=desc&items_count=3&apikey=key" => TOKEN_TRANSFERS,
                    _ => panic!("unexpected path: {path}"),
                };
                Ok(response.as_bytes().to_vec())
            }),
            1,
            "key".to_string(),
        );

        let transactions = client.get_transactions("0x123", 3).await.unwrap();
        let transfers = client.get_token_transfers("0x123", 3).await.unwrap();

        assert_eq!(
            transactions,
            vec![
                Transaction {
                    hash: "0xnormal".to_string(),
                    block_number: 10
                },
                Transaction {
                    hash: "0xshared".to_string(),
                    block_number: 8
                }
            ]
        );
        assert_eq!(
            transfers,
            vec![
                TokenTransfer {
                    transaction_hash: "0xtoken".to_string(),
                    block_number: 11
                },
                TokenTransfer {
                    transaction_hash: "0xshared".to_string(),
                    block_number: 8
                }
            ]
        );
    }

    #[tokio::test]
    async fn test_get_token_balances() {
        let client = Client::new(
            MockClient::new().with_get(|path| {
                assert_eq!(path, "/1/api/v2/addresses/0x123/token-balances?apikey=key");
                Ok(TOKEN_BALANCES.as_bytes().to_vec())
            }),
            1,
            "key".to_string(),
        );

        let balances = client.get_token_balances("0x123").await.unwrap();

        assert_eq!(balances.len(), 4);
        assert_eq!(balances[0].value, BigUint::from(42u8));
        assert_eq!(balances[2].token.reputation.as_deref(), Some("spam"));
        assert_eq!(balances[3].token.token_type, "ERC-721");
    }
}

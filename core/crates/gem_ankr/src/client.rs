use gem_client::Client as Transport;
use gem_jsonrpc::client::JsonRpcClient;
use gem_jsonrpc::types::JsonRpcError;

use crate::jsonrpc::AnkrRpc;
use crate::model::{TokenBalances, TokenTransfers, Transactions};
use crate::{TokenBalance, TokenTransfer, Transaction};

pub struct Client<C: Transport + Clone> {
    client: JsonRpcClient<C>,
    network: &'static str,
}

impl<C: Transport + Clone> Client<C> {
    pub fn new(client: JsonRpcClient<C>, network: &'static str) -> Self {
        Self { client, network }
    }

    pub async fn get_transactions(&self, address: &str, limit: usize) -> Result<Vec<Transaction>, JsonRpcError> {
        let response: Transactions = self
            .client
            .request(AnkrRpc::TransactionsByAddress {
                address,
                network: self.network,
                limit,
            })
            .await?;
        Ok(response.transactions)
    }

    pub async fn get_token_transfers(&self, address: &str, limit: usize) -> Result<Vec<TokenTransfer>, JsonRpcError> {
        let response: TokenTransfers = self
            .client
            .request(AnkrRpc::TokenTransfers {
                address,
                network: self.network,
                limit,
            })
            .await?;
        Ok(response.transfers)
    }

    pub async fn get_token_balances(&self, address: &str) -> Result<Vec<TokenBalance>, JsonRpcError> {
        let response: TokenBalances = self.client.request(AnkrRpc::AccountBalance { address, network: self.network }).await?;
        Ok(response.assets)
    }
}

#[cfg(test)]
mod tests {
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use num_bigint::BigUint;
    use serde_json::{from_str, json};

    use super::*;
    use crate::testkit::{ACCOUNT_BALANCE, TOKEN_TRANSFERS, TRANSACTIONS};

    #[tokio::test]
    async fn test_get_transactions() {
        let client = Client::new(
            mock_jsonrpc_client(|method, params| match method {
                "ankr_getTransactionsByAddress" => {
                    assert_eq!(params, &json!({"address": "0x123", "blockchain": "bsc", "pageSize": 2, "descOrder": true}));
                    Ok(from_str(TRANSACTIONS).unwrap())
                }
                "ankr_getTokenTransfers" => {
                    assert_eq!(params, &json!({"address": "0x123", "blockchain": "bsc", "pageSize": 2}));
                    Ok(from_str(TOKEN_TRANSFERS).unwrap())
                }
                _ => panic!("unexpected method: {method}"),
            }),
            "bsc",
        );

        let transactions = client.get_transactions("0x123", 2).await.unwrap();
        let transfers = client.get_token_transfers("0x123", 2).await.unwrap();

        assert_eq!(
            transactions,
            vec![Transaction {
                hash: "0xcee2abf4d8cc0ea0b9ecc9d21d81b7579f614a27a8740210856b199e5521f6f7".to_string()
            }]
        );
        assert_eq!(
            transfers,
            vec![
                TokenTransfer {
                    transaction_hash: "0x1111111111111111111111111111111111111111111111111111111111111111".to_string()
                },
                TokenTransfer {
                    transaction_hash: "0xcee2abf4d8cc0ea0b9ecc9d21d81b7579f614a27a8740210856b199e5521f6f7".to_string()
                }
            ]
        );
    }

    #[tokio::test]
    async fn test_get_token_balances() {
        let client = Client::new(
            mock_jsonrpc_client(|method, params| {
                assert_eq!(method, "ankr_getAccountBalance");
                assert_eq!(params, &json!([{"walletAddress": "0x123", "blockchain": "xlayer", "onlyWhitelisted": true}]));
                Ok(from_str(ACCOUNT_BALANCE).unwrap())
            }),
            "xlayer",
        );

        let balances = client.get_token_balances("0x123").await.unwrap();

        assert_eq!(
            balances,
            vec![TokenBalance {
                contract_address: Some("0x227D920e20eBAc8A40E7D6431B7d724Bb64D7245".to_string()),
                balance_raw_integer: BigUint::from(3_371_908_000_000_000_000u64),
            }]
        );
    }
}

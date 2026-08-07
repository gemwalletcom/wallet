use std::collections::HashSet;
use std::error::Error;

use crate::rpc::{EVMIndexerClient, TransactionReference};
use gem_client::Client;
use gem_jsonrpc::client::JsonRpcClient;
use num_bigint::BigUint;

use super::{
    jsonrpc::AnkrRpc,
    model::{TokenBalances, TokenTransfers, Transactions},
};

pub(crate) struct AnkrClient<C: Client + Clone> {
    chain: &'static str,
    client: JsonRpcClient<C>,
}

impl<C: Client + Clone> AnkrClient<C> {
    pub(crate) fn new(client: JsonRpcClient<C>, chain: &'static str) -> Self {
        Self { chain, client }
    }
}

impl<C: Client + Clone> EVMIndexerClient for AnkrClient<C> {
    async fn get_transactions_by_address(&self, address: &str, limit: usize) -> Result<Vec<TransactionReference>, Box<dyn Error + Send + Sync>> {
        let (transactions, token_transfers): (Transactions, TokenTransfers) = futures::try_join!(
            self.client.request(AnkrRpc::TransactionsByAddress {
                address: address.to_string(),
                chain: self.chain,
                limit,
            }),
            self.client.request(AnkrRpc::TokenTransfers {
                address: address.to_string(),
                chain: self.chain,
                limit,
            }),
        )?;

        let transaction_ids = transactions
            .transactions
            .into_iter()
            .map(|transaction| transaction.hash)
            .chain(token_transfers.transfers.into_iter().map(|transfer| transfer.transaction_hash));

        let mut seen = HashSet::new();
        Ok(transaction_ids
            .filter(|hash| seen.insert(hash.clone()))
            .map(|hash| TransactionReference::new(hash, None))
            .collect())
    }

    async fn get_token_balances(&self, address: &str) -> Result<Vec<(String, BigUint)>, Box<dyn Error + Send + Sync>> {
        let balances: TokenBalances = self
            .client
            .request(AnkrRpc::AccountBalance {
                address: address.to_string(),
                chain: self.chain,
            })
            .await?;
        Ok(balances
            .assets
            .into_iter()
            .filter_map(|asset| asset.contract_address.map(|address| (address, asset.balance_raw_integer)))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::method;
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use primitives::testkit::json::load_json;
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test_get_transaction_ids_by_address() {
        let rpc_client = mock_jsonrpc_client(|request_method, params| match request_method {
            method::ANKR_GET_TRANSACTIONS_BY_ADDRESS => {
                assert_eq!(
                    params,
                    &json!({
                        "address": "0x123",
                        "blockchain": "bsc",
                        "pageSize": 2,
                        "descOrder": true
                    })
                );
                Ok(load_json(include_str!("../../../testdata/ankr_get_transactions_by_address.json")))
            }
            method::ANKR_GET_TOKEN_TRANSFERS => {
                assert_eq!(
                    params,
                    &json!({
                        "address": "0x123",
                        "blockchain": "bsc",
                        "pageSize": 2
                    })
                );
                Ok(load_json(include_str!("../../../testdata/ankr_get_token_transfers.json")))
            }
            _ => panic!("unexpected method: {request_method}"),
        });
        let client = AnkrClient::new(rpc_client, "bsc");

        let transaction_ids = client.get_transactions_by_address("0x123", 2).await.unwrap();

        assert_eq!(
            transaction_ids,
            vec![
                TransactionReference::new("0xcee2abf4d8cc0ea0b9ecc9d21d81b7579f614a27a8740210856b199e5521f6f7".to_string(), None),
                TransactionReference::new("0x1111111111111111111111111111111111111111111111111111111111111111".to_string(), None)
            ]
        );
    }

    #[tokio::test]
    async fn test_get_token_balances() {
        let rpc_client = mock_jsonrpc_client(|request_method, params| {
            assert_eq!(request_method, method::ANKR_GET_ACCOUNT_BALANCE);
            assert_eq!(
                params,
                &json!([{
                    "walletAddress": "0x123",
                    "blockchain": "xlayer",
                    "onlyWhitelisted": true
                }])
            );
            Ok(load_json(include_str!("../../../testdata/ankr_get_account_balance.json")))
        });
        let client = AnkrClient::new(rpc_client, "xlayer");

        let balances = client.get_token_balances("0x123").await.unwrap();

        assert_eq!(
            balances,
            vec![("0x227D920e20eBAc8A40E7D6431B7d724Bb64D7245".to_string(), BigUint::from(3_371_908_000_000_000_000u64))]
        );
    }
}

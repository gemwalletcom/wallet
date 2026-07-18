use std::cmp::Reverse;
use std::collections::HashSet;
use std::error::Error;

use gem_client::Client;
use gem_jsonrpc::client::JsonRpcClient as GenericJsonRpcClient;
use num_bigint::BigUint;
use primitives::EVMChain;
use serde_json::json;

use crate::rpc::EVMIndexerClient;

use super::model::{TokenBalances, TokenTransfers, Transactions};

#[derive(Debug, Clone)]
pub(crate) struct AnkrClient<C: Client + Clone> {
    chain: &'static str,
    rpc_client: GenericJsonRpcClient<C>,
}

impl<C: Client + Clone> AnkrClient<C> {
    pub(crate) fn new(client: GenericJsonRpcClient<C>, chain: EVMChain) -> Option<Self> {
        Some(Self {
            chain: Self::chain_name(chain)?,
            rpc_client: client,
        })
    }

    fn chain_name(chain: EVMChain) -> Option<&'static str> {
        match chain {
            EVMChain::Ethereum => Some("eth"),
            EVMChain::Polygon => Some("polygon"),
            EVMChain::AvalancheC => Some("avalanche"),
            EVMChain::SmartChain => Some("bsc"),
            EVMChain::Arbitrum => Some("arbitrum"),
            EVMChain::Optimism => Some("optimism"),
            EVMChain::Base => Some("base"),
            EVMChain::Fantom => Some("fantom"),
            EVMChain::Gnosis => Some("gnosis"),
            EVMChain::Linea => Some("linea"),
            EVMChain::XLayer => Some("xlayer"),
            EVMChain::OpBNB
            | EVMChain::Manta
            | EVMChain::Blast
            | EVMChain::ZkSync
            | EVMChain::Mantle
            | EVMChain::Celo
            | EVMChain::World
            | EVMChain::Sonic
            | EVMChain::SeiEvm
            | EVMChain::Abstract
            | EVMChain::Berachain
            | EVMChain::Ink
            | EVMChain::Unichain
            | EVMChain::Hyperliquid
            | EVMChain::Plasma
            | EVMChain::Monad
            | EVMChain::Robinhood
            | EVMChain::Stable => None,
        }
    }
}

impl<C: Client + Clone> EVMIndexerClient for AnkrClient<C> {
    async fn get_transaction_ids_by_address(&self, address: &str, limit: usize) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let transactions: Transactions = self
            .rpc_client
            .call(
                "ankr_getTransactionsByAddress",
                json!({
                    "address": address,
                    "blockchain": self.chain,
                    "pageSize": limit,
                    "descOrder": true
                }),
            )
            .await?;
        let token_transfers: TokenTransfers = self
            .rpc_client
            .call(
                "ankr_getTokenTransfers",
                json!({
                    "address": address,
                    "blockchain": self.chain,
                    "pageSize": limit
                }),
            )
            .await?;

        let mut transactions = transactions
            .transactions
            .into_iter()
            .map(|transaction| (transaction.hash, transaction.timestamp))
            .chain(token_transfers.transfers.into_iter().map(|transfer| (transfer.transaction_hash, transfer.timestamp)))
            .collect::<Vec<_>>();
        transactions.sort_by_key(|(_, timestamp)| Reverse(*timestamp));

        let mut seen = HashSet::new();
        Ok(transactions
            .into_iter()
            .filter_map(|(hash, _)| seen.insert(hash.clone()).then_some(hash))
            .take(limit)
            .collect())
    }

    async fn get_token_balances(&self, address: &str) -> Result<Vec<(String, BigUint)>, Box<dyn Error + Send + Sync>> {
        let params = json!([{
            "walletAddress": address,
            "blockchain": self.chain,
            "onlyWhitelisted": true,
        }]);
        let balances: TokenBalances = self.rpc_client.call("ankr_getAccountBalance", params).await?;
        Ok(balances
            .assets
            .into_iter()
            .filter_map(|asset| asset.contract_address.map(|address| (address, asset.balance_raw_integer)))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use primitives::testkit::json::load_json;

    use super::*;

    #[tokio::test]
    async fn test_get_transaction_ids_by_address() {
        let rpc_client = mock_jsonrpc_client(|method, params| match method {
            "ankr_getTransactionsByAddress" => {
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
            "ankr_getTokenTransfers" => {
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
            _ => panic!("unexpected method: {method}"),
        });
        let client = AnkrClient::new(rpc_client, EVMChain::SmartChain).unwrap();

        let transaction_ids = client.get_transaction_ids_by_address("0x123", 2).await.unwrap();

        assert_eq!(
            transaction_ids,
            vec![
                "0x1111111111111111111111111111111111111111111111111111111111111111",
                "0xcee2abf4d8cc0ea0b9ecc9d21d81b7579f614a27a8740210856b199e5521f6f7"
            ]
        );
    }

    #[tokio::test]
    async fn test_get_token_balances() {
        let rpc_client = mock_jsonrpc_client(|method, params| {
            assert_eq!(method, "ankr_getAccountBalance");
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
        let client = AnkrClient::new(rpc_client, EVMChain::XLayer).unwrap();

        let balances = client.get_token_balances("0x123").await.unwrap();

        assert_eq!(
            balances,
            vec![("0x227D920e20eBAc8A40E7D6431B7d724Bb64D7245".to_string(), BigUint::from(3_371_908_000_000_000_000u64))]
        );
    }
}

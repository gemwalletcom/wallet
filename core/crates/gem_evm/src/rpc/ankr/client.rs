use std::error::Error;

use gem_client::Client;
use gem_jsonrpc::client::JsonRpcClient as GenericJsonRpcClient;
use primitives::EVMChain;
use serde_json::json;

use crate::rpc::ankr::model::{TokenBalances, Transactions, ankr_chain};

#[derive(Debug, Clone)]
pub struct AnkrClient<C: Client + Clone> {
    pub chain: EVMChain,
    rpc_client: GenericJsonRpcClient<C>,
}

impl<C: Client + Clone> AnkrClient<C> {
    pub fn new(client: GenericJsonRpcClient<C>, chain: EVMChain) -> Self {
        Self { chain, rpc_client: client }
    }
}

impl<C: Client + Clone> AnkrClient<C> {
    /// Reference: https://www.ankr.com/docs/advanced-api/query-methods/#ankr_gettransactionsbyaddress
    pub async fn get_ankr_transactions_by_address(&self, address: &str) -> Result<Transactions, Box<dyn Error + Send + Sync>> {
        if let Some(chain) = ankr_chain(self.chain) {
            let params = serde_json::json!({
                "address": address,
                "blockchain": chain,
                "descOrder": true
            });
            Ok(self.rpc_client.call("ankr_getTransactionsByAddress", params).await?)
        } else {
            Ok(Transactions { transactions: vec![] })
        }
    }

    /// Reference: https://www.ankr.com/docs/advanced-api/token-methods/#ankr_getaccountbalance
    pub async fn get_token_balances(&self, address: &str) -> Result<TokenBalances, Box<dyn Error + Send + Sync>> {
        if let Some(chain) = ankr_chain(self.chain) {
            let params = json!([
                {
                    "walletAddress": address,
                    "blockchain": chain,
                    "onlyWhitelisted": true,
                }
            ]);

            Ok(self.rpc_client.call("ankr_getAccountBalance", params).await?)
        } else {
            Ok(TokenBalances { assets: vec![] })
        }
    }
}

#[cfg(test)]
mod tests {
    use gem_jsonrpc::testkit::mock_jsonrpc_client;

    use super::*;

    #[tokio::test]
    async fn test_get_ankr_transactions_by_address_request() {
        let rpc_client = mock_jsonrpc_client(|method, params| {
            assert_eq!(method, "ankr_getTransactionsByAddress");
            assert_eq!(
                params,
                &json!({
                    "address": "0x123",
                    "blockchain": "xlayer",
                    "descOrder": true
                })
            );
            Ok(json!({ "transactions": [] }))
        });
        let client = AnkrClient::new(rpc_client, EVMChain::XLayer);

        let transactions = client.get_ankr_transactions_by_address("0x123").await.unwrap();

        assert_eq!(transactions.transactions.len(), 0);
    }
}

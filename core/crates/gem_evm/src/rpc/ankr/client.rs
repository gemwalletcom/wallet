use std::error::Error;

use gem_client::Client;
use gem_jsonrpc::client::JsonRpcClient as GenericJsonRpcClient;
use primitives::EVMChain;
use serde_json::json;

use crate::rpc::ankr::model::{TokenBalances, Transactions, ankr_chain};

#[derive(Debug, Clone)]
pub struct AnkrClient<C: Client + Clone> {
    chain: EVMChain,
    rpc_client: GenericJsonRpcClient<C>,
}

impl<C: Client + Clone> AnkrClient<C> {
    pub fn new(client: GenericJsonRpcClient<C>, chain: EVMChain) -> Self {
        Self { chain, rpc_client: client }
    }

    /// Reference: https://www.ankr.com/docs/advanced-api/query-methods/#ankr_gettransactionsbyaddress
    pub async fn get_ankr_transactions_by_address(&self, address: &str, limit: usize) -> Result<Transactions, Box<dyn Error + Send + Sync>> {
        let Some(chain) = ankr_chain(self.chain) else {
            return Ok(Transactions { transactions: vec![] });
        };
        let params = json!({
            "address": address,
            "blockchain": chain,
            "pageSize": limit,
            "descOrder": true
        });
        Ok(self.rpc_client.call("ankr_getTransactionsByAddress", params).await?)
    }

    /// Reference: https://www.ankr.com/docs/advanced-api/token-methods/#ankr_getaccountbalance
    pub async fn get_token_balances(&self, address: &str) -> Result<TokenBalances, Box<dyn Error + Send + Sync>> {
        let Some(chain) = ankr_chain(self.chain) else {
            return Ok(TokenBalances { assets: vec![] });
        };
        let params = json!([
            {
                "walletAddress": address,
                "blockchain": chain,
                "onlyWhitelisted": true,
            }
        ]);

        Ok(self.rpc_client.call("ankr_getAccountBalance", params).await?)
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
                    "pageSize": 25,
                    "descOrder": true
                })
            );
            Ok(json!({ "transactions": [] }))
        });
        let client = AnkrClient::new(rpc_client, EVMChain::XLayer);

        let transactions = client.get_ankr_transactions_by_address("0x123", 25).await.unwrap();

        assert_eq!(transactions.transactions.len(), 0);
    }
}

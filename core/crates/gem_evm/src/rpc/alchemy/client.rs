use std::collections::HashSet;
use std::error::Error;

use gem_client::Client;
use gem_jsonrpc::client::JsonRpcClient as GenericJsonRpcClient;
use num_bigint::BigUint;
use primitives::EVMChain;
use serde_json::{Value, json};

use super::model::{TokenBalances, Transfer, Transfers};
use crate::rpc::EVMIndexerClient;

pub fn alchemy_url(chain: EVMChain, key: &str) -> Option<String> {
    let network = match chain {
        EVMChain::Blast => "blast-mainnet",
        EVMChain::ZkSync => "zksync-mainnet",
        EVMChain::Celo => "celo-mainnet",
        EVMChain::World => "worldchain-mainnet",
        EVMChain::Abstract => "abstract-mainnet",
        EVMChain::Berachain => "berachain-mainnet",
        EVMChain::Ink => "ink-mainnet",
        EVMChain::Unichain => "unichain-mainnet",
        EVMChain::Hyperliquid => "hyperliquid-mainnet",
        EVMChain::Monad => "monad-mainnet",
        EVMChain::Robinhood => "robinhood-mainnet",
        _ => return None,
    };

    Some(format!("https://{network}.g.alchemy.com/v2/{key}"))
}

#[derive(Debug, Clone)]
pub(crate) struct AlchemyClient<C: Client + Clone> {
    rpc_client: GenericJsonRpcClient<C>,
}

impl<C: Client + Clone> AlchemyClient<C> {
    pub(crate) fn new(client: GenericJsonRpcClient<C>) -> Self {
        Self { rpc_client: client }
    }

    async fn get_asset_transfers(&self, address_field: &str, address: &str, limit: usize) -> Result<Vec<Transfer>, Box<dyn Error + Send + Sync>> {
        let mut request = json!({
            "category": ["external", "erc20", "erc721", "erc1155"],
            "excludeZeroValue": false,
            "maxCount": format!("0x{limit:x}"),
            "order": "desc"
        });
        request[address_field] = Value::String(address.to_string());
        let response: Transfers = self.rpc_client.call("alchemy_getAssetTransfers", json!([request])).await?;
        Ok(response.transfers)
    }
}

impl<C: Client + Clone> EVMIndexerClient for AlchemyClient<C> {
    async fn get_transaction_ids_by_address(&self, address: &str, limit: usize) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let outgoing = self.get_asset_transfers("fromAddress", address, limit).await?;
        let incoming = self.get_asset_transfers("toAddress", address, limit).await?;
        let mut transfers = outgoing.into_iter().chain(incoming).collect::<Vec<_>>();
        transfers.sort_by_key(|transfer| std::cmp::Reverse(transfer.block_num));

        let mut transaction_ids = HashSet::new();
        Ok(transfers
            .into_iter()
            .filter_map(|transfer| transaction_ids.insert(transfer.hash.clone()).then_some(transfer.hash))
            .take(limit)
            .collect())
    }

    async fn get_token_balances(&self, address: &str) -> Result<Vec<(String, BigUint)>, Box<dyn Error + Send + Sync>> {
        let balances: TokenBalances = self.rpc_client.call("alchemy_getTokenBalances", json!([address, "erc20"])).await?;
        Ok(balances
            .token_balances
            .into_iter()
            .filter_map(|balance| balance.token_balance.map(|value| (balance.contract_address, value)))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use primitives::testkit::json::load_json;

    use super::*;

    #[test]
    fn test_alchemy_url() {
        assert_eq!(alchemy_url(EVMChain::Robinhood, "key"), Some("https://robinhood-mainnet.g.alchemy.com/v2/key".to_string()));
        assert_eq!(alchemy_url(EVMChain::Ethereum, "key"), None);
    }

    #[tokio::test]
    async fn test_get_transaction_ids_by_address() {
        let rpc_client = mock_jsonrpc_client(|method, params| {
            assert_eq!(method, "alchemy_getAssetTransfers");
            let request = &params[0];
            assert_eq!(request["category"], json!(["external", "erc20", "erc721", "erc1155"]));
            assert_eq!(request["maxCount"], "0x2");
            assert_eq!(request["order"], "desc");

            let field = if request.get("fromAddress").is_some() { "fromAddress" } else { "toAddress" };
            assert_eq!(request[field], "0x123");
            Ok(match field {
                "fromAddress" => load_json(include_str!("../../../testdata/alchemy_get_asset_transfers_from.json")),
                _ => load_json(include_str!("../../../testdata/alchemy_get_asset_transfers_to.json")),
            })
        });
        let client = AlchemyClient::new(rpc_client);

        let transaction_ids = client.get_transaction_ids_by_address("0x123", 2).await.unwrap();

        assert_eq!(transaction_ids, vec!["0xin", "0xout"]);
    }

    #[tokio::test]
    async fn test_get_token_balances() {
        let rpc_client = mock_jsonrpc_client(|method, params| {
            assert_eq!(method, "alchemy_getTokenBalances");
            assert_eq!(params, &json!(["0x123", "erc20"]));
            Ok(load_json(include_str!("../../../testdata/alchemy_get_token_balances.json")))
        });
        let client = AlchemyClient::new(rpc_client);

        let balances = client.get_token_balances("0x123").await.unwrap();

        assert_eq!(balances, vec![("0xtoken".to_string(), BigUint::from(42u8))]);
    }
}

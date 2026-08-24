use std::error::Error;

use gem_client::Client as Transport;
use gem_jsonrpc::client::JsonRpcClient;
use num_bigint::BigUint;
use serde_json::{Value, json};

use super::jsonrpc::AlchemyRpc;
use super::model::{TokenBalances, Transfer, Transfers};

pub struct Client<C: Transport + Clone> {
    client: JsonRpcClient<C>,
}

impl<C: Transport + Clone> Client<C> {
    pub fn new(client: JsonRpcClient<C>) -> Self {
        Self { client }
    }

    pub async fn get_asset_transfers(&self, address_field: &str, address: &str, limit: usize) -> Result<Vec<Transfer>, Box<dyn Error + Send + Sync>> {
        let mut request = json!({
            "category": ["external", "erc20", "erc721", "erc1155"],
            "excludeZeroValue": false,
            "maxCount": format!("0x{limit:x}"),
            "order": "desc"
        });
        request[address_field] = Value::String(address.to_string());
        let response: Transfers = self.client.request(AlchemyRpc::GetAssetTransfers(request)).await?;
        Ok(response.transfers)
    }

    pub async fn get_token_balances(&self, address: &str) -> Result<Vec<(String, BigUint)>, Box<dyn Error + Send + Sync>> {
        let balances: TokenBalances = self.client.request(AlchemyRpc::GetTokenBalances(address.to_string())).await?;
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

    use super::*;

    #[tokio::test]
    async fn test_get_asset_transfers() {
        let client = Client::new(mock_jsonrpc_client(|method, params| {
            assert_eq!(method, "alchemy_getAssetTransfers");
            assert_eq!(params[0]["fromAddress"], "0x123");
            assert_eq!(params[0]["maxCount"], "0x2");
            Ok(json!({"transfers": [{"blockNum": "0x2", "hash": "0xout"}]}))
        }));

        let transfers = client.get_asset_transfers("fromAddress", "0x123", 2).await.unwrap();

        assert_eq!(
            transfers,
            vec![Transfer {
                block_num: 2,
                hash: "0xout".to_string()
            }]
        );
    }

    #[tokio::test]
    async fn test_get_token_balances() {
        let client = Client::new(mock_jsonrpc_client(|method, params| {
            assert_eq!(method, "alchemy_getTokenBalances");
            assert_eq!(params, &json!(["0x123", "erc20"]));
            Ok(json!({
                "tokenBalances": [
                    {"contractAddress": "0xtoken", "tokenBalance": "0x2a"},
                    {"contractAddress": "0xerror", "tokenBalance": null}
                ]
            }))
        }));

        let balances = client.get_token_balances("0x123").await.unwrap();

        assert_eq!(balances, vec![("0xtoken".to_string(), BigUint::from(42u8))]);
    }
}

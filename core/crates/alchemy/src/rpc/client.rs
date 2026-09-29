use std::error::Error;

use gem_client::Client as Transport;
use gem_jsonrpc::client::JsonRpcClient;
use num_bigint::BigUint;

use super::jsonrpc::{AlchemyRpc, TransferDirection};
use super::model::{TokenBalances, Transfer, TransferCategory, Transfers};

pub struct Client<C: Transport + Clone> {
    client: JsonRpcClient<C>,
}

impl<C: Transport + Clone> Client<C> {
    pub fn new(client: JsonRpcClient<C>) -> Self {
        Self { client }
    }

    pub async fn get_asset_transfers(&self, direction: TransferDirection, address: &str, limit: usize) -> Result<Vec<Transfer>, Box<dyn Error + Send + Sync>> {
        let request = AlchemyRpc::AssetTransfers {
            direction,
            address: address.to_string(),
            limit,
        };
        Ok(self.client.request::<Transfers, _>(request).await?.transfers)
    }

    pub async fn get_transfers_to(&self, address: &str, categories: &[TransferCategory], from_block: u64, page_key: Option<String>, limit: usize) -> Result<Transfers, Box<dyn Error + Send + Sync>> {
        let request = AlchemyRpc::TransfersTo {
            address: address.to_string(),
            categories: categories.to_vec(),
            from_block,
            page_key,
            limit,
        };
        Ok(self.client.request::<Transfers, _>(request).await?)
    }

    pub async fn get_token_balances(&self, address: &str) -> Result<Vec<(String, BigUint)>, Box<dyn Error + Send + Sync>> {
        let balances: TokenBalances = self.client.request(AlchemyRpc::TokenBalances { address: address.to_string() }).await?;
        Ok(balances.token_balances.into_iter().filter_map(|balance| balance.token_balance.map(|value| (balance.contract_address, value))).collect())
    }
}

#[cfg(test)]
mod tests {
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use serde_json::json;

    use super::*;
    use crate::rpc::RawContract;

    #[tokio::test]
    async fn test_get_asset_transfers() {
        let client = Client::new(mock_jsonrpc_client(|method, params| {
            assert_eq!(method, "alchemy_getAssetTransfers");
            assert_eq!(params[0]["fromAddress"], "0x123");
            assert_eq!(params[0]["maxCount"], "0x2");
            Ok(json!({"transfers": [{"blockNum": "0x2", "hash": "0xout", "from": "0x123", "category": "erc20", "rawContract": {"value": "0x2a", "address": "0xtoken"}}]}))
        }));

        let transfers = client.get_asset_transfers(TransferDirection::From, "0x123", 2).await.unwrap();

        assert_eq!(
            transfers,
            vec![Transfer {
                block_num: 2,
                hash: "0xout".to_string(),
                from: "0x123".to_string(),
                category: TransferCategory::Erc20,
                raw_contract: RawContract {
                    value: Some(BigUint::from(42u8)),
                    address: Some("0xtoken".to_string()),
                },
            }]
        );
    }

    #[tokio::test]
    async fn test_get_transfers_to() {
        let client = Client::new(mock_jsonrpc_client(|method, params| {
            assert_eq!(method, "alchemy_getAssetTransfers");
            assert_eq!(
                params[0],
                json!({
                    "category": ["external", "internal", "erc20"],
                    "excludeZeroValue": true,
                    "fromBlock": "0x10",
                    "maxCount": "0x64",
                    "order": "asc",
                    "pageKey": "page",
                    "toAddress": "0x123",
                })
            );
            Ok(json!({"transfers": [{"blockNum": "0x11", "hash": "0xin", "from": "0xrouter", "category": "internal", "rawContract": {"value": "0x2a", "address": null}}], "pageKey": "next"}))
        }));

        let page = client
            .get_transfers_to("0x123", &[TransferCategory::External, TransferCategory::Internal, TransferCategory::Erc20], 16, Some("page".to_string()), 100)
            .await
            .unwrap();

        assert_eq!(page.page_key.as_deref(), Some("next"));
        assert_eq!(page.transfers[0].category, TransferCategory::Internal);
        assert_eq!(page.transfers[0].raw_contract.address, None);
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

use std::collections::HashSet;
use std::error::Error;

use gem_alchemy::rpc::Client as AlchemyClient;
use gem_client::Client;
use num_bigint::BigUint;

use super::{EVMIndexerClient, TransactionReference};

impl<C: Client + Clone> EVMIndexerClient for AlchemyClient<C> {
    async fn get_transactions_by_address(&self, address: &str, limit: usize) -> Result<Vec<TransactionReference>, Box<dyn Error + Send + Sync>> {
        let (outgoing, incoming) = futures::try_join!(
            self.get_asset_transfers("fromAddress", address, limit),
            self.get_asset_transfers("toAddress", address, limit),
        )?;
        let mut transfers = outgoing.into_iter().chain(incoming).collect::<Vec<_>>();
        transfers.sort_by_key(|transfer| std::cmp::Reverse(transfer.block_num));

        let mut transaction_ids = HashSet::new();
        Ok(transfers
            .into_iter()
            .filter_map(|transfer| {
                transaction_ids
                    .insert(transfer.hash.clone())
                    .then_some(TransactionReference::new(transfer.hash, Some(transfer.block_num)))
            })
            .take(limit)
            .collect())
    }

    async fn get_token_balances(&self, address: &str) -> Result<Vec<(String, BigUint)>, Box<dyn Error + Send + Sync>> {
        AlchemyClient::get_token_balances(self, address).await
    }
}

#[cfg(test)]
mod tests {
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use primitives::testkit::json::load_json;
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test_get_transaction_ids_by_address() {
        let client = AlchemyClient::new(mock_jsonrpc_client(|method, params| {
            assert_eq!(method, "alchemy_getAssetTransfers");
            let request = &params[0];
            assert_eq!(request["category"], json!(["external", "erc20", "erc721", "erc1155"]));
            assert_eq!(request["maxCount"], "0x2");
            assert_eq!(request["order"], "desc");

            let field = if request.get("fromAddress").is_some() { "fromAddress" } else { "toAddress" };
            assert_eq!(request[field], "0x123");
            Ok(match field {
                "fromAddress" => load_json(include_str!("../../testdata/alchemy_get_asset_transfers_from.json")),
                _ => load_json(include_str!("../../testdata/alchemy_get_asset_transfers_to.json")),
            })
        }));

        let transaction_ids = client.get_transactions_by_address("0x123", 2).await.unwrap();

        assert_eq!(
            transaction_ids,
            vec![
                TransactionReference::new("0xin".to_string(), Some(3)),
                TransactionReference::new("0xout".to_string(), Some(2))
            ]
        );
    }
}

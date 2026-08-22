use std::collections::HashSet;
use std::error::Error;

use gem_client::Client;
use gem_jsonrpc::client::JsonRpcClient;
use num_bigint::BigUint;
use primitives::Chain;
use serde_json::{Value, json};

use super::{
    jsonrpc::AlchemyRpc,
    model::{TokenBalances, Transfer, Transfers},
};
use crate::rpc::{EVMIndexerClient, TransactionReference};

pub fn alchemy_url(chain: Chain, base_url: &str, key: &str) -> String {
    let network = match chain {
        Chain::Ethereum => "eth-mainnet",
        Chain::SmartChain => "bnb-mainnet",
        Chain::Solana => "solana-mainnet",
        Chain::Polygon => "polygon-mainnet",
        Chain::Plasma => "plasma-mainnet",
        Chain::Arbitrum => "arb-mainnet",
        Chain::Optimism => "opt-mainnet",
        Chain::Base => "base-mainnet",
        Chain::AvalancheC => "avax-mainnet",
        Chain::OpBNB => "opbnb-mainnet",
        Chain::Gnosis => "gnosis-mainnet",
        Chain::Blast => "blast-mainnet",
        Chain::ZkSync => "zksync-mainnet",
        Chain::Linea => "linea-mainnet",
        Chain::Mantle => "mantle-mainnet",
        Chain::Celo => "celo-mainnet",
        Chain::World => "worldchain-mainnet",
        Chain::Sonic => "sonic-mainnet",
        Chain::SeiEvm => "sei-mainnet",
        Chain::Abstract => "abstract-mainnet",
        Chain::Berachain => "berachain-mainnet",
        Chain::Ink => "ink-mainnet",
        Chain::Unichain => "unichain-mainnet",
        Chain::Hyperliquid => "hyperliquid-mainnet",
        Chain::Monad => "monad-mainnet",
        Chain::XLayer => "xlayer-mainnet",
        Chain::Robinhood => "robinhood-mainnet",
        Chain::Stable => "stable-mainnet",
        Chain::Fantom => "fantom-mainnet",
        Chain::Manta => "manta-mainnet",
        _ => panic!("Alchemy is not supported for {chain}"),
    };

    format!("{}/v2/{key}", base_url.replace("{chain}", chain.as_ref()).replace("{network}", network))
}

pub(crate) struct AlchemyClient<C: Client + Clone> {
    client: JsonRpcClient<C>,
}

impl<C: Client + Clone> AlchemyClient<C> {
    pub(crate) fn new(client: JsonRpcClient<C>) -> Self {
        Self { client }
    }

    async fn get_asset_transfers(&self, address_field: &str, address: &str, limit: usize) -> Result<Vec<Transfer>, Box<dyn Error + Send + Sync>> {
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
}

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
    use crate::method;
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use primitives::testkit::json::load_json;

    use super::*;

    #[test]
    fn test_alchemy_url() {
        assert_eq!(alchemy_url(Chain::Solana, "http://egress/alchemy_{chain}", ""), "http://egress/alchemy_solana/v2/");
        assert_eq!(
            alchemy_url(Chain::Solana, "https://{network}.g.alchemy.com", "key"),
            "https://solana-mainnet.g.alchemy.com/v2/key"
        );
    }

    #[tokio::test]
    async fn test_get_transaction_ids_by_address() {
        let rpc_client = mock_jsonrpc_client(|request_method, params| {
            assert_eq!(request_method, method::ALCHEMY_GET_ASSET_TRANSFERS);
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

        let transaction_ids = client.get_transactions_by_address("0x123", 2).await.unwrap();

        assert_eq!(
            transaction_ids,
            vec![
                TransactionReference::new("0xin".to_string(), Some(3)),
                TransactionReference::new("0xout".to_string(), Some(2))
            ]
        );
    }

    #[tokio::test]
    async fn test_get_token_balances() {
        let rpc_client = mock_jsonrpc_client(|request_method, params| {
            assert_eq!(request_method, method::ALCHEMY_GET_TOKEN_BALANCES);
            assert_eq!(params, &json!(["0x123", "erc20"]));
            Ok(load_json(include_str!("../../../testdata/alchemy_get_token_balances.json")))
        });
        let client = AlchemyClient::new(rpc_client);

        let balances = client.get_token_balances("0x123").await.unwrap();

        assert_eq!(balances, vec![("0xtoken".to_string(), BigUint::from(42u8))]);
    }
}

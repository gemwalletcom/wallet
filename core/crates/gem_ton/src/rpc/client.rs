use std::{collections::HashMap, error::Error};

use primitives::{Asset, AssetId, AssetType, chain::Chain};
use serde::Serialize;

use chain_traits::{ChainAccount, ChainAddressStatus, ChainPerpetual, ChainStaking, ChainTraits};
use gem_client::{Client, ClientExt, build_path_with_query};

use crate::models::{
    ApiResult, BroadcastTransaction, Chainhead, DnsRecordsResponse, JettonMastersResponse, JettonWalletsResponse, NftCollectionsResponse, NftItemsResponse, RunGetMethodRequest,
    RunGetMethodResult, StackArg, TraceByAddressQuery, TraceByBlockQuery, TraceByMessageQuery, TraceByTransactionQuery, TraceResponse, WalletInfo,
    simulation::{TonEmulationRequest, TonEmulationResponse},
};

const TONCENTER_V3_BLOCK_LIMIT: usize = 100;
const TONCENTER_SORT_DESC: &str = "desc";
const TONCENTER_SORT_ASC: &str = "asc";
const TONCENTER_ACTIONS_VERSION: &str = "5";

#[derive(Debug)]
pub struct TonClient<C: Client> {
    pub client: C,
}

impl<C: Client> TonClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_master_head(&self) -> Result<Chainhead, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/api/v3/masterchainInfo").await?)
    }

    pub async fn get_dns_records(&self, domain: &str) -> Result<DnsRecordsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v3/dns/records?domain={domain}&limit=1")).await?)
    }

    pub async fn get_token_info(&self, token_id: &str) -> Result<JettonMastersResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v3/jetton/masters?address={}", token_id)).await?)
    }

    pub async fn get_balance(&self, address: String) -> Result<String, Box<dyn Error + Send + Sync>> {
        let response: ApiResult<String> = self.client.get(&format!("/api/v2/getAddressBalance?address={}", address)).await?;
        Ok(response.result)
    }

    pub async fn get_wallet_information(&self, address: String) -> Result<WalletInfo, Box<dyn Error + Send + Sync>> {
        let response: ApiResult<WalletInfo> = self.client.get(&format!("/api/v2/getWalletInformation?address={}", address)).await?;
        Ok(response.result)
    }

    pub async fn broadcast_transaction(&self, data: String) -> Result<ApiResult<BroadcastTransaction>, Box<dyn Error + Send + Sync>> {
        let body = serde_json::json!({ "boc": data });
        Ok(self.client.post("/api/v2/sendBocReturnHash", &body).await?)
    }

    pub(crate) async fn emulate_ton_connect(&self, request: &TonEmulationRequest<'_>) -> Result<TonEmulationResponse, Box<dyn Error + Send + Sync>> {
        let headers = HashMap::from([("X-Actions-Version".to_string(), TONCENTER_ACTIONS_VERSION.to_string())]);
        Ok(self.client.post_with_headers("/api/emulate/v1/emulateTonConnect", request, headers).await?)
    }

    pub async fn run_get_method(&self, address: &str, method: &str, stack: Vec<StackArg>) -> Result<RunGetMethodResult, Box<dyn Error + Send + Sync>> {
        self.run_get_method_with_headers(address, method, stack, HashMap::new()).await
    }

    pub async fn run_get_method_with_headers(
        &self,
        address: &str,
        method: &str,
        stack: Vec<StackArg>,
        headers: HashMap<String, String>,
    ) -> Result<RunGetMethodResult, Box<dyn Error + Send + Sync>> {
        let request = RunGetMethodRequest {
            address: address.to_string(),
            method: method.to_string(),
            stack,
        };
        let response: ApiResult<serde_json::Value> = self.client.post_with_headers("/api/v2/runGetMethod", &request, headers).await?;
        if !response.ok {
            let message = match response.result.as_str() {
                Some(message) => message.to_string(),
                None => response.result.to_string(),
            };
            return Err(format!("TON runGetMethod failed: {message}").into());
        }
        Ok(serde_json::from_value(response.result)?)
    }

    pub async fn get_traces_by_message(&self, hash: String) -> Result<TraceResponse, Box<dyn Error + Send + Sync>> {
        let query = TraceByMessageQuery {
            msg_hash: hash,
            include_actions: true,
        };
        self.get_traces(query).await
    }

    pub async fn get_traces_by_transaction(&self, hash: String) -> Result<TraceResponse, Box<dyn Error + Send + Sync>> {
        let query = TraceByTransactionQuery {
            tx_hash: hash,
            include_actions: true,
        };
        self.get_traces(query).await
    }

    pub async fn get_traces_by_hash(&self, hash: String) -> Result<TraceResponse, Box<dyn Error + Send + Sync>> {
        let traces = self.get_traces_by_message(hash.clone()).await?;
        if traces.traces.is_empty() {
            self.get_traces_by_transaction(hash).await
        } else {
            Ok(traces)
        }
    }

    pub async fn get_traces_by_masterchain_block(&self, block: u64) -> Result<TraceResponse, Box<dyn Error + Send + Sync>> {
        let query = TraceByBlockQuery {
            mc_seqno: block,
            include_actions: true,
            limit: TONCENTER_V3_BLOCK_LIMIT,
            offset: 0,
            sort: TONCENTER_SORT_ASC,
        };
        self.get_traces(query).await
    }

    pub async fn get_traces_by_address(&self, address: String, limit: usize) -> Result<TraceResponse, Box<dyn Error + Send + Sync>> {
        let query = TraceByAddressQuery {
            account: address,
            include_actions: true,
            limit,
            offset: 0,
            sort: TONCENTER_SORT_DESC,
        };
        self.get_traces(query).await
    }

    async fn get_traces<T: Serialize>(&self, query: T) -> Result<TraceResponse, Box<dyn Error + Send + Sync>> {
        let path = build_path_with_query("/api/v3/traces", &query);
        Ok(self.client.get(&path).await?)
    }

    pub async fn get_jetton_wallets(&self, address: String) -> Result<JettonWalletsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v3/jetton/wallets?owner_address={}&limit=100&offset=0", address)).await?)
    }

    pub async fn get_nft_items_by_owner(&self, owner_address: &str) -> Result<NftItemsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v3/nft/items?owner_address={}&limit=1000&offset=0", owner_address)).await?)
    }

    pub async fn get_nft_item(&self, address: &str) -> Result<NftItemsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v3/nft/items?address={}", address)).await?)
    }

    pub async fn get_nft_collection(&self, collection_address: &str) -> Result<NftCollectionsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v3/nft/collections?collection_address={}", collection_address)).await?)
    }

    pub async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let response = self.get_token_info(&token_id).await?;
        let master = response.jetton_masters.first().ok_or("missing jetton master")?;
        let indexed_info = response
            .metadata
            .get(&master.address)
            .and_then(|metadata| metadata.token_info.iter().find(|info| info.valid));
        let inline_metadata = master.jetton_content.name.as_ref().zip(master.jetton_content.symbol.as_ref());
        let indexed_metadata = indexed_info.and_then(|info| info.name.as_ref().zip(info.symbol.as_ref()));
        let (name, symbol) = inline_metadata.or(indexed_metadata).ok_or("invalid jetton metadata")?;
        let decimals = master
            .jetton_content
            .decimals
            .or_else(|| indexed_info.and_then(|info| info.extra.as_ref()?.decimals))
            .unwrap_or(9);
        let decimals = i32::from(u8::try_from(decimals).map_err(|_| "invalid jetton decimals")?);

        Ok(Asset::new(
            AssetId::from_token(Chain::Ton, &token_id),
            name.clone(),
            symbol.clone(),
            decimals,
            AssetType::JETTON,
        ))
    }
}

impl<C: Client> ChainTraits for TonClient<C> {}
impl<C: Client> ChainAccount for TonClient<C> {}
impl<C: Client> ChainPerpetual for TonClient<C> {}
impl<C: Client> ChainAddressStatus for TonClient<C> {}
impl<C: Client> ChainStaking for TonClient<C> {}
impl<C: Client> chain_traits::ChainProvider for TonClient<C> {
    fn get_chain(&self) -> primitives::Chain {
        Chain::Ton
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;
    use primitives::asset_constants::TON_DUST_TOKEN_ID;

    use super::*;

    fn mock_client(expected_path: &'static str, response: &'static [u8]) -> TonClient<MockClient> {
        TonClient::new(MockClient::new().with_get(move |path| {
            assert_eq!(path, expected_path);
            Ok(response.to_vec())
        }))
    }

    #[tokio::test]
    async fn test_get_token_data_v3() {
        let client = mock_client(
            "/api/v3/jetton/masters?address=EQBlqsm144Dq6SjbPI4jjZvA1hqTIP3CvHovbIfW_t-SCALE",
            include_bytes!("../../testdata/jetton_master_dedust.json"),
        );
        let dedust = client.get_token_data(TON_DUST_TOKEN_ID.to_string()).await.unwrap();
        assert_eq!(dedust.name, "DeDust");
        assert_eq!(dedust.symbol, "DUST");
        assert_eq!(dedust.decimals, 9);

        let client = mock_client("/api/v3/jetton/masters?address=inline", include_bytes!("../../testdata/jetton_master_inline.json"));
        let inline = client.get_token_data("inline".to_string()).await.unwrap();
        assert_eq!(inline.name, "Inline Token");
        assert_eq!(inline.symbol, "INL");
        assert_eq!(inline.decimals, 8);

        let client = mock_client(
            "/api/v3/jetton/masters?address=indexed_decimals",
            include_bytes!("../../testdata/jetton_master_indexed_decimals.json"),
        );
        let indexed_decimals = client.get_token_data("indexed_decimals".to_string()).await.unwrap();
        assert_eq!(indexed_decimals.name, "Indexed Token");
        assert_eq!(indexed_decimals.symbol, "IDX");
        assert_eq!(indexed_decimals.decimals, 6);

        let client = mock_client("/api/v3/jetton/masters?address=missing", include_bytes!("../../testdata/jetton_master_missing.json"));
        let missing_master = client.get_token_data("missing".to_string()).await.unwrap_err();
        assert_eq!(missing_master.to_string(), "missing jetton master");

        let client = mock_client(
            "/api/v3/jetton/masters?address=invalid_token_info",
            include_bytes!("../../testdata/jetton_master_invalid_token_info.json"),
        );
        let invalid_token_info = client.get_token_data("invalid_token_info".to_string()).await.unwrap_err();
        assert_eq!(invalid_token_info.to_string(), "invalid jetton metadata");

        let client = mock_client(
            "/api/v3/jetton/masters?address=missing_fields",
            include_bytes!("../../testdata/jetton_master_missing_fields.json"),
        );
        let missing_fields = client.get_token_data("missing_fields".to_string()).await.unwrap_err();
        assert_eq!(missing_fields.to_string(), "invalid jetton metadata");

        let client = mock_client(
            "/api/v3/jetton/masters?address=invalid_decimals",
            include_bytes!("../../testdata/jetton_master_invalid_decimals.json"),
        );
        let invalid_decimals = client.get_token_data("invalid_decimals".to_string()).await.unwrap_err();
        assert_eq!(invalid_decimals.to_string(), "invalid jetton decimals");
    }
}

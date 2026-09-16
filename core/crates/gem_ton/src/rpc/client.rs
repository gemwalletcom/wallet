use std::error::Error;

use primitives::{Asset, AssetId, AssetType, chain::Chain};

use chain_traits::{ChainAccount, ChainAddressStatus, ChainPerpetual, ChainStaking, ChainTraits};
use gem_client::{Client, ClientExt};

use crate::models::{
    AddressInformation, BroadcastTransaction, Chainhead, DnsRecordsResponse, JettonMastersResponse, JettonWalletsResponse, NftCollectionsResponse, NftItemsResponse,
    RunGetMethodRequest, RunGetMethodResult, SendBocRequest, StackArg, TraceByAddressQuery, TraceByBlockQuery, TraceByMessageQuery, TraceByTransactionQuery, TraceResponse,
    WalletInfo,
    simulation::{TonEmulationRequest, TonEmulationResponse},
};
use crate::rpc::target::TonCenterTarget;

const TONCENTER_V3_BLOCK_LIMIT: usize = 100;
const TONCENTER_SORT_DESC: &str = "desc";
const TONCENTER_SORT_ASC: &str = "asc";

#[derive(Debug)]
pub struct TonClient<C: Client> {
    pub client: C,
}

impl<C: Client> TonClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_master_head(&self) -> Result<Chainhead, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(TonCenterTarget::GetMasterchainInfo).await?)
    }

    pub async fn get_dns_records(&self, domain: &str) -> Result<DnsRecordsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(TonCenterTarget::GetDnsRecords { domain: domain.to_string() }).await?)
    }

    pub async fn get_token_info(&self, token_id: &str) -> Result<JettonMastersResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(TonCenterTarget::GetJettonMasters { address: token_id.to_string() }).await?)
    }

    pub async fn get_balance(&self, address: String) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get::<AddressInformation>(TonCenterTarget::GetAddressInformation { address }).await?.balance)
    }

    pub async fn get_wallet_information(&self, address: String) -> Result<WalletInfo, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(TonCenterTarget::GetWalletInformation { address }).await?)
    }

    pub async fn broadcast_transaction(&self, data: String) -> Result<BroadcastTransaction, Box<dyn Error + Send + Sync>> {
        Ok(self.client.post(TonCenterTarget::SendBoc, &SendBocRequest { boc: data }).await?)
    }

    pub(crate) async fn emulate_ton_connect(&self, request: &TonEmulationRequest<'_>) -> Result<TonEmulationResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.post(TonCenterTarget::EmulateTonConnect, request).await?)
    }

    pub async fn run_get_method(&self, address: &str, method: &str, stack: Vec<StackArg>) -> Result<RunGetMethodResult, Box<dyn Error + Send + Sync>> {
        let request = RunGetMethodRequest {
            address: address.to_string(),
            method: method.to_string(),
            stack,
        };
        Ok(self.client.post(TonCenterTarget::RunGetMethod, &request).await?)
    }

    pub async fn get_traces_by_message(&self, hash: String) -> Result<TraceResponse, Box<dyn Error + Send + Sync>> {
        let query = TraceByMessageQuery {
            msg_hash: hash,
            include_actions: true,
        };
        Ok(self.client.get(TonCenterTarget::GetTracesByMessage { query }).await?)
    }

    pub async fn get_traces_by_transaction(&self, hash: String) -> Result<TraceResponse, Box<dyn Error + Send + Sync>> {
        let query = TraceByTransactionQuery {
            tx_hash: hash,
            include_actions: true,
        };
        Ok(self.client.get(TonCenterTarget::GetTracesByTransaction { query }).await?)
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
        Ok(self.client.get(TonCenterTarget::GetTracesByBlock { query }).await?)
    }

    pub async fn get_traces_by_address(&self, address: String, limit: usize) -> Result<TraceResponse, Box<dyn Error + Send + Sync>> {
        let query = TraceByAddressQuery {
            account: address,
            include_actions: true,
            limit,
            offset: 0,
            sort: TONCENTER_SORT_DESC,
        };
        Ok(self.client.get(TonCenterTarget::GetTracesByAddress { query }).await?)
    }

    pub async fn get_jetton_wallets(&self, address: String) -> Result<JettonWalletsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(TonCenterTarget::GetJettonWallets { owner: address }).await?)
    }

    pub async fn get_nft_items_by_owner(&self, owner_address: &str) -> Result<NftItemsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(TonCenterTarget::GetNftItemsByOwner { owner: owner_address.to_string() }).await?)
    }

    pub async fn get_nft_item(&self, address: &str) -> Result<NftItemsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(TonCenterTarget::GetNftItem { address: address.to_string() }).await?)
    }

    pub async fn get_nft_collection(&self, collection_address: &str) -> Result<NftCollectionsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(TonCenterTarget::GetNftCollection {
                address: collection_address.to_string(),
            })
            .await?)
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
    use chain_traits::ChainTransactionBroadcast;
    use gem_client::testkit::MockClient;
    use primitives::BroadcastOptions;
    use primitives::asset_constants::TON_DUST_TOKEN_ID;

    use super::*;

    fn mock_client(expected_path: &'static str, response: &'static [u8]) -> TonClient<MockClient> {
        TonClient::new(MockClient::new().with_get(move |path| {
            assert_eq!(path, expected_path);
            Ok(response.to_vec())
        }))
    }

    #[tokio::test]
    async fn test_get_balance_v3() {
        let client = mock_client("/api/v3/addressInformation?address=account", br#"{"balance":"79349435046","status":"active"}"#);
        assert_eq!(client.get_balance("account".into()).await.unwrap(), "79349435046");
        let client = mock_client("/api/v3/addressInformation?address=account", br#"{"balance":"0","status":"uninit"}"#);
        assert_eq!(client.get_balance("account".into()).await.unwrap(), "0");
    }

    #[tokio::test]
    async fn test_get_wallet_information_v3() {
        let client = mock_client("/api/v3/walletInformation?address=account", br#"{"seqno":217,"status":"active"}"#);
        assert_eq!(client.get_wallet_information("account".into()).await.unwrap().seqno, Some(217));
        let client = mock_client("/api/v3/walletInformation?address=account", br#"{"status":"uninit"}"#);
        assert_eq!(client.get_wallet_information("account".into()).await.unwrap().seqno, None);
    }

    #[tokio::test]
    async fn test_transaction_broadcast_v3() {
        let client = TonClient::new(MockClient::new().with_post(|path, body| {
            assert_eq!(path, "/api/v3/message");
            assert_eq!(body, br#"{"boc":"signed-message"}"#);
            Ok(br#"{"message_hash":"gyjq/7IJ5KpSvZlnwixaS3RjI2xk1+5pup0k++S/yXY=","message_hash_norm":"different-normalized-hash"}"#.to_vec())
        }));
        assert_eq!(
            client.transaction_broadcast("signed-message".into(), BroadcastOptions::default()).await.unwrap(),
            "8328eaffb209e4aa52bd9967c22c5a4b7463236c64d7ee69ba9d24fbe4bfc976"
        );
    }

    #[tokio::test]
    async fn test_run_get_method_v3() {
        let client = TonClient::new(MockClient::new().with_post(|path, body| {
            assert_eq!(path, "/api/v3/runGetMethod");
            assert_eq!(body, br#"{"address":"account","method":"get_wallet_address","stack":[{"type":"slice","value":"te6cc"}]}"#);
            Ok(br#"{"exit_code":0,"stack":[{"type":"cell","value":"te6address"}]}"#.to_vec())
        }));
        let result = client.run_get_method("account", "get_wallet_address", vec![StackArg::slice("te6cc")]).await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stack[0].as_cell_bytes(), Some("te6address"));
    }

    #[tokio::test]
    async fn test_v3_error_responses_are_rejected() {
        let client = TonClient::new(MockClient::new().with_post(|_, _| Ok(br#"{"error":"invalid request","code":400}"#.to_vec())));
        assert!(client.run_get_method("account", "seqno", vec![]).await.is_err());
        assert!(client.transaction_broadcast("invalid".into(), BroadcastOptions::default()).await.is_err());
        let client = mock_client("/api/v3/addressInformation?address=account", br#"{"error":"invalid address"}"#);
        assert!(client.get_balance("account".into()).await.is_err());
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

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use std::error::Error;

    use primitives::asset_constants::TON_USDT_TOKEN_ID;

    use crate::{
        address::Address,
        models::StackArg,
        provider::testkit::{TEST_ADDRESS, create_ton_test_client},
    };

    #[tokio::test]
    async fn test_v3_wallet_and_contract_calls() -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = create_ton_test_client();
        client.get_balance(TEST_ADDRESS.into()).await?.parse::<u64>()?;
        assert!(client.get_wallet_information(TEST_ADDRESS.into()).await?.seqno.is_some());
        let result = client.run_get_method(TEST_ADDRESS, "seqno", vec![]).await?;
        assert_eq!(result.exit_code, 0);
        assert!(result.stack[0].as_num().is_some());
        let result = client
            .run_get_method(
                TON_USDT_TOKEN_ID,
                "get_wallet_address",
                vec![StackArg::slice(Address::parse(TEST_ADDRESS)?.to_boc_base64()?)],
            )
            .await?;
        assert_eq!(result.exit_code, 0);
        Address::from_boc_base64(result.stack[0].as_cell_bytes().ok_or("missing jetton wallet address")?)?;
        Ok(())
    }
}

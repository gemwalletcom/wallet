use crate::{Options, QuoteRequest, SwapperQuoteAsset};
use gem_client::testkit::MockClient;
use gem_ton::rpc::client::TonClient;
use num_bigint::BigUint;
use primitives::{AssetId, Chain, asset_constants::TON_USDT_TOKEN_ID};
use serde::Deserialize;
use serde_json::Value;
use std::sync::{Arc, Mutex};

pub const TEST_WALLET: &str = "EQBI792u0Cq0CKwY2yGc-jpVhiFMVcV1Zd-X69_dEtW1tZGG";
pub const TEST_USDT_WALLET: &str = "EQDTUKJxH6LRwJYvQ5DkDgYmBt033ECr_q9MhmDwkBxM-DUJ";
pub const NOT_TOKEN_ID: &str = "EQAvlWFDxGF2lXm67y4yzC17wYKD9A0guwPkMs1gOsM__NOT";
pub const STTON_TOKEN_ID: &str = "EQDNhy-nxYFgUqzfUzImBEP67JqsyMIcyk2S5_RwNNEYku0k";
pub const NATIVE_VAULT: &str = "EQDa4VOnTYlLvDJ0gZjNYm5PXfSmmtL6Vs6A_CZEtXCNICq_";
pub const USDT_VAULT: &str = "EQAYqo4u7VF0fa4DPAebk4g9lBytj2VFny7pzXR0trjtXQaO";
pub const TON_USDT_POOL: &str = "EQA-X_yo3fzzbDbJ_0bzFWKqtRuZFIRa1sJsveZJ1YpViO3r";
pub const TON_NOT_POOL: &str = "EQCUhnOllvxtLDEj3_9k0yMeTQ4Wv6rFBtXr2wnzKzucZeER";
pub const USDT_NOT_POOL: &str = "EQA26n1eVOKbzxZQr7hIswf7_kykj8gGhybp_7w5FFmwyA21";
pub const TON_STTON_STABLE_POOL: &str = "EQCHFiQM_TTSIiKhUCmWSN4aPSTqxJ4VSBEyDFaZ4izyq95Y";

pub fn ton() -> AssetId {
    AssetId::from_chain(Chain::Ton)
}

pub fn usdt() -> AssetId {
    AssetId::from_token(Chain::Ton, TON_USDT_TOKEN_ID)
}

pub fn not() -> AssetId {
    AssetId::from_token(Chain::Ton, NOT_TOKEN_ID)
}

pub fn stton() -> AssetId {
    AssetId::from_token(Chain::Ton, STTON_TOKEN_ID)
}

pub fn mock_request(from: AssetId, to: AssetId, value: u64) -> QuoteRequest {
    QuoteRequest {
        from_asset: SwapperQuoteAsset::from(from),
        to_asset: SwapperQuoteAsset::from(to),
        wallet_address: TEST_WALLET.to_string(),
        destination_address: TEST_WALLET.to_string(),
        value: BigUint::from(value),
        options: Options::default(),
    }
}

#[derive(Deserialize)]
struct GetMethodRecord {
    request: Value,
    response: Value,
}

pub fn mock_ton_client(records: &str) -> TonClient<MockClient> {
    mock_ton_client_with_calls(records, Arc::default())
}

pub fn mock_ton_client_with_calls(records: &str, calls: Arc<Mutex<Vec<String>>>) -> TonClient<MockClient> {
    let records: Vec<GetMethodRecord> = serde_json::from_str(records).unwrap();
    TonClient::new(MockClient::new().with_post(move |_, body| {
        let request: Value = serde_json::from_slice(body).unwrap();
        calls.lock().unwrap().push(request["method"].as_str().unwrap_or_default().to_string());
        let record = records.iter().find(|record| record.request == request).unwrap_or_else(|| panic!("unrecorded TON get-method {request}"));
        Ok(serde_json::to_vec(&record.response).unwrap())
    }))
}

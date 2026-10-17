use gem_aptos::{AccountAddress, AptosClient};
use gem_client::{ClientError, testkit::MockClient};
use primitives::{asset_constants::APTOS_USDC_TOKEN_ID, swap::APTOS_REFERRAL_ADDRESS};
use serde::Deserialize;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use super::model::PartnerSwap;

impl PartnerSwap {
    pub fn mock() -> Self {
        Self {
            pools: vec!["0x925660b8618394809f89f8002e2926600c775221f43bf1919782b297a79400d8".to_string()],
            token_in: AccountAddress::from_hex("0xa").unwrap(),
            token_out: AccountAddress::from_hex(APTOS_USDC_TOKEN_ID).unwrap(),
            amount_in: 100_000_000,
            amount_out_min: 834_397,
            fee: 500_000,
            fee_receiver: AccountAddress::from_hex(APTOS_REFERRAL_ADDRESS).unwrap(),
        }
    }
}

#[derive(Deserialize)]
struct ViewRecord {
    function: String,
    arguments: Value,
    status: u16,
    response: Value,
}

pub fn mock_aptos_client(views: &str) -> AptosClient<MockClient> {
    mock_aptos_client_with_calls(views, Arc::default())
}

pub fn mock_aptos_client_with_calls(views: &str, calls: Arc<Mutex<Vec<String>>>) -> AptosClient<MockClient> {
    let records: Vec<ViewRecord> = serde_json::from_str(views).unwrap();
    AptosClient::new(MockClient::new().with_post(move |_, body| {
        let request: Value = serde_json::from_slice(body).unwrap();
        calls.lock().unwrap().push(request["function"].as_str().unwrap_or_default().to_string());
        let record = records
            .iter()
            .find(|record| request["function"] == record.function && request["arguments"] == record.arguments)
            .unwrap_or_else(|| panic!("unrecorded Aptos view {request}"));
        let response = serde_json::to_vec(&record.response).unwrap();
        match record.status {
            200 => Ok(response),
            status => Err(ClientError::Http { status, body: response }),
        }
    }))
}

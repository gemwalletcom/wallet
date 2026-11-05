use alloy_primitives::hex;
use alloy_sol_types::SolError;
use gem_client::testkit::MockClient;
use serde_json::Value;

use super::ccip::CcipClient;
use super::contract::UniversalResolver::OffchainLookup;
use super::provider::EnsProvider;

pub const TEST_CCIP_URL: &str = "https://ccip-v3.ens.xyz";
pub const TEST_LOOKUP_RESPONSE: &[u8] = include_bytes!("../../../testdata/ens_offchain_lookup_response.json");
pub const TEST_CCIP_RESPONSE: &[u8] = include_bytes!("../../../testdata/ens_ccip_response.json");

impl OffchainLookup {
    pub fn mock() -> Self {
        let response: Value = serde_json::from_slice(TEST_LOOKUP_RESPONSE).unwrap();
        Self::abi_decode(&hex::decode(response["error"]["data"].as_str().unwrap()).unwrap()).unwrap()
    }
}

impl CcipClient<MockClient> {
    pub fn mock(client: MockClient) -> Self {
        Self::new(client, TEST_CCIP_URL)
    }
}

impl EnsProvider<MockClient> {
    pub fn mock(client: MockClient, ccip: MockClient) -> Self {
        Self::new(client, CcipClient::mock(ccip))
    }

    pub fn mock_with_response(response: impl Into<Vec<u8>>) -> Self {
        let response = response.into();
        Self::mock(MockClient::new().with_post(move |_, _| Ok(response.clone())), MockClient::new())
    }
}

pub fn mock_offchain_rpc(callback_response: Vec<u8>) -> MockClient {
    let response: Value = serde_json::from_slice(TEST_CCIP_RESPONSE).unwrap();
    let callback = hex::encode_prefixed(OffchainLookup::mock().callback(hex::decode(response["data"].as_str().unwrap()).unwrap().into()));
    MockClient::new().with_post(move |_, body| {
        let data = serde_json::from_slice::<Value>(body).unwrap()["params"][0]["data"].take();
        Ok(if data == callback.as_str() { callback_response.clone() } else { TEST_LOOKUP_RESPONSE.to_vec() })
    })
}

use alloy_primitives::hex;
use alloy_sol_types::SolError;
use serde_json::Value;

use super::contract::UniversalResolver::OffchainLookup;

pub const CCIP_URL: &str = "https://ccip-v3.ens.xyz";

pub fn offchain_lookup() -> OffchainLookup {
    let response: Value = serde_json::from_str(include_str!("../../../testdata/ens_offchain_lookup_response.json")).unwrap();
    OffchainLookup::abi_decode(&hex::decode(response["error"]["data"].as_str().unwrap()).unwrap()).unwrap()
}

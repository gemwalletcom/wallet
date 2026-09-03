#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::client::AptosClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use settings::testkit::get_test_settings;

use crate::DEFAULT_MAX_GAS_AMOUNT;
use crate::models::{TransactionSignature, TransactionSimulation};
use crate::provider::payload_builder::build_transfer_transaction_payload;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "0x6467997d9c3a5bc9f714e17a168984595ce9bec7350645713a1fe7983a7f5fcc";
#[cfg(test)]
pub const TEST_TRANSACTION_ID: &str = "0x6a43e0034486583a30cff449c03c4d882c641b351e392096272496168240de8e";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS_STAKING: &str = "0xc95615aa095c100b18eb6eaa0f0a0f30b9cd96685118a7cbc1a2328a91ca2eda";

impl TransactionSimulation {
    pub fn mock() -> Self {
        Self {
            expiration_timestamp_secs: "1".to_string(),
            gas_unit_price: "100".to_string(),
            max_gas_amount: DEFAULT_MAX_GAS_AMOUNT.to_string(),
            payload: build_transfer_transaction_payload("0x1", "1"),
            sender: "0x1".to_string(),
            sequence_number: "0".to_string(),
            signature: TransactionSignature::no_account(),
        }
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_aptos_test_client() -> AptosClient<ReqwestClient> {
    let settings = get_test_settings();
    let reqwest_client = ReqwestClient::new(settings.chains.aptos.url, gem_client::reqwest_client());
    AptosClient::new(reqwest_client)
}

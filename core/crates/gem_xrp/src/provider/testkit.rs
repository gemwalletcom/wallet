#[cfg(test)]
use crate::models::rpc::AccountInfo;
#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::XrpClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_jsonrpc::client::JsonRpcClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use settings::testkit::get_test_settings;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "rnZmVGX6f4pUYyS4oXYJzoLdRojQV8y297";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS_EMPTY: &str = "rPGZTtsiBXS8izwJcktUmxtzZSic1jbpLi";
#[cfg(test)]
pub const TEST_TRANSACTION_ID: &str = "474F58E6C78F1DE8542036AB3C16E2B5A4089241DEE3E58142154DC3CA0E8271";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_xrp_test_client() -> XrpClient<ReqwestClient> {
    let settings = get_test_settings();
    let reqwest_client = ReqwestClient::new(settings.chains.xrp.url, gem_client::reqwest_client());
    let rpc_client = JsonRpcClient::new(reqwest_client);
    XrpClient::new(rpc_client)
}

#[cfg(test)]
impl AccountInfo {
    pub fn mock_with_balance(balance: u64, owner_count: u32) -> Self {
        Self {
            balance,
            sequence: 100,
            owner_count,
            account: None,
            flags: None,
            ledger_entry_type: None,
        }
    }
}

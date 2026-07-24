#[cfg(feature = "chain_integration_tests")]
use crate::rpc::AlgorandProvider;
#[cfg(feature = "chain_integration_tests")]
use gem_client::ReqwestClient;
#[cfg(feature = "chain_integration_tests")]
use settings::testkit::get_test_settings;

#[cfg(test)]
pub const TEST_TRANSACTION_ID: &str = "LAEWXAG6FYFIEDAY76YQFKO46EIKEOIT4GTONUQFD6TL23XG45KQ";

#[cfg(feature = "chain_integration_tests")]
pub const TEST_ADDRESS: &str = "RXIOUIR5IGFZMIZ7CR7FJXDYY4JI7NZG5UCWCZZNWXUPFJRLG6K6X5ITXM";

#[cfg(feature = "chain_integration_tests")]
pub fn create_algorand_test_client() -> AlgorandProvider<ReqwestClient> {
    use crate::rpc::{AlgorandClient, AlgorandIndexer};

    let settings = get_test_settings();
    let client = gem_client::reqwest_client();
    AlgorandProvider::new(
        AlgorandClient::new(ReqwestClient::new(settings.chains.algorand.url, client.clone())),
        Box::new(AlgorandIndexer::new(ReqwestClient::new(settings.indexer.algorand.url, client))),
    )
}

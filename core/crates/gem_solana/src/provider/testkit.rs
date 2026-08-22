#[cfg(feature = "chain_integration_tests")]
use crate::rpc::{SolanaClient, SolanaIndexer, SolanaProvider};
#[cfg(feature = "chain_integration_tests")]
use gem_client::ReqwestClient;
#[cfg(feature = "chain_integration_tests")]
use gem_jsonrpc::JsonRpcClient;
#[cfg(feature = "chain_integration_tests")]
use settings::testkit::get_test_settings;

#[cfg(feature = "chain_integration_tests")]
pub const TEST_EMPTY_ADDRESS: &str = "EniLGJRPvjbD51z5r59HRN4XoeMmRC4zMtHNHBKi1sFA";
#[cfg(test)]
pub const TEST_TRANSACTION_ID: &str = "4dHnggcXjvmMJY2J6iGqse12PeCYQzuTySgwJa36K8MuntmwNrCNztvYRX5ZGpQXzKjaf7g5vaZM7LTuXLNbi2Zx";

#[cfg(feature = "chain_integration_tests")]
pub fn create_solana_test_client() -> SolanaProvider<ReqwestClient> {
    let settings = get_test_settings();
    let alchemy_url = settings
        .indexer
        .alchemy
        .url
        .replace("{network}", "solana-mainnet")
        .replace("{key}", &settings.indexer.alchemy.key.secret);
    SolanaProvider::new(
        SolanaClient::new(JsonRpcClient::new(ReqwestClient::new_test_client(settings.chains.solana.url))),
        Box::new(SolanaIndexer::new(JsonRpcClient::new(ReqwestClient::new_test_client(alchemy_url)))),
    )
}

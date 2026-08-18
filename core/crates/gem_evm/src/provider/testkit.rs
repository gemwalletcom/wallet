#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
use std::sync::Arc;

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
use crate::{
    ether_conv,
    rpc::{EVMAssetBalanceProvider, EVMIndexer, EVMTransactionsByAddressProvider, EthereumClient, EthereumProvider, alchemy_url},
};
#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
use gem_jsonrpc::JsonRpcClient;
#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
use primitives::{EVMChain, FeeRate};
#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
use settings::testkit::get_test_settings;

pub use crate::testkit::{TEST_ADDRESS, TEST_MONAD_ADDRESS, TEST_SMARTCHAIN_STAKING_ADDRESS, TEST_TRANSACTION_ID, TOKEN_DAI_ADDRESS, TOKEN_USDC_ADDRESS};

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
fn build_test_client(chain: EVMChain, rpc_url: &str) -> EthereumProvider<ReqwestClient> {
    let client = ReqwestClient::new_test_client(rpc_url.to_string());
    EthereumProvider::new_rpc_only(EthereumClient::new(JsonRpcClient::new(client), chain))
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
fn build_test_indexer(chain: EVMChain, rpc_url: &str) -> Arc<EVMIndexer<ReqwestClient>> {
    let settings = get_test_settings();
    let client = ReqwestClient::new_test_client(rpc_url.to_string());
    Arc::new(
        EVMIndexer::for_chain(
            client.clone().with_request_timeout(settings.indexer.alchemy.request.timeout).with_base_url(alchemy_url(
                chain.to_chain(),
                &settings.indexer.alchemy.url,
                &settings.indexer.alchemy.key.secret,
            )),
            client.clone().with_request_timeout(settings.indexer.ankr.request.timeout).with_base_url(format!(
                "{}/{}",
                settings.indexer.ankr.url.trim_end_matches('/'),
                settings.indexer.ankr.key.secret
            )),
            settings.indexer.blockscout.remote_provider_config().configure_client(client),
            settings.indexer.blockscout.key.secret,
            chain,
        )
        .unwrap(),
    )
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
pub fn create_ethereum_test_client() -> EthereumProvider<ReqwestClient> {
    let settings = get_test_settings();
    build_test_client(EVMChain::Ethereum, &settings.chains.ethereum.url)
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
pub fn create_ethereum_test_transactions_by_address_provider() -> EVMTransactionsByAddressProvider<ReqwestClient> {
    let settings = get_test_settings();
    EVMTransactionsByAddressProvider::new(build_test_indexer(EVMChain::Ethereum, &settings.chains.ethereum.url))
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
pub fn create_ethereum_test_asset_balance_provider() -> EVMAssetBalanceProvider<ReqwestClient> {
    let settings = get_test_settings();
    EVMAssetBalanceProvider::new(build_test_indexer(EVMChain::Ethereum, &settings.chains.ethereum.url))
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
pub fn create_smartchain_test_client() -> EthereumProvider<ReqwestClient> {
    let settings = get_test_settings();
    build_test_client(EVMChain::SmartChain, &settings.chains.smartchain.url)
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
pub fn create_arbitrum_test_client() -> EthereumProvider<ReqwestClient> {
    let settings = get_test_settings();
    build_test_client(EVMChain::Arbitrum, &settings.chains.arbitrum.url)
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
pub fn create_monad_test_client() -> EthereumProvider<ReqwestClient> {
    let settings = get_test_settings();
    build_test_client(EVMChain::Monad, &settings.chains.monad.url)
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
pub fn print_fee_rates(fee_rates: Vec<FeeRate>) {
    for fee_rate in &fee_rates {
        println!(
            "Fee rate: {:?} total: {}, gas_price: {}, priority_fee: {}",
            fee_rate.priority,
            ether_conv::EtherConv::to_gwei(&fee_rate.gas_price_type.total_fee()),
            ether_conv::EtherConv::to_gwei(&fee_rate.gas_price_type.gas_price()),
            ether_conv::EtherConv::to_gwei(&fee_rate.gas_price_type.priority_fee())
        );
    }
}

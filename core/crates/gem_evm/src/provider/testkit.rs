pub use crate::testkit::{TEST_ADDRESS, TEST_MONAD_ADDRESS, TEST_SMARTCHAIN_STAKING_ADDRESS, TEST_TRANSACTION_ID, TOKEN_DAI_ADDRESS, TOKEN_USDC_ADDRESS};

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
use crate::{
    ether_conv,
    rpc::{EVMIndexer, EthereumClient},
};
#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
use gem_jsonrpc::JsonRpcClient;
#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
use primitives::{EVMChain, FeeRate};
#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
use settings::testkit::get_test_settings;

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
fn build_test_client(chain: EVMChain, rpc_url: &str) -> EthereumClient<ReqwestClient> {
    let settings = get_test_settings();
    let rpc_client = JsonRpcClient::new_reqwest(rpc_url.to_string());

    let indexer = EVMIndexer::new(
        JsonRpcClient::new_reqwest(format!("https://rpc.ankr.com/multichain/{}", settings.ankr.key.secret)),
        None,
        chain,
    );

    EthereumClient::new_with_indexer(rpc_client, chain, indexer)
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
pub fn create_ethereum_test_client() -> EthereumClient<ReqwestClient> {
    let settings = get_test_settings();
    build_test_client(EVMChain::Ethereum, &settings.chains.ethereum.url)
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
pub fn create_smartchain_test_client() -> EthereumClient<ReqwestClient> {
    let settings = get_test_settings();
    build_test_client(EVMChain::SmartChain, &settings.chains.smartchain.url)
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
pub fn create_polygon_test_client() -> EthereumClient<ReqwestClient> {
    let settings = get_test_settings();
    build_test_client(EVMChain::Polygon, &settings.chains.polygon.url)
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
pub fn create_arbitrum_test_client() -> EthereumClient<ReqwestClient> {
    let settings = get_test_settings();
    build_test_client(EVMChain::Arbitrum, &settings.chains.arbitrum.url)
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
pub fn create_monad_test_client() -> EthereumClient<ReqwestClient> {
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

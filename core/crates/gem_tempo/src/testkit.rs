#[cfg(all(feature = "rpc", feature = "reqwest"))]
use gem_client::ReqwestClient;
#[cfg(all(feature = "rpc", feature = "reqwest"))]
use gem_evm::rpc::EthereumClient;
#[cfg(all(feature = "rpc", feature = "reqwest"))]
use primitives::EVMChain;
use primitives::{Asset, Chain, TransactionInputType, TransferDataExtra, WalletConnectionSessionAppMetadata};

#[cfg(feature = "rpc")]
pub(crate) const TEMPO_TEST_ADDRESS: &str = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";
pub(crate) const TEMPO_TEST_USER_FEE_TOKEN: &str = "0x20C00000000000000000000014f22CA97301EB73";
pub(crate) fn mock_tempo_generic_input(to: &str, data: Vec<u8>) -> TransactionInputType {
    TransactionInputType::Generic(
        Asset::from_chain(Chain::Tempo),
        WalletConnectionSessionAppMetadata::mock(),
        TransferDataExtra {
            to: to.to_string(),
            data: Some(data),
            ..TransferDataExtra::mock()
        },
    )
}

#[cfg(all(feature = "rpc", feature = "reqwest"))]
pub(crate) fn create_tempo_test_client() -> EthereumClient<ReqwestClient> {
    let settings = settings::testkit::get_test_settings();
    EthereumClient::mock_with_url(EVMChain::Tempo, &settings.chains.tempo.url)
}

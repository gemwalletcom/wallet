use primitives::{Asset, AssetType, Chain, TransactionInputType, TransferDataExtra, WalletConnectionSessionAppMetadata};

pub const TEMPO_TEST_ADDRESS: &str = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";
pub const TEMPO_TEST_USER_FEE_TOKEN: &str = "0x20C00000000000000000000014f22CA97301EB73";
pub const TEMPO_TEST_CBBTC_TOKEN: &str = "0x20C000000000000000000000c412Ec89D0c08be5";

pub fn mock_tempo_cbbtc_asset() -> Asset {
    Asset::mock_with_params(
        Chain::Tempo,
        Some(TEMPO_TEST_CBBTC_TOKEN.to_string()),
        "Coinbase Wrapped BTC".to_string(),
        "cbBTC".to_string(),
        6,
        AssetType::TIP20,
    )
}

pub fn mock_tempo_generic_input(to: &str, data: Vec<u8>) -> TransactionInputType {
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
pub fn create_tempo_test_client() -> gem_evm::rpc::EthereumClient<gem_client::ReqwestClient> {
    let settings = settings::testkit::get_test_settings();
    gem_evm::rpc::EthereumClient::mock_with_url(primitives::EVMChain::Tempo, &settings.chains.tempo.url)
}

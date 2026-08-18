use gem_evm::constants::{DEFAULT_SWAP_GAS_LIMIT, TOKEN_TRANSFER_GAS_LIMIT};
use primitives::{
    Asset, AssetType, Chain, SignerInput, TransactionInputType, TransactionLoadMetadata, TransferDataExtra, WalletConnectionSessionAppMetadata,
    swap::{ApprovalData, SwapData, SwapQuoteData},
};

pub(crate) const TEMPO_TEST_ADDRESS: &str = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";
pub(crate) const TEMPO_TEST_ROUTER_ADDRESS: &str = "0xA2Dc7d0266f0CC50b3eEaF36c9BFCeCFF1BEea91";
pub(crate) const TEMPO_TEST_USER_FEE_TOKEN: &str = "0x20C00000000000000000000014f22CA97301EB73";
pub(crate) const TEMPO_TEST_CBBTC_TOKEN: &str = "0x20C000000000000000000000c412Ec89D0c08be5";

const SWAP_CALL_DATA: &str = "abcd";

pub(crate) fn mock_tempo_cbbtc_asset() -> Asset {
    Asset::mock_with_params(
        Chain::Tempo,
        Some(TEMPO_TEST_CBBTC_TOKEN.to_string()),
        "Coinbase Wrapped BTC".to_string(),
        "cbBTC".to_string(),
        6,
        AssetType::TIP20,
    )
}

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

pub(crate) fn mock_tempo_swap_input(from_asset: Asset, fee_asset: Asset, approval: Option<ApprovalData>) -> SignerInput {
    let has_approval = approval.is_some();
    let gas_limit = if has_approval { TOKEN_TRANSFER_GAS_LIMIT } else { DEFAULT_SWAP_GAS_LIMIT };
    let swap_data = SwapData::mock();
    let mut input = SignerInput::mock_evm_with_metadata(
        TransactionInputType::Swap(
            from_asset,
            Asset::from_chain(Chain::Tempo),
            SwapData {
                data: SwapQuoteData {
                    to: TEMPO_TEST_ROUTER_ADDRESS.to_string(),
                    data: SWAP_CALL_DATA.to_string(),
                    gas_limit: has_approval.then(|| DEFAULT_SWAP_GAS_LIMIT.to_string()),
                    approval,
                    ..swap_data.data
                },
                ..swap_data
            },
        ),
        "0",
        gas_limit,
        TransactionLoadMetadata::mock_evm(0, Chain::Tempo.network_id().parse().unwrap()),
    );
    input.fee.fee_asset = fee_asset;
    input
}

#[cfg(all(feature = "rpc", feature = "reqwest"))]
pub(crate) fn create_tempo_test_client() -> gem_evm::rpc::EthereumClient<gem_client::ReqwestClient> {
    let settings = settings::testkit::get_test_settings();
    gem_evm::rpc::EthereumClient::mock_with_url(primitives::EVMChain::Tempo, &settings.chains.tempo.url)
}

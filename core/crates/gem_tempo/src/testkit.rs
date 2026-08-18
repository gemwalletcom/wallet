#[cfg(all(feature = "rpc", feature = "reqwest"))]
use gem_client::ReqwestClient;
#[cfg(feature = "signer")]
use gem_evm::constants::{DEFAULT_SWAP_GAS_LIMIT, TOKEN_TRANSFER_GAS_LIMIT};
#[cfg(all(feature = "rpc", feature = "reqwest"))]
use gem_evm::rpc::EthereumClient;
#[cfg(all(feature = "rpc", feature = "reqwest"))]
use primitives::EVMChain;
use primitives::{Asset, Chain, TransactionInputType, TransferDataExtra, WalletConnectionSessionAppMetadata};
#[cfg(feature = "signer")]
use primitives::{
    AssetId, SignerInput, TransactionLoadMetadata,
    known_assets::TEMPO_PATHUSD,
    swap::{ApprovalData, SwapData, SwapQuoteData},
};

#[cfg(feature = "rpc")]
pub(crate) const TEMPO_TEST_ADDRESS: &str = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";
#[cfg(feature = "signer")]
pub(crate) const TEMPO_TEST_ROUTER_ADDRESS: &str = "0xA2Dc7d0266f0CC50b3eEaF36c9BFCeCFF1BEea91";
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

#[cfg(feature = "signer")]
pub(crate) fn mock_tempo_swap_input(from_asset: Asset, fee_asset: AssetId, approval: Option<ApprovalData>) -> SignerInput {
    let has_approval = approval.is_some();
    let gas_limit = if has_approval { TOKEN_TRANSFER_GAS_LIMIT } else { DEFAULT_SWAP_GAS_LIMIT };
    let swap_data = SwapData::mock();
    let mut input = SignerInput::mock_evm_with_metadata(
        TransactionInputType::Swap(
            from_asset,
            TEMPO_PATHUSD.clone(),
            SwapData {
                data: SwapQuoteData {
                    to: TEMPO_TEST_ROUTER_ADDRESS.to_string(),
                    data: "abcd".to_string(),
                    gas_limit: if has_approval { Some(DEFAULT_SWAP_GAS_LIMIT.to_string()) } else { None },
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
pub(crate) fn create_tempo_test_client() -> EthereumClient<ReqwestClient> {
    let settings = settings::testkit::get_test_settings();
    EthereumClient::mock_with_url(EVMChain::Tempo, &settings.chains.tempo.url)
}

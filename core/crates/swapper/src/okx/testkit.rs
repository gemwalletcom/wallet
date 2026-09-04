use super::{
    constants::{PROXY_QUOTE_PATH, PROXY_SWAP_PATH},
    model::{QuoteData, TokenInfo},
    provider::OkxProvider,
};
use crate::{QuoteRequest, SwapperQuoteAsset, alien::mock::ProviderMock, testkit::mock_quote};
use gem_client::testkit::MockClient;
use num_bigint::BigUint;
use primitives::{AssetId, Chain, asset_constants::SOLANA_USDC_ASSET_ID, swap::QuoteAsset, testkit::signer_mock::TEST_SOLANA_SENDER};
use std::sync::Arc;

pub(super) const TEST_TRON_WALLET: &str = "TW1dU4L3eNm7Lw8WvieLKEHpXWAussRG9Z";
#[cfg(feature = "swap_integration_tests")]
pub(super) const TEST_EVM_WALLET: &str = "0x1085c5f70F7F7591D97da281A64688385455c2bD";

impl OkxProvider<MockClient> {
    pub fn mock(client: MockClient, rpc_result: &str) -> Self {
        Self::new_with_client(client, Arc::new(ProviderMock::new(rpc_result.to_string())))
    }
}

pub(super) fn mock_solana_request() -> QuoteRequest {
    let mut request = mock_quote(
        SwapperQuoteAsset::from(AssetId::from_chain(Chain::Solana)),
        SwapperQuoteAsset::from(SOLANA_USDC_ASSET_ID.clone()),
    );
    request.wallet_address = TEST_SOLANA_SENDER.to_string();
    request.value = BigUint::from(100000000u64);
    request
}

pub(super) fn mock_client(quote_response: &'static str, swap_response: &'static str) -> MockClient {
    MockClient::new().with_post(move |path, _| match path {
        PROXY_QUOTE_PATH => Ok(quote_response.as_bytes().to_vec()),
        PROXY_SWAP_PATH => Ok(swap_response.as_bytes().to_vec()),
        other => panic!("unexpected path: {other}"),
    })
}

pub(super) fn mock_quote_asset_with_symbol(id: &str, symbol: &str) -> QuoteAsset {
    QuoteAsset {
        symbol: symbol.to_string(),
        decimals: 18,
        ..QuoteAsset::from(AssetId::new(id).unwrap())
    }
}

pub(super) fn mock_quote_data(from_token: &str, to_token: &str) -> QuoteData {
    QuoteData {
        from_token: TokenInfo {
            token_contract_address: from_token.to_string(),
        },
        to_token: TokenInfo {
            token_contract_address: to_token.to_string(),
        },
        to_token_amount: "200".to_string(),
    }
}

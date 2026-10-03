use crate::{QuoteRequest, SwapperQuoteAsset, testkit::mock_quote};
use num_bigint::BigUint;
use primitives::{
    AssetId, Chain,
    asset_constants::{BASE_USDC_ASSET_ID, SMARTCHAIN_USDT_ASSET_ID},
};

pub const TEST_EVM_WALLET: &str = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";

pub fn mock_opbnb_to_bsc_usdt_request() -> QuoteRequest {
    QuoteRequest {
        wallet_address: TEST_EVM_WALLET.to_string(),
        destination_address: TEST_EVM_WALLET.to_string(),
        value: BigUint::from(100_000_000_000_000_000u64),
        ..mock_quote(
            SwapperQuoteAsset::mock_with_asset_id(AssetId::from_chain(Chain::OpBNB), "BNB", 18),
            SwapperQuoteAsset::mock_with_asset_id(SMARTCHAIN_USDT_ASSET_ID.clone(), "USDT", 18),
        )
    }
}

pub fn mock_base_usdc_to_bsc_usdt_request() -> QuoteRequest {
    QuoteRequest {
        wallet_address: TEST_EVM_WALLET.to_string(),
        destination_address: TEST_EVM_WALLET.to_string(),
        value: BigUint::from(100_000_000u64),
        ..mock_quote(
            SwapperQuoteAsset::mock_with_asset_id(BASE_USDC_ASSET_ID.clone(), "USDC", 6),
            SwapperQuoteAsset::mock_with_asset_id(SMARTCHAIN_USDT_ASSET_ID.clone(), "USDT", 18),
        )
    }
}

#[cfg(feature = "swap_integration_tests")]
use crate::{QuoteRequest, SwapperQuoteAsset, testkit::mock_quote};
#[cfg(feature = "swap_integration_tests")]
use num_bigint::BigUint;
#[cfg(feature = "swap_integration_tests")]
use primitives::{
    AssetId, Chain,
    asset_constants::{BASE_USDC_ASSET_ID, SMARTCHAIN_USDT_ASSET_ID, TON_USDT_ASSET_ID, XRP_RLUSD_ASSET_ID},
};

pub const TEST_EVM_WALLET: &str = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";
#[cfg(feature = "swap_integration_tests")]
pub const TEST_XRP_WALLET: &str = "rJ9L1E8CTfz8XXADbV4LJe5nxWhnA4SBfW";
pub const TEST_TON_WALLET: &str = "UQDxJKarPSp0bCta9DFgp81Mpt5hpGbuVcSxwfeza0Bin201";

#[cfg(feature = "swap_integration_tests")]
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

#[cfg(feature = "swap_integration_tests")]
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

#[cfg(feature = "swap_integration_tests")]
pub fn mock_xrp_to_bsc_usdt_request() -> QuoteRequest {
    QuoteRequest {
        wallet_address: TEST_XRP_WALLET.to_string(),
        destination_address: TEST_EVM_WALLET.to_string(),
        value: BigUint::from(10_000_000u64),
        ..mock_quote(
            SwapperQuoteAsset::mock_with_asset_id(AssetId::from_chain(Chain::Xrp), "XRP", 6),
            SwapperQuoteAsset::mock_with_asset_id(SMARTCHAIN_USDT_ASSET_ID.clone(), "USDT", 18),
        )
    }
}

#[cfg(feature = "swap_integration_tests")]
pub fn mock_rlusd_to_bsc_usdt_request() -> QuoteRequest {
    QuoteRequest {
        wallet_address: TEST_XRP_WALLET.to_string(),
        destination_address: TEST_EVM_WALLET.to_string(),
        value: BigUint::from(10_000_000_000_000_000u64),
        ..mock_quote(
            SwapperQuoteAsset::mock_with_asset_id(XRP_RLUSD_ASSET_ID.clone(), "RLUSD", 15),
            SwapperQuoteAsset::mock_with_asset_id(SMARTCHAIN_USDT_ASSET_ID.clone(), "USDT", 18),
        )
    }
}

#[cfg(feature = "swap_integration_tests")]
pub fn mock_base_usdc_to_ton_usdt_request() -> QuoteRequest {
    QuoteRequest {
        wallet_address: TEST_EVM_WALLET.to_string(),
        destination_address: TEST_TON_WALLET.to_string(),
        value: BigUint::from(100_000_000u64),
        ..mock_quote(
            SwapperQuoteAsset::mock_with_asset_id(BASE_USDC_ASSET_ID.clone(), "USDC", 6),
            SwapperQuoteAsset::mock_with_asset_id(TON_USDT_ASSET_ID.clone(), "USDT", 6),
        )
    }
}

#[cfg(feature = "swap_integration_tests")]
pub fn mock_ton_to_bsc_usdt_request() -> QuoteRequest {
    QuoteRequest {
        wallet_address: TEST_TON_WALLET.to_string(),
        destination_address: TEST_EVM_WALLET.to_string(),
        value: BigUint::from(20_000_000_000u64),
        ..mock_quote(
            SwapperQuoteAsset::mock_with_asset_id(AssetId::from_chain(Chain::Ton), "TON", 9),
            SwapperQuoteAsset::mock_with_asset_id(SMARTCHAIN_USDT_ASSET_ID.clone(), "USDT", 18),
        )
    }
}

#[cfg(feature = "swap_integration_tests")]
pub fn mock_ton_usdt_to_bsc_usdt_request() -> QuoteRequest {
    QuoteRequest {
        wallet_address: TEST_TON_WALLET.to_string(),
        destination_address: TEST_EVM_WALLET.to_string(),
        value: BigUint::from(50_000_000u64),
        ..mock_quote(
            SwapperQuoteAsset::mock_with_asset_id(TON_USDT_ASSET_ID.clone(), "USDT", 6),
            SwapperQuoteAsset::mock_with_asset_id(SMARTCHAIN_USDT_ASSET_ID.clone(), "USDT", 18),
        )
    }
}

#[cfg(feature = "swap_integration_tests")]
pub fn mock_stable_to_bsc_usdt_request() -> QuoteRequest {
    QuoteRequest {
        wallet_address: TEST_EVM_WALLET.to_string(),
        destination_address: TEST_EVM_WALLET.to_string(),
        value: BigUint::from(20_000_000_000_000_000_000u128),
        ..mock_quote(
            SwapperQuoteAsset::mock_with_asset_id(AssetId::from_chain(Chain::Stable), "USDT0", 18),
            SwapperQuoteAsset::mock_with_asset_id(SMARTCHAIN_USDT_ASSET_ID.clone(), "USDT", 18),
        )
    }
}

use crate::{Options, QuoteRequest, SwapperQuoteAsset};
use num_bigint::BigUint;
use primitives::{AssetId, Chain};

pub const TEST_WALLET: &str = "GCVQA5JVHNDJFPODJZU6BM7H5344VK2KGBFXSUUEP7ZH3T5DXGMCR6NL";
pub const EURC_TOKEN_ID: &str = "GDHU6WRG4IEQXM5NZ4BMPKOXHW76MZM4Y2IEMFDVXBSDP6SJY4ITNPP2::EURC";

pub fn eurc() -> AssetId {
    AssetId::from_token(Chain::Stellar, EURC_TOKEN_ID)
}

pub fn mock_request(from: AssetId, to: AssetId, value: u64) -> QuoteRequest {
    QuoteRequest {
        from_asset: SwapperQuoteAsset::from(from),
        to_asset: SwapperQuoteAsset::from(to),
        wallet_address: TEST_WALLET.to_string(),
        destination_address: TEST_WALLET.to_string(),
        value: BigUint::from(value),
        options: Options::default(),
    }
}

use alloy_primitives::{Address, address};
use primitives::asset_constants::{BASE_USDC_TOKEN_ID, ETHEREUM_USDT_TOKEN_ID};
use primitives::{AssetId, Chain};

#[derive(Debug, Clone, Copy)]
pub struct YoAsset {
    pub chain: Chain,
    pub asset_token: &'static str,
    pub yo_token: Address,
}

impl YoAsset {
    pub fn asset_id(&self) -> AssetId {
        AssetId::from_token(self.chain, self.asset_token)
    }
}

pub const YO_USDC: YoAsset = YoAsset {
    chain: Chain::Base,
    asset_token: BASE_USDC_TOKEN_ID,
    yo_token: address!("0x0000000f2eB9f69274678c76222B35eEc7588a65"),
};

pub const YO_USDT: YoAsset = YoAsset {
    chain: Chain::Ethereum,
    asset_token: ETHEREUM_USDT_TOKEN_ID,
    yo_token: address!("0xb9a7da9e90D3B428083BAe04b860faA6325b721e"),
};

pub fn supported_assets() -> &'static [YoAsset] {
    &[YO_USDC, YO_USDT]
}

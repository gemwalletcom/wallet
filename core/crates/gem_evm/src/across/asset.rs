use alloy_primitives::U256;
use primitives::{Asset, AssetId, EVMChain, EvmNativeCurrency};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Funding {
    Value,
    Token,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcrossAsset {
    pub asset_id: AssetId,
    pub scale: U256,
    pub funding: Funding,
}

impl AcrossAsset {
    pub fn from_asset(asset_id: &AssetId) -> Option<Self> {
        if asset_id.is_token() {
            return Some(Self {
                asset_id: asset_id.clone(),
                scale: U256::from(1),
                funding: Funding::Token,
            });
        }
        let chain = asset_id.chain;
        match EVMChain::from_chain(chain)?.native_currency() {
            EvmNativeCurrency::Wrapped(token) | EvmNativeCurrency::Token(token) => Some(Self {
                asset_id: AssetId::from_token(chain, token),
                scale: U256::from(1),
                funding: Funding::Value,
            }),
            EvmNativeCurrency::Mirrored { token, decimals } => {
                let exponent = u32::try_from(Asset::from_chain(chain).decimals).ok()?.checked_sub(decimals)?;
                Some(Self {
                    asset_id: AssetId::from_token(chain, token),
                    scale: U256::from(10).pow(U256::from(exponent)),
                    funding: Funding::Token,
                })
            }
            EvmNativeCurrency::None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{
        Chain,
        asset_constants::{ARC_USDC_ASSET_ID, ETHEREUM_USDC_ASSET_ID, ETHEREUM_WETH_ASSET_ID},
    };

    #[test]
    fn test_from_asset() {
        assert_eq!(
            AcrossAsset::from_asset(&Chain::Arc.as_asset_id()),
            Some(AcrossAsset {
                asset_id: ARC_USDC_ASSET_ID.clone(),
                scale: U256::from(10u64.pow(12)),
                funding: Funding::Token,
            })
        );
        assert_eq!(
            AcrossAsset::from_asset(&Chain::Ethereum.as_asset_id()),
            Some(AcrossAsset {
                asset_id: ETHEREUM_WETH_ASSET_ID.clone(),
                scale: U256::from(1),
                funding: Funding::Value,
            })
        );
        assert_eq!(AcrossAsset::from_asset(&ETHEREUM_USDC_ASSET_ID).unwrap().funding, Funding::Token);
        assert_eq!(AcrossAsset::from_asset(&Chain::Tempo.as_asset_id()), None);
        assert_eq!(AcrossAsset::from_asset(&Chain::Tron.as_asset_id()), None);
    }
}

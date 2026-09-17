use alloy_primitives::{Address, U256};
use gem_evm::uniswap::path::{BasePair, get_base_pair};
#[cfg(test)]
use primitives::Chain;
use primitives::{Asset, AssetId, EVMChain, EvmNativeCurrency};

use crate::{SwapperError, eth_address};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Funding {
    Value,
    RouterBalance,
    Permit2,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Protocol {
    V3,
    V4,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoutedAsset {
    pub address: Address,
    pub scale: U256,
    pub funding: Funding,
}

impl RoutedAsset {
    pub fn from_asset(asset_id: &AssetId, chain: EVMChain, protocol: Protocol) -> Result<Self, SwapperError> {
        if let Some(token_id) = &asset_id.token_id {
            return Ok(Self::unscaled(eth_address::parse_str(token_id)?, Funding::Permit2));
        }
        match chain.native_currency() {
            EvmNativeCurrency::Wrapped(wrapped) => {
                let address = match protocol {
                    Protocol::V3 => eth_address::parse_str(wrapped)?,
                    Protocol::V4 => Address::ZERO,
                };
                Ok(Self::unscaled(address, Funding::Value))
            }
            EvmNativeCurrency::Token(token) => Ok(Self::unscaled(eth_address::parse_str(token)?, Funding::Permit2)),
            EvmNativeCurrency::Mirrored { token, decimals } => {
                let native_decimals = u32::try_from(Asset::from_chain(chain.to_chain()).decimals).map_err(|_| SwapperError::NotSupportedChain)?;
                let exponent = native_decimals.checked_sub(decimals).ok_or(SwapperError::NotSupportedChain)?;
                Ok(Self {
                    address: eth_address::parse_str(token)?,
                    scale: U256::from(10u64).pow(U256::from(exponent)),
                    funding: Funding::RouterBalance,
                })
            }
            EvmNativeCurrency::None => Err(SwapperError::NotSupportedChain),
        }
    }

    fn unscaled(address: Address, funding: Funding) -> Self {
        Self {
            address,
            scale: U256::from(1),
            funding,
        }
    }
}

pub fn base_pair(chain: EVMChain, protocol: Protocol) -> Option<BasePair> {
    let native = RoutedAsset::from_asset(&AssetId::from_chain(chain.to_chain()), chain, protocol);
    get_base_pair(&chain, native.map(|asset| asset.address).unwrap_or(Address::ZERO))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mirrored_native_scale() {
        let arc = RoutedAsset::from_asset(&AssetId::from_chain(Chain::Arc), EVMChain::Arc, Protocol::V4).unwrap();
        assert_eq!(arc.scale, U256::from(1_000_000_000_000u64));
        assert_eq!(arc.funding, Funding::RouterBalance);
        assert!(base_pair(EVMChain::Arc, Protocol::V4).unwrap().stables.is_empty());
        assert!(RoutedAsset::from_asset(&AssetId::from_chain(Chain::Tempo), EVMChain::Tempo, Protocol::V4).is_err());
    }
}

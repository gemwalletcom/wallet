use crate::{AssetId, Chain};

const PERPETUAL_PREFIX: &str = "perpetual";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HyperCoreToken {
    Spot { symbol: String, contract: Option<String>, index: Option<i32> },
    Perpetual { symbol: String },
}

impl HyperCoreToken {
    pub fn from_asset_id(asset_id: &AssetId) -> Option<Self> {
        if asset_id.chain != Chain::HyperCore {
            return None;
        }
        match AssetId::decode_token_id(asset_id.token_id.as_ref()?).as_slice() {
            [prefix, symbol] if prefix == PERPETUAL_PREFIX => Some(Self::Perpetual { symbol: symbol.clone() }),
            [symbol, rest @ ..] if !symbol.is_empty() => Some(Self::Spot {
                symbol: symbol.clone(),
                contract: rest.first().filter(|contract| !contract.is_empty()).cloned(),
                index: rest.get(1).and_then(|index| index.parse().ok()),
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset_constants::{HYPERCORE_PERPETUAL_USDC_ASSET_ID, HYPERCORE_SPOT_HYPE_ASSET_ID};

    #[test]
    fn test_from_asset_id() {
        assert_eq!(
            HyperCoreToken::from_asset_id(&HYPERCORE_SPOT_HYPE_ASSET_ID),
            Some(HyperCoreToken::Spot {
                symbol: "HYPE".to_string(),
                contract: Some("0x0d01dc56dcaaca66ad901c959b4011ec".to_string()),
                index: Some(150),
            })
        );
        assert_eq!(HyperCoreToken::from_asset_id(&HYPERCORE_PERPETUAL_USDC_ASSET_ID), Some(HyperCoreToken::Perpetual { symbol: "USDC".to_string() }));
        assert_eq!(HyperCoreToken::from_asset_id(&AssetId::from_chain(Chain::HyperCore)), None);
    }
}

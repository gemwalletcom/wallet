use std::str::FromStr;

use num_bigint::BigInt;
use primitives::{Asset, AssetId, BITCOINCASH_PREFIX, Chain};

use super::chain::ChainName;

const THORCHAIN_DECIMALS: u32 = 8;

pub fn value_from(value: &str, decimals: u32) -> BigInt {
    let value = BigInt::from_str(value).unwrap_or_default();
    let factor = BigInt::from(10).pow(decimals.abs_diff(THORCHAIN_DECIMALS));
    if decimals > THORCHAIN_DECIMALS { value / factor } else { value * factor }
}

pub fn value_to(value: &str, decimals: u32) -> BigInt {
    let value = BigInt::from_str(value).unwrap_or_default();
    let factor = BigInt::from(10).pow(decimals.abs_diff(THORCHAIN_DECIMALS));
    if decimals > THORCHAIN_DECIMALS { value * factor } else { value / factor }
}

#[derive(Clone, Debug)]
pub struct THORChainAsset {
    pub symbol: String,
    pub chain: ChainName,
    pub token_id: Option<String>,
    pub decimals: u32,
}

impl THORChainAsset {
    pub fn quote_asset_name(&self) -> String {
        format!("{}.{}", self.chain.long_name(), self.symbol)
    }

    pub fn is_token(&self) -> bool {
        self.token_id.is_some()
    }

    pub fn use_evm_router(&self) -> bool {
        self.is_token() && self.chain.is_evm_chain()
    }

    pub fn from_id(asset_id: &AssetId) -> Option<THORChainAsset> {
        let chain = ChainName::from_chain(asset_id.chain)?;
        if let Some(token_id) = &asset_id.token_id {
            THORChainAsset::from(chain, token_id)
        } else {
            let asset = Asset::from_chain(asset_id.chain);
            Some(THORChainAsset {
                symbol: asset.symbol,
                chain,
                token_id: None,
                decimals: asset.decimals,
            })
        }
    }

    pub fn from_asset_id(asset_id: &str) -> Option<THORChainAsset> {
        THORChainAsset::from_id(&AssetId::new(asset_id)?)
    }

    pub fn from(chain: ChainName, token_id: &str) -> Option<THORChainAsset> {
        chain.token_asset(token_id).map(|asset| THORChainAsset {
            symbol: asset.symbol,
            chain,
            token_id: asset.id.token_id,
            decimals: asset.decimals,
        })
    }

    pub fn swap_memo(&self, asset_name: &str, destination_address: String, minimum: i64, interval: i64, quantity: i64, fee_address: String, bps: u32) -> String {
        let address = match self.chain.chain() {
            Chain::BitcoinCash => destination_address.trim_prefix(BITCOINCASH_PREFIX),
            _ => destination_address.as_str(),
        };
        format!("=:{asset_name}:{address}:{minimum}/{interval}/{quantity}:{fee_address}:{bps}")
    }
}

#[cfg(test)]
mod tests {
    use primitives::{
        Chain,
        asset_constants::{ETHEREUM_USDT_ASSET_ID, ETHEREUM_USDT_TOKEN_ID, SMARTCHAIN_USDT_TOKEN_ID, THORCHAIN_TCY_ASSET_ID, TRON_USDT_ASSET_ID},
    };

    use super::*;

    #[test]
    fn test_thorchain_name_token() {
        let test_cases = vec![(Chain::Ethereum, ETHEREUM_USDT_TOKEN_ID, "USDT", 6), (Chain::SmartChain, SMARTCHAIN_USDT_TOKEN_ID, "USDT", 18)];

        for (chain, token_id, expected_symbol, expected_decimals) in test_cases {
            let chain = ChainName::from_chain(chain).unwrap();
            let asset = THORChainAsset::from(chain, token_id);
            assert!(asset.is_some());
            let asset = asset.unwrap();
            assert_eq!(asset.symbol, expected_symbol);
            assert_eq!(asset.decimals, expected_decimals);
        }
    }

    #[test]
    fn test_thorchain_asset_name() {
        let asset_with_token = THORChainAsset {
            symbol: "USDT".to_string(),
            chain: ChainName::from_chain(Chain::Ethereum).unwrap(),
            token_id: Some(ETHEREUM_USDT_TOKEN_ID.to_string()),
            decimals: 6,
        };
        assert_eq!(asset_with_token.quote_asset_name(), "ETH.USDT");

        let asset_with_token = THORChainAsset {
            symbol: "USDT".to_string(),
            chain: ChainName::from_chain(Chain::SmartChain).unwrap(),
            token_id: Some(SMARTCHAIN_USDT_TOKEN_ID.to_string()),
            decimals: 6,
        };
        assert_eq!(asset_with_token.quote_asset_name(), "BSC.USDT");

        let asset_without_token = THORChainAsset {
            symbol: "RUNE".to_string(),
            chain: ChainName::from_chain(Chain::Thorchain).unwrap(),
            token_id: None,
            decimals: 8,
        };
        assert_eq!(asset_without_token.quote_asset_name(), "THOR.RUNE");

        let zcash = THORChainAsset::from_asset_id(Chain::Zcash.as_ref()).unwrap();
        assert_eq!(zcash.quote_asset_name(), "ZEC.ZEC");
    }

    #[test]
    fn test_swap_memo() {
        let destination_address = "0x1234567890abcdef".to_string();
        let fee_address = "g1".to_string();
        let bps = 50;

        assert_eq!(
            THORChainAsset::from_asset_id(Chain::SmartChain.as_ref())
                .unwrap()
                .swap_memo("s", destination_address.clone(), 0, 1, 0, fee_address.clone(), bps),
            "=:s:0x1234567890abcdef:0/1/0:g1:50"
        );
        assert_eq!(
            THORChainAsset::from_asset_id(Chain::Ethereum.as_ref()).unwrap().swap_memo("e", destination_address.clone(), 0, 1, 0, fee_address.clone(), bps),
            "=:e:0x1234567890abcdef:0/1/0:g1:50"
        );
        assert_eq!(
            THORChainAsset::from_asset_id(Chain::Doge.as_ref()).unwrap().swap_memo("d", destination_address.clone(), 0, 1, 0, fee_address.clone(), bps),
            "=:d:0x1234567890abcdef:0/1/0:g1:50"
        );
        assert_eq!(
            THORChainAsset::from_id(&ETHEREUM_USDT_ASSET_ID).unwrap().swap_memo("ETH.USDT", destination_address.clone(), 0, 1, 0, fee_address.clone(), bps),
            "=:ETH.USDT:0x1234567890abcdef:0/1/0:g1:50"
        );
        assert_eq!(
            THORChainAsset::from_asset_id(Chain::BitcoinCash.as_ref())
                .unwrap()
                .swap_memo("c", "bitcoincash:qpcns7lget89x9km0t8ry5fk52e8lhl53q0a64gd65".to_string(), 0, 1, 0, fee_address.clone(), bps),
            "=:c:qpcns7lget89x9km0t8ry5fk52e8lhl53q0a64gd65:0/1/0:g1:50"
        );
        assert_eq!(
            THORChainAsset::from_asset_id(&THORCHAIN_TCY_ASSET_ID.to_string())
                .unwrap()
                .swap_memo("THOR.TCY", destination_address, 0, 1, 0, fee_address.clone(), bps),
            "=:THOR.TCY:0x1234567890abcdef:0/1/0:g1:50"
        );
        assert_eq!(
            THORChainAsset::from_asset_id(Chain::Zcash.as_ref())
                .unwrap()
                .swap_memo("z", "t1Ku2KLyndDPsR32jwnrTMd3yvi9tfFP8ML".to_string(), 0, 1, 0, fee_address, bps),
            "=:z:t1Ku2KLyndDPsR32jwnrTMd3yvi9tfFP8ML:0/1/0:g1:50"
        );
    }

    #[test]
    fn test_value_from() {
        assert_eq!(value_from("1000000000000000000", 18), BigInt::from(100000000));
        assert_eq!(value_from("1000000000", 10), BigInt::from(10000000));
        assert_eq!(value_from("1000000000", 6), BigInt::from_str("100000000000").unwrap());
        assert_eq!(value_from("1000000000", 8), BigInt::from(1000000000));
    }

    #[test]
    fn test_value_to() {
        assert_eq!(value_to("2509674", 18), BigInt::from_str("25096740000000000").unwrap());
        assert_eq!(value_to("10000000", 10), BigInt::from(1000000000));
        assert_eq!(value_to("79158429", 6), BigInt::from(791584));
        assert_eq!(value_to("160661010", 8), BigInt::from(160661010));
    }

    #[test]
    fn test_tron_usdt_memo() {
        let tron_destination = "TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb".to_string();
        let fee_address = "g1".to_string();
        let bps = 50;

        let asset = THORChainAsset::from_id(&TRON_USDT_ASSET_ID);

        assert!(asset.is_some(), "TRON USDT asset should be recognized");

        let memo = asset.unwrap().swap_memo("TRON.USDT", tron_destination, 0, 1, 0, fee_address, bps);

        assert_eq!(memo, "=:TRON.USDT:TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb:0/1/0:g1:50");
    }
}

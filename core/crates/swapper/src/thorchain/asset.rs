use std::str::FromStr;

use num_bigint::{BigInt, BigUint};
use primitives::{Asset, AssetId, BITCOINCASH_PREFIX, Chain};

use super::{THORChainNetwork, chain::ChainName};
use crate::fees::subtract_bps;

const THORCHAIN_DECIMALS: i32 = 8;
const MEMO_AMOUNT_SIGNIFICANT_DIGITS: usize = 5;

pub fn value_from(value: &str, decimals: i32) -> BigInt {
    let value = BigInt::from_str(value).unwrap_or_default();
    let diff = decimals - THORCHAIN_DECIMALS;
    let factor = BigInt::from(10).pow(diff.unsigned_abs());
    if diff > 0 { value / factor } else { value * factor }
}

pub fn value_to(value: &str, decimals: i32) -> BigInt {
    let value = BigInt::from_str(value).unwrap_or_default();
    let diff = decimals - THORCHAIN_DECIMALS;
    let factor = BigInt::from(10).pow(diff.unsigned_abs());
    if diff > 0 { value * factor } else { value / factor }
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

    pub fn from_id(network: THORChainNetwork, asset_id: &AssetId) -> Option<THORChainAsset> {
        let chain = ChainName::from_chain(network, asset_id.chain)?;
        if let Some(token_id) = &asset_id.token_id {
            THORChainAsset::from(chain, token_id)
        } else {
            let asset = Asset::from_chain(asset_id.chain);
            Some(THORChainAsset {
                symbol: asset.symbol,
                chain,
                token_id: None,
                decimals: asset.decimals as u32,
            })
        }
    }

    pub fn from_asset_id(network: THORChainNetwork, asset_id: &str) -> Option<THORChainAsset> {
        THORChainAsset::from_id(network, &AssetId::new(asset_id)?)
    }

    pub fn from(chain: ChainName, token_id: &str) -> Option<THORChainAsset> {
        chain.token_asset(token_id).map(|asset| THORChainAsset {
            symbol: asset.symbol,
            chain,
            token_id: asset.id.token_id,
            decimals: asset.decimals as u32,
        })
    }

    pub fn trade_limit(&self, value: &BigUint, slippage_bps: u32) -> BigUint {
        value_from(&subtract_bps(value, slippage_bps).to_string(), self.decimals as i32).magnitude().clone()
    }

    pub fn swap_memo(&self, asset_name: &str, destination_address: String, limit: &BigUint, interval: i64, quantity: i64, fee_address: String, bps: u32) -> String {
        let address = match self.chain.chain() {
            Chain::BitcoinCash => destination_address.strip_prefix(BITCOINCASH_PREFIX).unwrap_or(&destination_address),
            _ => destination_address.as_str(),
        };
        let limit = memo_amount(limit);
        format!("=:{asset_name}:{address}:{limit}/{interval}/{quantity}:{fee_address}:{bps}")
    }
}

fn memo_amount(limit: &BigUint) -> String {
    let digits = limit.to_string();
    match digits.len().saturating_sub(MEMO_AMOUNT_SIGNIFICANT_DIGITS) {
        0 => digits,
        exponent => format!("{}e{exponent}", &digits[..MEMO_AMOUNT_SIGNIFICANT_DIGITS]),
    }
}

#[cfg(test)]
mod tests {
    use primitives::{
        Chain,
        asset_constants::{ARBITRUM_USDC_TOKEN_ID, ARBITRUM_USDT_TOKEN_ID, ETHEREUM_USDT_ASSET_ID, ETHEREUM_USDT_TOKEN_ID, SMARTCHAIN_USDT_TOKEN_ID, THORCHAIN_TCY_ASSET_ID, TRON_USDT_ASSET_ID},
    };

    use super::*;

    #[test]
    fn test_thorchain_name_token() {
        let test_cases = vec![
            (THORChainNetwork::Thorchain, Chain::Ethereum, ETHEREUM_USDT_TOKEN_ID, "USDT", 6),
            (THORChainNetwork::Thorchain, Chain::SmartChain, SMARTCHAIN_USDT_TOKEN_ID, "USDT", 18),
            (THORChainNetwork::Mayachain, Chain::Arbitrum, ARBITRUM_USDC_TOKEN_ID, "USDC", 6),
            (THORChainNetwork::Mayachain, Chain::Arbitrum, ARBITRUM_USDT_TOKEN_ID, "USDT", 6),
        ];

        for (network, chain, token_id, expected_symbol, expected_decimals) in test_cases {
            let chain = ChainName::from_chain(network, chain).unwrap();
            let asset = THORChainAsset::from(chain, token_id);
            assert!(asset.is_some());
            let asset = asset.unwrap();
            assert_eq!(asset.symbol, expected_symbol);
            assert_eq!(asset.decimals, expected_decimals);
        }

        assert!(THORChainAsset::from_id(THORChainNetwork::Mayachain, &THORCHAIN_TCY_ASSET_ID).is_none());
    }

    #[test]
    fn test_thorchain_asset_name() {
        let asset_with_token = THORChainAsset {
            symbol: "USDT".to_string(),
            chain: ChainName::from_chain(THORChainNetwork::Thorchain, Chain::Ethereum).unwrap(),
            token_id: Some(ETHEREUM_USDT_TOKEN_ID.to_string()),
            decimals: 6,
        };
        assert_eq!(asset_with_token.quote_asset_name(), "ETH.USDT");

        let asset_with_token = THORChainAsset {
            symbol: "USDT".to_string(),
            chain: ChainName::from_chain(THORChainNetwork::Thorchain, Chain::SmartChain).unwrap(),
            token_id: Some(SMARTCHAIN_USDT_TOKEN_ID.to_string()),
            decimals: 6,
        };
        assert_eq!(asset_with_token.quote_asset_name(), "BSC.USDT");

        let asset_without_token = THORChainAsset {
            symbol: "RUNE".to_string(),
            chain: ChainName::from_chain(THORChainNetwork::Thorchain, Chain::Thorchain).unwrap(),
            token_id: None,
            decimals: 8,
        };
        assert_eq!(asset_without_token.quote_asset_name(), "THOR.RUNE");

        let zcash = THORChainAsset::from_asset_id(THORChainNetwork::Mayachain, Chain::Zcash.as_ref()).unwrap();
        assert_eq!(zcash.quote_asset_name(), "ZEC.ZEC");

        let dash = THORChainAsset::from_asset_id(THORChainNetwork::Mayachain, Chain::Dash.as_ref()).unwrap();
        assert_eq!(dash.quote_asset_name(), "DASH.DASH");

        let cardano = THORChainAsset::from_asset_id(THORChainNetwork::Mayachain, Chain::Cardano.as_ref()).unwrap();
        assert_eq!(cardano.quote_asset_name(), "ADA.ADA");

        let arbitrum = THORChainAsset::from_asset_id(THORChainNetwork::Mayachain, Chain::Arbitrum.as_ref()).unwrap();
        assert_eq!(arbitrum.quote_asset_name(), "ARB.ETH");
    }

    #[test]
    fn test_swap_memo() {
        let destination_address = "0x1234567890abcdef".to_string();
        let fee_address = "g1".to_string();
        let bps = 50;

        assert_eq!(
            THORChainAsset::from_asset_id(THORChainNetwork::Thorchain, Chain::SmartChain.as_ref())
                .unwrap()
                .swap_memo("s", destination_address.clone(), &BigUint::ZERO, 1, 0, fee_address.clone(), bps),
            "=:s:0x1234567890abcdef:0/1/0:g1:50"
        );
        assert_eq!(
            THORChainAsset::from_asset_id(THORChainNetwork::Thorchain, Chain::Ethereum.as_ref())
                .unwrap()
                .swap_memo("e", destination_address.clone(), &BigUint::ZERO, 1, 0, fee_address.clone(), bps),
            "=:e:0x1234567890abcdef:0/1/0:g1:50"
        );
        assert_eq!(
            THORChainAsset::from_asset_id(THORChainNetwork::Thorchain, Chain::Doge.as_ref())
                .unwrap()
                .swap_memo("d", destination_address.clone(), &BigUint::ZERO, 1, 0, fee_address.clone(), bps),
            "=:d:0x1234567890abcdef:0/1/0:g1:50"
        );
        assert_eq!(
            THORChainAsset::from_id(THORChainNetwork::Thorchain, &ETHEREUM_USDT_ASSET_ID)
                .unwrap()
                .swap_memo("ETH.USDT", destination_address.clone(), &BigUint::ZERO, 1, 0, fee_address.clone(), bps),
            "=:ETH.USDT:0x1234567890abcdef:0/1/0:g1:50"
        );
        assert_eq!(
            THORChainAsset::from_asset_id(THORChainNetwork::Thorchain, Chain::BitcoinCash.as_ref()).unwrap().swap_memo(
                "c",
                "bitcoincash:qpcns7lget89x9km0t8ry5fk52e8lhl53q0a64gd65".to_string(),
                &BigUint::ZERO,
                1,
                0,
                fee_address.clone(),
                bps
            ),
            "=:c:qpcns7lget89x9km0t8ry5fk52e8lhl53q0a64gd65:0/1/0:g1:50"
        );
        assert_eq!(
            THORChainAsset::from_asset_id(THORChainNetwork::Thorchain, &THORCHAIN_TCY_ASSET_ID.to_string())
                .unwrap()
                .swap_memo("THOR.TCY", destination_address, &BigUint::ZERO, 1, 0, fee_address.clone(), bps),
            "=:THOR.TCY:0x1234567890abcdef:0/1/0:g1:50"
        );
        assert_eq!(
            THORChainAsset::from_asset_id(THORChainNetwork::Mayachain, Chain::Zcash.as_ref())
                .unwrap()
                .swap_memo("z", "t1Ku2KLyndDPsR32jwnrTMd3yvi9tfFP8ML".to_string(), &BigUint::ZERO, 1, 0, fee_address.clone(), bps),
            "=:z:t1Ku2KLyndDPsR32jwnrTMd3yvi9tfFP8ML:0/1/0:g1:50"
        );
        assert_eq!(
            THORChainAsset::from_asset_id(THORChainNetwork::Mayachain, Chain::Cardano.as_ref()).unwrap().swap_memo(
                "aa",
                "addr1q92cmkgzv9h4e5q7mnrzsuxtgayvg4qr7y3gyx97ukmz3dfx7r9fu73vqn25377ke6r0xk97zw07dqr9y5myxlgadl2s0dgke5".to_string(),
                &BigUint::ZERO,
                1,
                0,
                fee_address,
                bps
            ),
            "=:aa:addr1q92cmkgzv9h4e5q7mnrzsuxtgayvg4qr7y3gyx97ukmz3dfx7r9fu73vqn25377ke6r0xk97zw07dqr9y5myxlgadl2s0dgke5:0/1/0:g1:50"
        );
    }

    #[test]
    fn test_trade_limit() {
        let bitcoin = THORChainAsset::from_asset_id(THORChainNetwork::Thorchain, Chain::Bitcoin.as_ref()).unwrap();
        let ethereum = THORChainAsset::from_asset_id(THORChainNetwork::Thorchain, Chain::Ethereum.as_ref()).unwrap();
        let usdt = THORChainAsset::from_id(THORChainNetwork::Thorchain, &ETHEREUM_USDT_ASSET_ID).unwrap();

        assert_eq!(bitcoin.trade_limit(&BigUint::from(100_000_000u64), 50), BigUint::from(99_500_000u64), "1 BTC less 0.5%, already in 8 decimals");
        assert_eq!(ethereum.trade_limit(&BigUint::from(10u64.pow(18)), 50), BigUint::from(99_500_000u64), "1 ETH less 0.5%, scaled down from 18 decimals");
        assert_eq!(usdt.trade_limit(&BigUint::from(1_000_000_000u64), 50), BigUint::from(99_500_000_000u64), "1000 USDT less 0.5%, scaled up from 6 decimals");
        assert_eq!(bitcoin.trade_limit(&BigUint::from(1u64), 50), BigUint::ZERO, "a dust quote has no limit to enforce");
    }

    #[test]
    fn test_memo_amount() {
        assert_eq!(memo_amount(&BigUint::ZERO), "0");
        assert_eq!(memo_amount(&BigUint::from(12345u32)), "12345");
        assert_eq!(memo_amount(&BigUint::from(123456u32)), "12345e1", "anything longer keeps five significant digits, rounded down");
        assert_eq!(memo_amount(&BigUint::from(99_500_000u64)), "99500e3", "and is exact when the rest is zeros");
        assert_eq!(memo_amount(&BigUint::from(u128::MAX)), "34028e34", "any amount takes at most eight bytes of a Bitcoin memo");
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

        let asset = THORChainAsset::from_id(THORChainNetwork::Thorchain, &TRON_USDT_ASSET_ID);

        assert!(asset.is_some(), "TRON USDT asset should be recognized");

        let memo = asset.unwrap().swap_memo("TRON.USDT", tron_destination, &BigUint::ZERO, 1, 0, fee_address, bps);

        assert_eq!(memo, "=:TRON.USDT:TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb:0/1/0:g1:50");
    }
}

use alloy_primitives::Address;
use primitives::chain_aptos::is_fungible_asset_token_id;
use primitives::{AssetId, Chain};
use std::str::FromStr;

const APTOS_COIN_OBJECT_ID: &str = "0xa";

pub fn is_native_token_id(chain: Chain, token_id: &str) -> bool {
    let Some(denom) = chain.as_denom() else {
        return false;
    };
    match chain {
        Chain::Sui => is_same_move_id(token_id, denom),
        Chain::Aptos => is_same_move_id(token_id, denom) || is_same_move_id(token_id, APTOS_COIN_OBJECT_ID),
        _ => token_id == denom,
    }
}

fn is_same_move_id(left: &str, right: &str) -> bool {
    let left = AssetId::decode_token_id(left);
    let right = AssetId::decode_token_id(right);
    left.len() == right.len() && is_same_move_address(&left[0], &right[0]) && left[1..] == right[1..]
}

fn is_same_move_address(left: &str, right: &str) -> bool {
    let digits = |address: &str| address.trim_start_matches("0x").trim_start_matches('0').to_ascii_lowercase();
    digits(left) == digits(right)
}

fn is_move_coin_type(token_id: &str) -> bool {
    token_id.starts_with("0x") && AssetId::decode_token_id(token_id).len() == 3
}

pub fn format_token_id(chain: Chain, token_id: String) -> Option<String> {
    if is_native_token_id(chain, &token_id) {
        return None;
    }
    match chain {
        Chain::Ethereum
        | Chain::SmartChain
        | Chain::Polygon
        | Chain::Arbitrum
        | Chain::Optimism
        | Chain::Base
        | Chain::AvalancheC
        | Chain::OpBNB
        | Chain::Fantom
        | Chain::Gnosis
        | Chain::Manta
        | Chain::Blast
        | Chain::ZkSync
        | Chain::Linea
        | Chain::Mantle
        | Chain::Celo
        | Chain::World
        | Chain::Sonic
        | Chain::SeiEvm
        | Chain::Abstract
        | Chain::Berachain
        | Chain::Ink
        | Chain::Unichain
        | Chain::Hyperliquid
        | Chain::HyperCore
        | Chain::Plasma
        | Chain::Monad
        | Chain::XLayer
        | Chain::Robinhood
        | Chain::Stable
        | Chain::Tempo
        | Chain::Arc => Address::from_str(&token_id).ok().map(|address| address.to_checksum(None)),
        Chain::Solana | Chain::Ton | Chain::Near => Some(token_id),
        Chain::Tron => (token_id.len() == 34 && token_id.starts_with('T')).then_some(token_id),
        Chain::Xrp => {
            if let Some((_, address)) = token_id.split_once('.')
                && address.starts_with('r')
            {
                return Some(address.to_string());
            }
            token_id.starts_with('r').then_some(token_id)
        }
        Chain::Algorand => token_id.parse::<i32>().ok().map(|token_id| token_id.to_string()),
        Chain::Sui => (token_id.len() >= 64 && is_move_coin_type(&token_id)).then_some(token_id),
        Chain::Stellar => {
            if let Some((issuer, symbol)) = token_id.split_once("::") {
                (issuer.len() == 56 && issuer.starts_with('G') && !symbol.is_empty()).then_some(token_id)
            } else {
                None
            }
        }
        Chain::Aptos => (is_move_coin_type(&token_id) || is_fungible_asset_token_id(&token_id)).then_some(token_id),
        Chain::Bitcoin
        | Chain::BitcoinCash
        | Chain::Litecoin
        | Chain::Thorchain
        | Chain::Mayachain
        | Chain::Cosmos
        | Chain::Osmosis
        | Chain::Celestia
        | Chain::Doge
        | Chain::Dash
        | Chain::Zcash
        | Chain::Injective
        | Chain::Noble
        | Chain::Sei
        | Chain::Polkadot
        | Chain::Cardano => None,
    }
}

#[cfg(test)]
mod tests {
    use primitives::asset_constants::{APTOS_USDC_TOKEN_ID, STELLAR_USDC_TOKEN_ID, SUI_WAL_TOKEN_ID, TRON_USDT_TOKEN_ID};

    use super::*;

    #[test]
    fn test_format_token_id_ethereum() {
        let chain = Chain::Ethereum;

        let valid_token_id = "0x1234567890abcdef1234567890abcdef12345678".to_string();
        let formatted_valid_token_id = format_token_id(chain, valid_token_id);

        assert_eq!(formatted_valid_token_id.unwrap(), "0x1234567890AbcdEF1234567890aBcdef12345678");
        assert_eq!(format_token_id(chain, "0x123".to_string()), None);
    }

    #[test]
    fn test_format_token_id_sui() {
        let chain = Chain::Sui;
        assert_eq!(format_token_id(chain, "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string()), None);
        assert_eq!(format_token_id(chain, "0x2::sui::SUI".to_string()), None);
        assert_eq!(format_token_id(chain, SUI_WAL_TOKEN_ID.to_string()), Some(SUI_WAL_TOKEN_ID.to_string()));
        assert_eq!(format_token_id(chain, "0x0000000000000000000000000000000000000000000000000000000000000002::sui::SUI".to_string()), None);
    }

    #[test]
    fn test_is_native_token_id() {
        assert!(is_native_token_id(Chain::Sui, "0x2::sui::SUI"));
        assert!(is_native_token_id(Chain::Sui, "0x0000000000000000000000000000000000000000000000000000000000000002::sui::SUI"));
        assert!(!is_native_token_id(Chain::Sui, SUI_WAL_TOKEN_ID));
        assert!(is_native_token_id(Chain::Aptos, "0x1::aptos_coin::AptosCoin"));
        assert!(is_native_token_id(Chain::Aptos, "0x0000000000000000000000000000000000000000000000000000000000000001::aptos_coin::AptosCoin"));
        assert!(is_native_token_id(Chain::Aptos, "0xa"));
        assert!(is_native_token_id(Chain::Aptos, "0x000000000000000000000000000000000000000000000000000000000000000A"));
        assert!(!is_native_token_id(Chain::Aptos, APTOS_USDC_TOKEN_ID));
        assert!(is_native_token_id(Chain::Cosmos, "uatom"));
        assert!(!is_native_token_id(Chain::Ethereum, "0x0000000000000000000000000000000000000000"));
    }

    #[test]
    fn test_format_token_id_aptos() {
        let chain = Chain::Aptos;
        let coin_type = "0x159df6b7689437016108a019fd5bef736bac692b6d4a1f10c941f6fbb9a74ca6::oft::CakeOFT".to_string();

        assert_eq!(format_token_id(chain, APTOS_USDC_TOKEN_ID.to_string()), Some(APTOS_USDC_TOKEN_ID.to_string()));
        assert_eq!(format_token_id(chain, coin_type.clone()), Some(coin_type));
        assert_eq!(format_token_id(chain, "0x1::aptos_coin::AptosCoin".to_string()), None);
        assert_eq!(format_token_id(chain, "0x000000000000000000000000000000000000000000000000000000000000000a".to_string()), None);
        assert_eq!(format_token_id(chain, "0xa".to_string()), None);
        assert_eq!(format_token_id(chain, "USDC".to_string()), None);
    }

    #[test]
    fn test_format_token_id_tron() {
        let chain = Chain::Tron;

        let valid_token_id = TRON_USDT_TOKEN_ID.to_string();
        let formatted_valid_token_id = format_token_id(chain, valid_token_id.clone());
        assert_eq!(formatted_valid_token_id, Some(valid_token_id));

        assert_eq!(format_token_id(chain, "1234567890123456789012345678901234".to_string()), None);
        assert_eq!(format_token_id(chain, "T123".to_string()), None);
    }

    #[test]
    fn test_format_token_id_xrp() {
        let chain = Chain::Xrp;

        assert_eq!(
            format_token_id(chain, "534F4C4F00000000000000000000000000000000.rsoLo2S1kiGeCcn6hCUXVrCpGMWLrRrLZz".to_string()),
            Some("rsoLo2S1kiGeCcn6hCUXVrCpGMWLrRrLZz".to_string())
        );
        assert_eq!(format_token_id(chain, "rsoLo2S1kiGeCcn6hCUXVrCpGMWLrRrLZz".to_string()), Some("rsoLo2S1kiGeCcn6hCUXVrCpGMWLrRrLZz".to_string()));
    }

    #[test]
    fn test_format_token_id_stellar() {
        let chain = Chain::Stellar;

        assert_eq!(format_token_id(chain, STELLAR_USDC_TOKEN_ID.to_string()), Some(STELLAR_USDC_TOKEN_ID.to_string()));
        assert_eq!(format_token_id(chain, "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN".to_string()), None);
        assert_eq!(format_token_id(chain, "invalid".to_string()), None);
        assert_eq!(format_token_id(chain, "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN::".to_string()), None);
    }
}

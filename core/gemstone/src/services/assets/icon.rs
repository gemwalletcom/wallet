use primitives::known_assets::{USDC_ASSETS, USDT_ASSETS};
use primitives::{Asset, AssetId, Chain};

use crate::config::chain::{badge_chain, icon_chain, is_ether_layer2};
use crate::config::image::GemImage;

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemAssetIcon {
    pub image: GemAssetIconImage,
    pub badge: Option<Chain>,
    pub placeholder: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemLocalTokenIcon {
    Usdt,
    Usdc,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum GemAssetIconImage {
    Local { chain: Chain },
    LocalToken { token: GemLocalTokenIcon },
    Remote { url: String },
}

pub fn asset_icon(asset_id: &AssetId) -> GemAssetIcon {
    let icon_asset_id = icon_asset_id(asset_id);
    let local_token = local_token_icon(asset_id);
    let image = if let Some(token) = local_token {
        GemAssetIconImage::LocalToken { token }
    } else if icon_asset_id.is_native() {
        GemAssetIconImage::Local { chain: icon_chain(icon_asset_id.chain) }
    } else {
        GemAssetIconImage::Remote {
            url: GemImage::Asset { asset_id: icon_asset_id }.url(),
        }
    };
    let badge = match asset_id.is_native() && local_token.is_none() {
        true => badge_chain(asset_id.chain),
        false => Some(icon_chain(asset_id.chain)),
    };
    GemAssetIcon {
        image,
        badge,
        placeholder: asset_id.chain.default_asset_type().map(|asset_type| asset_type.as_ref().to_string()),
    }
}

fn icon_asset_id(asset_id: &AssetId) -> AssetId {
    if let Some(coin) = perpetual_coin(asset_id) {
        return Chain::all().into_iter().find(|chain| Asset::from_chain(*chain).symbol == coin).map(AssetId::from_chain).unwrap_or_else(|| asset_id.clone());
    }
    if asset_id.is_native() && is_ether_layer2(asset_id.chain) {
        return AssetId::from_chain(Chain::Ethereum);
    }
    asset_id.clone()
}

fn local_token_icon(asset_id: &AssetId) -> Option<GemLocalTokenIcon> {
    if USDT_ASSETS.iter().any(|asset| asset.id == *asset_id) {
        return Some(GemLocalTokenIcon::Usdt);
    }
    if USDC_ASSETS.iter().any(|asset| asset.id == *asset_id) {
        return Some(GemLocalTokenIcon::Usdc);
    }
    None
}

fn perpetual_coin(asset_id: &AssetId) -> Option<String> {
    let ids = AssetId::decode_token_id(asset_id.token_id.as_deref()?);
    (asset_id.chain == Chain::HyperCore && ids.first().is_some_and(|kind| kind == "perpetual")).then(|| ids.get(1).cloned())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_hypercore::models::metadata::perpetual_asset_id;
    use primitives::EVMChain;

    #[test]
    fn test_native_assets_draw_their_coin_and_badge_only_on_ethereum_layer2() {
        assert_eq!(
            asset_icon(&AssetId::from_chain(Chain::Ethereum)),
            GemAssetIcon {
                image: GemAssetIconImage::Local { chain: Chain::Ethereum },
                badge: None,
                placeholder: Some("ERC20".to_string())
            }
        );
        assert_eq!(
            asset_icon(&AssetId::from_chain(Chain::Base)),
            GemAssetIcon {
                image: GemAssetIconImage::Local { chain: Chain::Ethereum },
                badge: Some(Chain::Base),
                placeholder: Some("ERC20".to_string())
            }
        );
        assert_eq!(
            asset_icon(&AssetId::from_chain(Chain::Robinhood)),
            GemAssetIcon {
                image: GemAssetIconImage::Local { chain: Chain::Ethereum },
                badge: Some(Chain::Robinhood),
                placeholder: Some("ERC20".to_string())
            }
        );
        assert_eq!(
            asset_icon(&AssetId::from_chain(Chain::SeiEvm)),
            GemAssetIcon {
                image: GemAssetIconImage::Local { chain: Chain::Sei },
                badge: None,
                placeholder: Some("ERC20".to_string())
            }
        );
        assert_eq!(
            asset_icon(&AssetId::from_chain(Chain::OpBNB)),
            GemAssetIcon {
                image: GemAssetIconImage::Local { chain: Chain::OpBNB },
                badge: None,
                placeholder: Some("BEP20".to_string())
            }
        );
    }

    #[test]
    fn test_layer2_coins_that_are_not_ether_draw_their_own_logo_without_a_badge() {
        for chain in [Chain::Celo, Chain::Mantle, Chain::XLayer] {
            assert_eq!(
                asset_icon(&AssetId::from_chain(chain)),
                GemAssetIcon {
                    image: GemAssetIconImage::Local { chain },
                    badge: None,
                    placeholder: Some("ERC20".to_string())
                }
            );
        }
    }

    #[test]
    fn test_every_ethereum_layer2_draws_ether_exactly_when_its_native_coin_is_ether() {
        let ether = Asset::from_chain(Chain::Ethereum).symbol;
        for chain in Chain::all().into_iter().filter(|chain| EVMChain::from_chain(*chain).is_some_and(|chain| chain.is_ethereum_layer2())) {
            let icon = asset_icon(&AssetId::from_chain(chain));
            match Asset::from_chain(chain).symbol == ether {
                true => assert_eq!(
                    icon,
                    GemAssetIcon {
                        image: GemAssetIconImage::Local { chain: Chain::Ethereum },
                        badge: Some(chain),
                        placeholder: chain.default_asset_type().map(|asset_type| asset_type.as_ref().to_string())
                    },
                    "{chain}"
                ),
                false => assert_eq!(
                    icon,
                    GemAssetIcon {
                        image: GemAssetIconImage::Local { chain },
                        badge: None,
                        placeholder: chain.default_asset_type().map(|asset_type| asset_type.as_ref().to_string())
                    },
                    "{chain}"
                ),
            }
        }
    }

    #[test]
    fn test_tokens_on_a_layer2_with_its_own_coin_keep_their_chain_badge() {
        let celo_usdt = AssetId::from_token(Chain::Celo, "0x48065fbBE25f71C9282ddf5e1cD6D6A887483D5e");
        assert_eq!(asset_icon(&celo_usdt).badge, Some(Chain::Celo));
    }

    #[test]
    fn test_known_usdt_and_usdc_draw_the_bundled_token_logo_badged_with_their_chain() {
        use primitives::known_assets::{ETHEREUM_USDT, HYPERCORE_PERPETUAL_USDC, SOLANA_USDC, SUI_SBUSDT, TEMPO_BRIDGED_USDC, TRON_USDT};
        assert_eq!(
            asset_icon(&ETHEREUM_USDT.id),
            GemAssetIcon {
                image: GemAssetIconImage::LocalToken { token: GemLocalTokenIcon::Usdt },
                badge: Some(Chain::Ethereum),
                placeholder: Some("ERC20".to_string())
            }
        );
        assert_eq!(
            asset_icon(&TRON_USDT.id),
            GemAssetIcon {
                image: GemAssetIconImage::LocalToken { token: GemLocalTokenIcon::Usdt },
                badge: Some(Chain::Tron),
                placeholder: Some("TRC20".to_string())
            }
        );
        assert_eq!(
            asset_icon(&SOLANA_USDC.id),
            GemAssetIcon {
                image: GemAssetIconImage::LocalToken { token: GemLocalTokenIcon::Usdc },
                badge: Some(Chain::Solana),
                placeholder: Some("SPL".to_string())
            }
        );
        assert_eq!(asset_icon(&HYPERCORE_PERPETUAL_USDC.id).image, GemAssetIconImage::LocalToken { token: GemLocalTokenIcon::Usdc });
        assert_eq!(asset_icon(&TEMPO_BRIDGED_USDC.id).image, GemAssetIconImage::mock_remote(&TEMPO_BRIDGED_USDC.id));
        assert_eq!(asset_icon(&SUI_SBUSDT.id).image, GemAssetIconImage::mock_remote(&SUI_SBUSDT.id));
        assert_eq!(
            asset_icon(&AssetId::from_token(Chain::Ethereum, "0x0000000000000000000000000000000000000001")).image,
            GemAssetIconImage::mock_remote(&AssetId::from_token(Chain::Ethereum, "0x0000000000000000000000000000000000000001"))
        );
    }

    #[test]
    fn test_the_placeholder_names_the_chain_token_standard_not_the_symbol() {
        let placeholder = |asset_id: AssetId| asset_icon(&asset_id).placeholder;

        assert_eq!(placeholder(AssetId::from_token(Chain::Ethereum, "0x6982508145454Ce325dDbE47a25d4ec3d2311933")), Some("ERC20".to_string()));
        assert_eq!(placeholder(AssetId::from_token(Chain::SmartChain, "0x55d398326f99059fF775485246999027B3197955")), Some("BEP20".to_string()));
        assert_eq!(placeholder(AssetId::from_token(Chain::Tron, "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t")), Some("TRC20".to_string()));
        assert_eq!(placeholder(AssetId::from_token(Chain::Solana, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")), Some("SPL".to_string()));
        assert_eq!(placeholder(AssetId::from_chain(Chain::Bitcoin)), None, "a chain without tokens has no placeholder text");
    }

    #[test]
    fn test_tokens_draw_their_remote_image_badged_with_their_own_chain() {
        let base_usdc = AssetId::from(Chain::Base, Some("0x833589fcd6edb6e08f4c7c32d4f71b54bda02913".to_string()));
        let ethereum_wbtc = AssetId::from_token(Chain::Ethereum, "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599");
        let sei_token = AssetId::from(Chain::SeiEvm, Some("0x3894085ef7ff0f0aedf52e2a2704928d1ec074f1".to_string()));

        assert_eq!(
            asset_icon(&base_usdc),
            GemAssetIcon {
                image: GemAssetIconImage::mock_remote(&base_usdc),
                badge: Some(Chain::Base),
                placeholder: Some("ERC20".to_string())
            }
        );
        assert_eq!(
            asset_icon(&ethereum_wbtc),
            GemAssetIcon {
                image: GemAssetIconImage::mock_remote(&ethereum_wbtc),
                badge: Some(Chain::Ethereum),
                placeholder: Some("ERC20".to_string())
            }
        );
        assert_eq!(
            asset_icon(&sei_token),
            GemAssetIcon {
                image: GemAssetIconImage::mock_remote(&sei_token),
                badge: Some(Chain::Sei),
                placeholder: Some("ERC20".to_string())
            }
        );
    }

    #[test]
    fn test_perpetuals_borrow_the_coin_chain_logo_when_the_coin_is_a_known_chain() {
        assert_eq!(
            asset_icon(&perpetual_asset_id("BTC")),
            GemAssetIcon {
                image: GemAssetIconImage::Local { chain: Chain::Bitcoin },
                badge: Some(Chain::HyperCore),
                placeholder: Some("TOKEN".to_string())
            }
        );
        assert_eq!(
            asset_icon(&perpetual_asset_id("ETH")),
            GemAssetIcon {
                image: GemAssetIconImage::Local { chain: Chain::Ethereum },
                badge: Some(Chain::HyperCore),
                placeholder: Some("TOKEN".to_string())
            }
        );
        let unknown = perpetual_asset_id("PUMP");
        assert_eq!(
            asset_icon(&unknown),
            GemAssetIcon {
                image: GemAssetIconImage::mock_remote(&unknown),
                badge: Some(Chain::HyperCore),
                placeholder: Some("TOKEN".to_string())
            }
        );
    }
}

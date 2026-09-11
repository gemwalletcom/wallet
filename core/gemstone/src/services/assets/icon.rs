use primitives::known_assets::{USDC_ASSETS, USDT_ASSETS};
use primitives::{Asset, AssetId, Chain};

use crate::config::chain::{badge_chain, icon_chain, is_ether_layer2};
use crate::config::image::GemImage;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAssetIcon {
    pub image: GemAssetIconImage,
    pub badge: Option<Chain>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemLocalTokenIcon {
    Usdt,
    Usdc,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAssetIconImage {
    Local { chain: Chain },
    LocalToken { token: GemLocalTokenIcon },
    Remote { url: String },
}

pub fn asset_icon(asset_id: &AssetId) -> GemAssetIcon {
    let icon_asset_id = icon_asset_id(asset_id);
    let image = if let Some(token) = local_token_icon(asset_id) {
        GemAssetIconImage::LocalToken { token }
    } else if icon_asset_id.is_native() {
        GemAssetIconImage::Local {
            chain: icon_chain(icon_asset_id.chain),
        }
    } else {
        GemAssetIconImage::Remote {
            url: GemImage::Asset { asset_id: icon_asset_id }.url(),
        }
    };
    let badge = match asset_id.is_native() {
        true => badge_chain(asset_id.chain),
        false => Some(icon_chain(asset_id.chain)),
    };
    GemAssetIcon { image, badge }
}

fn icon_asset_id(asset_id: &AssetId) -> AssetId {
    if let Some(coin) = perpetual_coin(asset_id) {
        return Chain::all()
            .into_iter()
            .find(|chain| Asset::from_chain(*chain).symbol == coin)
            .map(AssetId::from_chain)
            .unwrap_or_else(|| asset_id.clone());
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
    use primitives::EVMChain;

    fn local(chain: Chain) -> GemAssetIconImage {
        GemAssetIconImage::Local { chain }
    }

    fn remote(asset_id: &AssetId) -> GemAssetIconImage {
        GemAssetIconImage::Remote {
            url: GemImage::Asset { asset_id: asset_id.clone() }.url(),
        }
    }

    fn perpetual(coin: &str) -> AssetId {
        AssetId::from(Chain::HyperCore, Some(AssetId::sub_token_id(&["perpetual".to_string(), coin.to_string()])))
    }

    #[test]
    fn test_native_assets_draw_their_coin_and_badge_only_on_ethereum_layer2() {
        assert_eq!(
            asset_icon(&AssetId::from_chain(Chain::Ethereum)),
            GemAssetIcon {
                image: local(Chain::Ethereum),
                badge: None
            }
        );
        assert_eq!(
            asset_icon(&AssetId::from_chain(Chain::Base)),
            GemAssetIcon {
                image: local(Chain::Ethereum),
                badge: Some(Chain::Base)
            }
        );
        assert_eq!(
            asset_icon(&AssetId::from_chain(Chain::Robinhood)),
            GemAssetIcon {
                image: local(Chain::Ethereum),
                badge: Some(Chain::Robinhood)
            }
        );
        assert_eq!(
            asset_icon(&AssetId::from_chain(Chain::SeiEvm)),
            GemAssetIcon {
                image: local(Chain::Sei),
                badge: None
            }
        );
        assert_eq!(
            asset_icon(&AssetId::from_chain(Chain::OpBNB)),
            GemAssetIcon {
                image: local(Chain::OpBNB),
                badge: None
            }
        );
    }

    #[test]
    fn test_layer2_coins_that_are_not_ether_draw_their_own_logo_without_a_badge() {
        for chain in [Chain::Celo, Chain::Mantle, Chain::XLayer] {
            assert_eq!(
                asset_icon(&AssetId::from_chain(chain)),
                GemAssetIcon {
                    image: local(chain),
                    badge: None
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
                true => assert_eq!(icon, GemAssetIcon { image: local(Chain::Ethereum), badge: Some(chain) }, "{chain}"),
                false => assert_eq!(icon, GemAssetIcon { image: local(chain), badge: None }, "{chain}"),
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
        let token = |token| GemAssetIconImage::LocalToken { token };
        assert_eq!(
            asset_icon(&ETHEREUM_USDT.id),
            GemAssetIcon {
                image: token(GemLocalTokenIcon::Usdt),
                badge: Some(Chain::Ethereum)
            }
        );
        assert_eq!(
            asset_icon(&TRON_USDT.id),
            GemAssetIcon {
                image: token(GemLocalTokenIcon::Usdt),
                badge: Some(Chain::Tron)
            }
        );
        assert_eq!(
            asset_icon(&SOLANA_USDC.id),
            GemAssetIcon {
                image: token(GemLocalTokenIcon::Usdc),
                badge: Some(Chain::Solana)
            }
        );
        assert_eq!(asset_icon(&HYPERCORE_PERPETUAL_USDC.id).image, token(GemLocalTokenIcon::Usdc));
        assert_eq!(asset_icon(&TEMPO_BRIDGED_USDC.id).image, remote(&TEMPO_BRIDGED_USDC.id));
        assert_eq!(asset_icon(&SUI_SBUSDT.id).image, remote(&SUI_SBUSDT.id));
        assert_eq!(
            asset_icon(&AssetId::from_token(Chain::Ethereum, "0x0000000000000000000000000000000000000001")).image,
            remote(&AssetId::from_token(Chain::Ethereum, "0x0000000000000000000000000000000000000001"))
        );
    }

    #[test]
    fn test_tokens_draw_their_remote_image_badged_with_their_own_chain() {
        let base_usdc = AssetId::from(Chain::Base, Some("0x833589fcd6edb6e08f4c7c32d4f71b54bda02913".to_string()));
        let ethereum_wbtc = AssetId::from_token(Chain::Ethereum, "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599");
        let sei_token = AssetId::from(Chain::SeiEvm, Some("0x3894085ef7ff0f0aedf52e2a2704928d1ec074f1".to_string()));

        assert_eq!(
            asset_icon(&base_usdc),
            GemAssetIcon {
                image: remote(&base_usdc),
                badge: Some(Chain::Base)
            }
        );
        assert_eq!(
            asset_icon(&ethereum_wbtc),
            GemAssetIcon {
                image: remote(&ethereum_wbtc),
                badge: Some(Chain::Ethereum)
            }
        );
        assert_eq!(
            asset_icon(&sei_token),
            GemAssetIcon {
                image: remote(&sei_token),
                badge: Some(Chain::Sei)
            }
        );
    }

    #[test]
    fn test_perpetuals_borrow_the_coin_chain_logo_when_the_coin_is_a_known_chain() {
        assert_eq!(
            asset_icon(&perpetual("BTC")),
            GemAssetIcon {
                image: local(Chain::Bitcoin),
                badge: Some(Chain::HyperCore)
            }
        );
        assert_eq!(
            asset_icon(&perpetual("ETH")),
            GemAssetIcon {
                image: local(Chain::Ethereum),
                badge: Some(Chain::HyperCore)
            }
        );
        let unknown = perpetual("PUMP");
        assert_eq!(
            asset_icon(&unknown),
            GemAssetIcon {
                image: remote(&unknown),
                badge: Some(Chain::HyperCore)
            }
        );
    }
}

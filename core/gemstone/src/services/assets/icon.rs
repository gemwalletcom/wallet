use primitives::{Asset, AssetId, Chain};

use crate::config::chain::{badge_chain, icon_chain, is_ethereum_layer2};
use crate::config::image::GemImage;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAssetIcon {
    pub image: GemAssetIconImage,
    pub badge: Option<Chain>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAssetIconImage {
    Local { chain: Chain },
    Remote { url: String },
}

pub fn asset_icon(asset_id: &AssetId) -> GemAssetIcon {
    let icon_asset_id = icon_asset_id(asset_id);
    let image = match icon_asset_id.is_native() {
        true => GemAssetIconImage::Local {
            chain: icon_chain(icon_asset_id.chain),
        },
        false => GemAssetIconImage::Remote {
            url: GemImage::Asset { asset_id: icon_asset_id }.url(),
        },
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
    if asset_id.is_native() && is_ethereum_layer2(asset_id.chain) {
        return AssetId::from_chain(Chain::Ethereum);
    }
    asset_id.clone()
}

fn perpetual_coin(asset_id: &AssetId) -> Option<String> {
    let ids = AssetId::decode_token_id(asset_id.token_id.as_deref()?);
    (asset_id.chain == Chain::HyperCore && ids.first().is_some_and(|kind| kind == "perpetual")).then(|| ids.get(1).cloned())?
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_tokens_draw_their_remote_image_badged_with_their_own_chain() {
        let base_usdc = AssetId::from(Chain::Base, Some("0x833589fcd6edb6e08f4c7c32d4f71b54bda02913".to_string()));
        let ethereum_usdc = Asset::mock_ethereum_usdc().id;
        let sei_token = AssetId::from(Chain::SeiEvm, Some("0x3894085ef7ff0f0aedf52e2a2704928d1ec074f1".to_string()));

        assert_eq!(
            asset_icon(&base_usdc),
            GemAssetIcon {
                image: remote(&base_usdc),
                badge: Some(Chain::Base)
            }
        );
        assert_eq!(
            asset_icon(&ethereum_usdc),
            GemAssetIcon {
                image: remote(&ethereum_usdc),
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

use primitives::{Asset, AssetId, AssetType, Chain, Wallet};

use super::model::{GemReceiveAssetState, GemReceiveNetwork, GemReceiveNetworks, GemReceiveWarning};
use crate::config::chain::is_memo_supported;
use crate::services::localization::GemLocalizedText;

pub fn asset_state(asset: &Asset) -> GemReceiveAssetState {
    let text = crate::services::assets::rules::asset_text(asset);
    let chain = asset.chain();
    let memo = match (is_memo_supported(chain), chain) {
        (false, _) => None,
        (true, Chain::Xrp) => Some(GemReceiveWarning::NoDestinationTagRequired),
        (true, _) => Some(GemReceiveWarning::NoMemoRequired),
    };
    let network = GemReceiveWarning::AssetNetwork {
        symbol: asset.symbol.clone(),
        network: text.network_full_name.clone(),
    };
    GemReceiveAssetState {
        warnings: std::iter::once(network).chain(memo).collect(),
        asset: text,
    }
}

pub fn networks(asset: &Asset, associations: Vec<AssetId>, wallet: &Wallet) -> GemReceiveNetworks {
    let networks: Vec<GemReceiveNetwork> = network_asset_ids(asset.id.clone(), associations, wallet)
        .into_iter()
        .map(|asset_id| GemReceiveNetwork {
            standard: match asset_id == asset.id {
                true => standard(&asset.asset_type),
                false => asset_id.token_id.as_ref().and(asset_id.chain.default_asset_type()).as_ref().and_then(standard),
            },
            asset_id,
        })
        .collect();
    GemReceiveNetworks {
        shows_selector: networks.len() > 1,
        networks,
    }
}

fn standard(asset_type: &AssetType) -> Option<GemLocalizedText> {
    match asset_type {
        AssetType::ERC20 | AssetType::BEP20 | AssetType::TRC20 | AssetType::SPL | AssetType::JETTON | AssetType::TIP20 | AssetType::IBC | AssetType::SYNTH | AssetType::ASA => {
            Some(GemLocalizedText::Text { text: asset_type.as_ref().to_string() })
        }
        AssetType::NATIVE | AssetType::TOKEN | AssetType::SPL2022 | AssetType::PERPETUAL | AssetType::SPOT => None,
    }
}

fn network_asset_ids(asset_id: AssetId, associations: Vec<AssetId>, wallet: &Wallet) -> Vec<AssetId> {
    let mut asset_ids: Vec<AssetId> = Vec::new();
    for candidate in std::iter::once(asset_id).chain(associations) {
        if wallet.account(candidate.chain).is_some() && !asset_ids.contains(&candidate) {
            asset_ids.push(candidate);
        }
    }
    asset_ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Account;

    fn token(id: AssetId, asset_type: AssetType) -> Asset {
        Asset { id, asset_type, ..Asset::mock() }
    }

    #[test]
    fn test_networks_keep_wallet_chains_in_order_without_duplicates_and_offer_the_selector_past_one() {
        let ethereum_usdc = AssetId::from_token(Chain::Ethereum, "0xusdc");
        let base_usdc = AssetId::from_token(Chain::Base, "0xusdc");
        let solana_usdc = AssetId::from_token(Chain::Solana, "usdc");
        let wallet = Wallet::mock_with_accounts(Account::mock_chains(&[Chain::Ethereum, Chain::Base], "address"));

        let usdc = token(ethereum_usdc.clone(), AssetType::ERC20);
        let asset_ids = |networks: GemReceiveNetworks| networks.networks.into_iter().map(|network| network.asset_id).collect::<Vec<_>>();

        let several = networks(&usdc, vec![base_usdc.clone(), solana_usdc.clone(), ethereum_usdc.clone()], &wallet);
        let single = networks(&usdc, vec![solana_usdc], &wallet);

        assert!(several.shows_selector);
        assert_eq!(asset_ids(several), vec![ethereum_usdc.clone(), base_usdc]);
        assert!(!single.shows_selector);
        assert_eq!(asset_ids(single), vec![ethereum_usdc]);
    }

    #[test]
    fn test_a_network_names_a_known_token_standard_and_nothing_for_a_coin_or_a_generic_token() {
        let wallet = Wallet::mock_with_accounts(Account::mock_chains(&[Chain::Ethereum, Chain::Solana, Chain::Arbitrum, Chain::Sui], "address"));
        let label = |text: &str| Some(GemLocalizedText::Text { text: text.to_string() });
        let standards = |asset: Asset, associations: Vec<AssetId>| networks(&asset, associations, &wallet).networks.into_iter().map(|network| network.standard).collect::<Vec<_>>();

        assert_eq!(
            standards(
                token(AssetId::from_token(Chain::Ethereum, "0xusdc"), AssetType::ERC20),
                vec![AssetId::from_token(Chain::Solana, "usdc"), AssetId::from_token(Chain::Sui, "0x2::usdc")]
            ),
            vec![label("ERC20"), label("SPL"), None]
        );
        assert_eq!(
            standards(token(AssetId::from_token(Chain::Solana, "token2022"), AssetType::SPL2022), vec![]),
            vec![None],
            "a Token-2022 asset reads by its own type, not the network's usual one"
        );
        assert_eq!(standards(Asset::from_chain(Chain::Ethereum), vec![AssetId::from_chain(Chain::Arbitrum)]), vec![None, None], "a coin has no token standard");
    }

    #[test]
    fn test_receive_warnings_lead_with_the_asset_network_sentence_and_name_the_memo_field() {
        let warnings = |chain: Chain| asset_state(&Asset::from_chain(chain)).warnings;
        let network = |chain: Chain| {
            let asset = Asset::from_chain(chain);
            GemReceiveWarning::AssetNetwork {
                network: crate::services::assets::rules::asset_text(&asset).network_full_name,
                symbol: asset.symbol,
            }
        };
        assert_eq!(warnings(Chain::Ethereum), vec![network(Chain::Ethereum)]);
        assert_eq!(warnings(Chain::Bitcoin), vec![network(Chain::Bitcoin)]);
        assert_eq!(warnings(Chain::Xrp), vec![network(Chain::Xrp), GemReceiveWarning::NoDestinationTagRequired]);
        assert_eq!(warnings(Chain::Cosmos), vec![network(Chain::Cosmos), GemReceiveWarning::NoMemoRequired]);
        assert_eq!(warnings(Chain::Ton), vec![network(Chain::Ton), GemReceiveWarning::NoMemoRequired]);
    }

    #[test]
    fn test_a_token_names_its_network_and_standard_in_the_warning() {
        let usdt = Asset::new(AssetId::from_token(Chain::Ethereum, "0xdac17f958d2ee523a2206206994597c13d831ec7"), "Tether".into(), "USDT".into(), 6, AssetType::ERC20);

        assert_eq!(
            asset_state(&usdt).warnings[0],
            GemReceiveWarning::AssetNetwork {
                symbol: "USDT".to_string(),
                network: "Ethereum (ERC20)".to_string()
            }
        );
        assert_eq!(asset_state(&usdt).asset.subtitle_symbol.as_deref(), Some("USDT"));
    }
}

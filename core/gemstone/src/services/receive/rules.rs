use primitives::{AssetId, Chain, Wallet};

use super::model::{GemReceiveNetworks, GemReceiveWarning};
use crate::config::chain::is_memo_supported;

pub fn warnings(chain: Chain) -> Vec<GemReceiveWarning> {
    let memo = match (is_memo_supported(chain), chain) {
        (false, _) => None,
        (true, Chain::Xrp) => Some(GemReceiveWarning::NoDestinationTagRequired),
        (true, _) => Some(GemReceiveWarning::NoMemoRequired),
    };
    std::iter::once(GemReceiveWarning::AssetNetwork).chain(memo).collect()
}

pub fn networks(asset_id: AssetId, associations: Vec<AssetId>, wallet: &Wallet) -> GemReceiveNetworks {
    let asset_ids = network_asset_ids(asset_id, associations, wallet);
    GemReceiveNetworks {
        shows_selector: asset_ids.len() > 1,
        asset_ids,
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

    #[test]
    fn test_networks_keep_wallet_chains_in_order_without_duplicates_and_offer_the_selector_past_one() {
        let ethereum_usdc = AssetId::from_token(Chain::Ethereum, "0xusdc");
        let base_usdc = AssetId::from_token(Chain::Base, "0xusdc");
        let solana_usdc = AssetId::from_token(Chain::Solana, "usdc");
        let wallet = Wallet::mock_with_accounts(Account::mock_chains(&[Chain::Ethereum, Chain::Base], "address"));

        let several = networks(ethereum_usdc.clone(), vec![base_usdc.clone(), solana_usdc.clone(), ethereum_usdc.clone()], &wallet);
        let single = networks(ethereum_usdc.clone(), vec![solana_usdc], &wallet);

        assert_eq!(several.asset_ids, vec![ethereum_usdc.clone(), base_usdc]);
        assert!(several.shows_selector);
        assert_eq!(single.asset_ids, vec![ethereum_usdc]);
        assert!(!single.shows_selector);
    }

    #[test]
    fn test_receive_warnings_lead_with_the_asset_network_sentence_and_name_the_memo_field() {
        assert_eq!(warnings(Chain::Ethereum), vec![GemReceiveWarning::AssetNetwork]);
        assert_eq!(warnings(Chain::Bitcoin), vec![GemReceiveWarning::AssetNetwork]);
        assert_eq!(warnings(Chain::Xrp), vec![GemReceiveWarning::AssetNetwork, GemReceiveWarning::NoDestinationTagRequired]);
        assert_eq!(warnings(Chain::Cosmos), vec![GemReceiveWarning::AssetNetwork, GemReceiveWarning::NoMemoRequired]);
        assert_eq!(warnings(Chain::Ton), vec![GemReceiveWarning::AssetNetwork, GemReceiveWarning::NoMemoRequired]);
    }
}

use primitives::{Account, Chain, NFTAssetData, NFTData, VerificationStatus, Wallet, WalletType};

use crate::config::chain::supports_nft_transfer;
use crate::services::chain::rules::chain_matches_query;

pub fn verified_collections(data: Vec<NFTData>) -> Vec<NFTData> {
    collections(data, true)
}

pub fn unverified_collections(data: Vec<NFTData>) -> Vec<NFTData> {
    collections(data, false)
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemNftSearchItem {
    Collection { data: NFTData },
    Asset { data: NFTAssetData },
}

pub fn search_collections(data: Vec<NFTData>, query: &str) -> Vec<GemNftSearchItem> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Vec::new();
    }
    sorted_collections(data)
        .into_iter()
        .flat_map(|data| {
            if data.collection.name.to_lowercase().contains(&query) {
                return vec![GemNftSearchItem::Collection { data }];
            }
            let mut assets: Vec<_> = data.assets.into_iter().filter(|asset| asset.name.to_lowercase().contains(&query)).collect();
            assets.sort_by_key(|asset| asset.name.to_lowercase());
            assets
                .into_iter()
                .map(|asset| GemNftSearchItem::Asset {
                    data: NFTAssetData {
                        collection: data.collection.clone(),
                        asset,
                    },
                })
                .collect()
        })
        .collect()
}

pub fn sorted_collections(data: Vec<NFTData>) -> Vec<NFTData> {
    let mut sorted = data;
    sorted.sort_by(|left, right| {
        right
            .assets
            .len()
            .cmp(&left.assets.len())
            .then_with(|| left.collection.name.to_lowercase().cmp(&right.collection.name.to_lowercase()))
    });
    sorted
}

pub fn nft_chains() -> Vec<Chain> {
    Chain::all().into_iter().filter(Chain::is_nft_supported).collect()
}

pub fn receive_accounts(wallet: &Wallet, query: &str) -> Vec<Account> {
    wallet
        .accounts
        .iter()
        .filter(|account| account.chain.is_nft_supported() && chain_matches_query(account.chain, query))
        .cloned()
        .collect()
}

pub fn can_send(wallet_type: &WalletType, chain: Chain) -> bool {
    *wallet_type != WalletType::View && supports_nft_transfer(chain)
}

fn collections(data: Vec<NFTData>, verified: bool) -> Vec<NFTData> {
    data.into_iter()
        .filter(|item| !item.assets.is_empty() && (item.collection.status == VerificationStatus::Verified) == verified)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::NFTData;

    #[test]
    fn test_receive_accounts_keeps_nft_chains_matching_the_query() {
        let wallet = Wallet::mock_with_accounts(Account::mock_chains(&[Chain::Ethereum, Chain::Bitcoin, Chain::Solana], "address"));
        let chains = |query: &str| receive_accounts(&wallet, query).into_iter().map(|account| account.chain).collect::<Vec<_>>();
        assert_eq!(chains(""), vec![Chain::Ethereum, Chain::Solana]);
        assert_eq!(chains("sol"), vec![Chain::Solana]);
    }

    #[test]
    fn test_can_send_needs_a_signing_wallet_on_a_transfer_chain() {
        assert!(can_send(&WalletType::Multicoin, Chain::Ethereum));
        assert!(!can_send(&WalletType::View, Chain::Ethereum));
        assert!(!can_send(&WalletType::Multicoin, Chain::Bitcoin));
    }

    #[test]
    fn test_collections_split_by_verification_and_skip_empty_ones() {
        let verified = NFTData::mock_with("verified", VerificationStatus::Verified, 1);
        let unverified = NFTData::mock_with("unverified", VerificationStatus::Unverified, 2);
        let empty = NFTData::mock_with("empty", VerificationStatus::Verified, 0);
        let items = vec![verified, unverified, empty];

        assert_eq!(names(verified_collections(items.clone())), vec!["verified"]);
        assert_eq!(names(unverified_collections(items)), vec!["unverified"]);
    }

    fn names(data: Vec<NFTData>) -> Vec<String> {
        data.into_iter().map(|item| item.collection.name).collect()
    }

    #[test]
    fn test_search_matches_the_collection_name_or_its_assets() {
        let punks = NFTData::mock_with("Punks", VerificationStatus::Verified, 2);
        let mut apes = NFTData::mock_with("Apes", VerificationStatus::Verified, 3);
        apes.assets[1].name = "Punk Ape".to_string();
        apes.assets[2].name = "Ape 2".to_string();
        let items = vec![punks.clone(), apes.clone()];

        let results = search_collections(items.clone(), " punk ");
        let labels: Vec<String> = results
            .iter()
            .map(|item| match item {
                GemNftSearchItem::Collection { data } => format!("collection {} ({})", data.collection.name, data.assets.len()),
                GemNftSearchItem::Asset { data } => format!("asset {} of {}", data.asset.name, data.collection.name),
            })
            .collect();
        assert_eq!(labels, vec!["asset Punk Ape of Apes", "collection Punks (2)"]);
        assert!(search_collections(items.clone(), "").is_empty());
        assert!(search_collections(items, "zzz").is_empty());
    }

    #[test]
    fn test_collections_sort_by_size_then_name() {
        let big = NFTData::mock_with("zebra", VerificationStatus::Verified, 3);
        let small_a = NFTData::mock_with("alpha", VerificationStatus::Verified, 1);
        let small_b = NFTData::mock_with("beta", VerificationStatus::Verified, 1);

        let sorted = sorted_collections(vec![small_b, big, small_a]);

        assert_eq!(names(sorted), vec!["zebra", "alpha", "beta"]);
    }
}

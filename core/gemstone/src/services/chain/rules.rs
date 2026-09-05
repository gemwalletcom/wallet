use std::cmp::Reverse;

use primitives::{AssetId, Chain, ChainAsset, NodeCheckProfile, NodeCheckRequest, Wallet, node_check_request};

use crate::services::collections::unique;

pub fn chains_by_rank() -> Vec<Chain> {
    let mut chains = Chain::all();
    chains.sort_by_key(|chain| Reverse(AssetId::from_chain(*chain).default_rank()));
    chains
}

pub fn wallet_chains_by_rank(wallet: &Wallet) -> Vec<Chain> {
    let mut chains = unique(wallet.accounts.iter().map(|account| account.chain));
    chains.sort_by_key(|chain| Reverse(AssetId::from_chain(*chain).default_rank()));
    chains
}

pub fn matching_chains(chains: Vec<Chain>, query: &str) -> Vec<Chain> {
    chains.into_iter().filter(|chain| chain_matches_query(*chain, query)).collect()
}

pub fn chain_matches_query(chain: Chain, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return true;
    }
    let chain_asset = ChainAsset::from_chain(chain);
    [
        chain_asset.network_name.to_lowercase(),
        chain.as_ref().to_lowercase(),
        chain_asset.asset.name.to_lowercase(),
        chain_asset.asset.symbol.to_lowercase(),
    ]
    .iter()
    .any(|value| value.contains(&query))
}

pub fn mismatched_network_id(chain: Chain, network_id: Option<&str>) -> Option<String> {
    network_id.filter(|network_id| chain.network_id() != *network_id).map(str::to_string)
}

pub fn node_verification_address(chain: Chain) -> Option<String> {
    match chain {
        Chain::Polkadot => match node_check_request(chain, NodeCheckProfile::Wallet) {
            NodeCheckRequest::Wallet { address, .. } => Some(address),
            NodeCheckRequest::Basic | NodeCheckRequest::Parser => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Account;
    use primitives::Wallet;

    #[test]
    fn test_wallet_chains_by_rank_orders_the_wallet_accounts() {
        let wallet = Wallet::mock_with_accounts(Account::mock_chains(&[Chain::Doge, Chain::Ethereum, Chain::Doge, Chain::Bitcoin], "address"));
        assert_eq!(wallet_chains_by_rank(&wallet), vec![Chain::Bitcoin, Chain::Ethereum, Chain::Doge]);
    }

    #[test]
    fn test_chains_by_rank_orders_and_filters() {
        let chains = chains_by_rank();

        assert_eq!(chains.len(), Chain::all().len());
        let ranks: Vec<i32> = chains.iter().map(|chain| AssetId::from_chain(*chain).default_rank()).collect();
        assert!(ranks.windows(2).all(|pair| pair[0] >= pair[1]));
        assert_eq!(matching_chains(chains.clone(), "bitcoin").first(), Some(&Chain::Bitcoin));
        assert_eq!(matching_chains(chains, ""), chains_by_rank());
    }

    #[test]
    fn test_query_matching_ignores_case_and_surrounding_space() {
        assert!(chain_matches_query(Chain::Ethereum, " ETH "));
        assert!(chain_matches_query(Chain::Ethereum, "ethereum"));
        assert!(!chain_matches_query(Chain::Ethereum, "bitcoin"));
    }

    #[test]
    fn test_a_blank_query_keeps_every_chain() {
        assert_eq!(matching_chains(vec![Chain::Ethereum, Chain::Bitcoin], "   "), vec![Chain::Ethereum, Chain::Bitcoin]);
        assert_eq!(matching_chains(vec![Chain::Ethereum, Chain::Bitcoin], "bit"), vec![Chain::Bitcoin]);
    }

    #[test]
    fn test_only_a_reported_network_id_is_checked() {
        assert_eq!(mismatched_network_id(Chain::Ethereum, Some(Chain::Ethereum.network_id())), None);
        assert_eq!(mismatched_network_id(Chain::Ethereum, None), None);
        assert_eq!(
            mismatched_network_id(Chain::Ethereum, Some(Chain::SmartChain.network_id())),
            Some(Chain::SmartChain.network_id().to_string())
        );
    }
}

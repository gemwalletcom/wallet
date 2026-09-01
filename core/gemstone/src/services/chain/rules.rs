use std::cmp::Reverse;

use primitives::{AssetId, Chain, ChainAsset, NodeCheckProfile, NodeCheckRequest, node_check_request};

pub fn chains_by_rank() -> Vec<Chain> {
    let mut chains = Chain::all();
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

pub fn is_valid_network_id(chain: Chain, network_id: &str) -> bool {
    chain.network_id() == network_id
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
    fn test_is_valid_network_id() {
        assert!(is_valid_network_id(Chain::Ethereum, Chain::Ethereum.network_id()));
        assert!(!is_valid_network_id(Chain::Ethereum, Chain::SmartChain.network_id()));
    }
}

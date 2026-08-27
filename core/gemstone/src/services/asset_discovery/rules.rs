use std::collections::HashSet;

use primitives::{Account, AssetId, Chain};

use crate::services::collections::unique;

pub fn discoverable_asset_ids(asset_ids: Vec<String>, accounts: &[Account]) -> Vec<AssetId> {
    let chains: HashSet<Chain> = accounts.iter().map(|account| account.chain).collect();
    unique(asset_ids.into_iter().filter_map(|id| AssetId::new(&id)).filter(|asset_id| chains.contains(&asset_id.chain)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discoverable_asset_ids_keeps_wallet_chains_and_dedupes() {
        let account = Account {
            chain: Chain::Ethereum,
            address: "0xaddress".into(),
            derivation_path: "".into(),
            extended_public_key: None,
        };

        let asset_ids = discoverable_asset_ids(
            vec!["ethereum_0xusdc".into(), "ethereum_0xusdc".into(), "solana_usdc".into(), "not an id".into()],
            &[account],
        );

        assert_eq!(asset_ids, vec![AssetId::from_token(Chain::Ethereum, "0xusdc")]);
    }
}

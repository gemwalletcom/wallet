use std::collections::HashSet;

use primitives::AssetId;

pub fn missing_asset_ids(requested: Vec<AssetId>, existing: Vec<AssetId>) -> Vec<AssetId> {
    let existing: HashSet<AssetId> = existing.into_iter().collect();
    let mut seen: HashSet<AssetId> = HashSet::new();
    requested
        .into_iter()
        .filter(|asset_id| !existing.contains(asset_id) && seen.insert(asset_id.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_missing_asset_ids_drops_known_and_duplicate_ids() {
        let bitcoin = AssetId::from_chain(Chain::Bitcoin);
        let ethereum = AssetId::from_chain(Chain::Ethereum);

        let missing = missing_asset_ids(vec![bitcoin.clone(), ethereum.clone(), ethereum.clone()], vec![bitcoin]);

        assert_eq!(missing, vec![ethereum]);
    }
}

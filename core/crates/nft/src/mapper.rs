use std::collections::HashMap;

use primitives::{NFTAsset, NFTCollection, NFTCollectionId, NFTData};

pub(crate) fn map_nft_data(assets: Vec<NFTAsset>, collections: Vec<NFTCollection>) -> Vec<NFTData> {
    let mut by_collection: HashMap<NFTCollectionId, Vec<NFTAsset>> = HashMap::new();
    for asset in assets {
        by_collection.entry(asset.collection_id.clone()).or_default().push(asset);
    }

    collections
        .into_iter()
        .filter_map(|collection| {
            let assets = by_collection.remove(&collection.id)?;
            Some(NFTData { collection, assets })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use primitives::NFTAssetId;

    use super::*;

    #[test]
    fn test_map_nft_data_skips_missing_collections() {
        let asset = NFTAsset::mock();
        let collection = NFTCollection::mock();
        let data = map_nft_data(vec![NFTAsset::mock_ton(), asset.clone()], vec![collection.clone()]);

        assert_eq!(data.len(), 1);
        assert_eq!(data[0].collection, collection);
        assert_eq!(data[0].assets.iter().map(|asset| asset.id.clone()).collect::<Vec<_>>(), vec![asset.id]);
    }

    #[test]
    fn test_map_nft_data_groups_assets() {
        let first = NFTAsset::mock();
        let second = NFTAsset {
            id: NFTAssetId::new(first.chain, &first.collection_id.contract_address, "2"),
            token_id: "2".to_string(),
            ..first.clone()
        };
        let data = map_nft_data(vec![first.clone(), second.clone()], vec![NFTCollection::mock()]);

        assert_eq!(data.len(), 1);
        assert_eq!(data[0].assets.iter().map(|asset| asset.id.clone()).collect::<Vec<_>>(), vec![first.id, second.id]);
    }

    #[test]
    fn test_map_nft_data_without_available_nfts() {
        assert_eq!(map_nft_data(vec![NFTAsset::mock()], vec![]).len(), 0);
        assert_eq!(map_nft_data(vec![], vec![NFTCollection::mock()]).len(), 0);
        assert_eq!(map_nft_data(vec![], vec![]).len(), 0);
    }
}

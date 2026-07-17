use gem_ton::address::Address;
use gem_ton::models::{NftCollectionsResponse, NftItem, NftItemsResponse, NftOffchainMetadata, TokenInfo, TokenMetadata};
use primitives::{Address as _, Chain, NFTAsset, NFTAssetId, NFTCollection, NFTCollectionId, NFTImages, NFTResource, NFTType, VerificationStatus};

use super::verified::is_verified;

pub fn map_assets(response: &NftItemsResponse) -> Vec<NFTAssetId> {
    response.nft_items.iter().filter_map(asset_id_from_item).collect()
}

pub fn map_collection(response: NftCollectionsResponse, collection_id: NFTCollectionId) -> Option<NFTCollection> {
    let address = Address::try_parse_base64(&collection_id.contract_address)?;
    let collection = response.nft_collections.into_iter().next()?;
    let info = valid_named_token_info(response.metadata.get(&collection.address))?;
    Some(build_collection(&collection_id, &address, info))
}

pub fn map_asset(response: &NftItemsResponse, asset_id: NFTAssetId) -> Option<NFTAsset> {
    Address::try_parse_base64(&asset_id.contract_address)?;
    let item = response.nft_items.first()?;
    map_indexed_asset(response, item, asset_id)
}

pub fn map_offchain_asset(metadata: NftOffchainMetadata, asset_id: NFTAssetId) -> Option<NFTAsset> {
    if metadata.name.is_empty() {
        return None;
    }
    Some(build_asset(asset_id, &metadata.name, metadata.description, metadata.image.as_deref()))
}

pub(super) fn map_indexed_asset(response: &NftItemsResponse, item: &NftItem, asset_id: NFTAssetId) -> Option<NFTAsset> {
    let info = valid_named_token_info(response.metadata.get(&item.address))?;
    let collection_image = item
        .collection_address
        .as_deref()
        .and_then(|address| valid_named_token_info(response.metadata.get(address)))
        .and_then(|info| info.image.as_deref());
    Some(build_asset(
        asset_id,
        token_info_name(info)?,
        info.description.clone(),
        info.image.as_deref().or(collection_image),
    ))
}

pub(super) fn asset_id_from_item(item: &NftItem) -> Option<NFTAssetId> {
    let collection = Address::try_parse_hex(item.collection_address.as_deref()?)?;
    let token = Address::try_parse_hex(&item.address)?;
    Some(NFTAssetId::new(Chain::Ton, &collection.encode(), &token.encode()))
}

fn build_asset(asset_id: NFTAssetId, name: &str, description: Option<String>, image: Option<&str>) -> NFTAsset {
    let image = image.unwrap_or_default();
    let collection_id = asset_id.get_collection_id();
    NFTAsset {
        chain: asset_id.chain,
        contract_address: Some(asset_id.token_id.clone()),
        token_id: asset_id.token_id.clone(),
        id: asset_id,
        collection_id,
        token_type: NFTType::JETTON,
        name: name.to_string(),
        description,
        resource: NFTResource::from_url(image),
        images: NFTImages {
            preview: NFTResource::from_url(image),
        },
        attributes: vec![],
    }
}

fn build_collection(collection_id: &NFTCollectionId, address: &Address, info: &TokenInfo) -> NFTCollection {
    let image = info.image.clone().unwrap_or_default();
    let is_verified = is_verified(address, info);
    NFTCollection {
        id: collection_id.clone(),
        name: token_info_name(info).unwrap_or_default().to_string(),
        symbol: None,
        description: info.description.clone(),
        chain: collection_id.chain,
        contract_address: collection_id.contract_address.clone(),
        images: NFTImages {
            preview: NFTResource::from_url(&image),
        },
        status: VerificationStatus::from_verified(is_verified),
        links: vec![],
        is_verified,
    }
}

fn valid_named_token_info(metadata: Option<&TokenMetadata>) -> Option<&TokenInfo> {
    metadata?.token_info.iter().find(|info| info.valid && token_info_name(info).is_some())
}

fn token_info_name(info: &TokenInfo) -> Option<&str> {
    info.name
        .as_deref()
        .or_else(|| info.extra.as_ref().and_then(|e| e.domain.as_deref()))
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VERIFIED_COLLECTION: &str = "EQCA14o1-VWhS2efqoh_9M1b_A9DtKTuoqfmkn83AbJzwnPi";
    const ITEM: &str = "EQCvxJy4eG8hyHBFsZ7eePxrRsUQSEUTP46abUQGAcGY6mOw";
    const NUMBERS_COLLECTION: &str = "EQAOQdwdw8kGftJCSFgOErM1mBjYPe4DBPq8-AhF6vr9si5N";
    const UNVERIFIED_COLLECTION: &str = "EQBBhhF6O-jfi1TEF1rs6pEaynEjhFrcjUCC2DfUwzJ4pRXR";
    const GETGEMS_COLLECTION: &str = "EQCwbUhN1REI4q-N8EV2ordMxXognDW2y0teRiBP0RjgCc6T";

    #[test]
    fn test_map_assets() {
        let response: NftItemsResponse = serde_json::from_str(include_str!("../../../testdata/ton/items.json")).unwrap();
        let asset_ids = map_assets(&response);

        assert_eq!(asset_ids.len(), 1);
        let first = &asset_ids[0];
        assert_eq!(first.chain, Chain::Ton);
        assert_eq!(first.contract_address, VERIFIED_COLLECTION);
        assert_eq!(first.token_id, ITEM);
        assert_eq!(first.to_string(), format!("ton_{VERIFIED_COLLECTION}::{ITEM}"));
    }

    #[test]
    fn test_map_asset() {
        let response: NftItemsResponse = serde_json::from_str(include_str!("../../../testdata/ton/items.json")).unwrap();
        let asset_id = NFTAssetId::new(Chain::Ton, VERIFIED_COLLECTION, ITEM);
        let asset = map_asset(&response, asset_id).expect("Failed to map asset");

        assert_eq!(asset.id.to_string(), format!("ton_{VERIFIED_COLLECTION}::{ITEM}"));
        assert_eq!(asset.collection_id.to_string(), format!("ton_{VERIFIED_COLLECTION}"));
        assert_eq!(asset.chain, Chain::Ton);
        assert_eq!(asset.token_id, ITEM);
        assert_eq!(asset.contract_address.as_deref(), Some(ITEM));
        assert_eq!(asset.name, "Resolved Item Name");
        assert_eq!(asset.token_type, NFTType::JETTON);
        assert_eq!(asset.images.preview.url, "https://example.com/resolved-item.png");
    }

    #[test]
    fn test_map_asset_unverified_collection() {
        let response: NftItemsResponse = serde_json::from_str(include_str!("../../../testdata/ton/items_unverified.json")).unwrap();
        let asset_id = NFTAssetId::new(Chain::Ton, UNVERIFIED_COLLECTION, ITEM);
        let asset = map_asset(&response, asset_id).unwrap();

        assert_eq!(asset.id, NFTAssetId::new(Chain::Ton, UNVERIFIED_COLLECTION, ITEM));
        assert_eq!(asset.collection_id, NFTCollectionId::new(Chain::Ton, UNVERIFIED_COLLECTION));
        assert_eq!(asset.name, "Unverified Item");
    }

    #[test]
    fn test_map_offchain_asset() {
        let metadata: NftOffchainMetadata = serde_json::from_str(include_str!("../../../testdata/ton/item_offchain.json")).unwrap();
        let asset_id = NFTAssetId::new(Chain::Ton, VERIFIED_COLLECTION, ITEM);
        let asset = map_offchain_asset(metadata, asset_id.clone()).unwrap();

        assert_eq!(asset.id, asset_id);
        assert_eq!(asset.name, "Swag Bag #219028");
        assert_eq!(asset.description.as_deref(), Some("An exclusive Swag Bag by Snoop Dogg."));
        assert_eq!(asset.images.preview.url, "https://nft.fragment.com/gift/swagbag-219028.webp");
    }

    #[test]
    fn test_map_assets_includes_unindexed_metadata() {
        let response: NftItemsResponse = serde_json::from_str(include_str!("../../../testdata/ton/item_unindexed.json")).unwrap();

        assert_eq!(
            map_assets(&response),
            vec![NFTAssetId::new(
                Chain::Ton,
                "EQCgaTxb2wA_3Bi8Ec4FFNu8CauoHo0VPpnwxdrhAgOrOXvA",
                "EQCrhnIgB3ITBJbu4hm0ie8Hm76pdPEsl-1_1wLaRmMQOUTN"
            )]
        );
    }

    #[test]
    fn test_map_collection() {
        let response: NftCollectionsResponse = serde_json::from_str(include_str!("../../../testdata/ton/collections.json")).unwrap();
        let collection_id = NFTCollectionId::new(Chain::Ton, NUMBERS_COLLECTION);
        let collection = map_collection(response, collection_id).expect("Failed to map collection");

        assert_eq!(collection.id.to_string(), format!("ton_{NUMBERS_COLLECTION}"));
        assert_eq!(collection.chain, Chain::Ton);
        assert_eq!(collection.contract_address, NUMBERS_COLLECTION);
        assert_eq!(collection.name, "Anonymous Telegram Numbers");
        assert!(collection.is_verified);
        assert_eq!(collection.status, VerificationStatus::Verified);
    }

    #[test]
    fn test_map_collection_getgems_verified() {
        let response: NftCollectionsResponse = serde_json::from_str(include_str!("../../../testdata/ton/collections_getgems.json")).unwrap();
        let collection_id = NFTCollectionId::new(Chain::Ton, GETGEMS_COLLECTION);
        let collection = map_collection(response, collection_id).unwrap();

        assert_eq!(collection.status, VerificationStatus::Verified);
        assert!(collection.is_verified);
        assert_eq!(collection.links, vec![]);
    }

    #[test]
    fn test_map_collection_rejects_invalid_metadata() {
        let response: NftCollectionsResponse = serde_json::from_str(include_str!("../../../testdata/ton/collections_invalid.json")).unwrap();
        let collection_id = NFTCollectionId::new(Chain::Ton, UNVERIFIED_COLLECTION);
        assert!(map_collection(response, collection_id).is_none());
    }
}

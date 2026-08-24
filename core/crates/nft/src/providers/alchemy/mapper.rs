use gem_alchemy::nft::{Attribute, ContractMetadata, NftMetadata, OwnedNft};
use gem_evm::ethereum_address_checksum;
use primitives::{Chain, NFTAsset, NFTAssetId, NFTAttribute, NFTAttributeType, NFTCollection, NFTCollectionId, NFTImages, NFTResource, VerificationStatus};

use crate::providers::attribute::json_attribute_value;

pub fn map_assets(assets: Vec<OwnedNft>, chain: Chain) -> Vec<NFTAssetId> {
    assets
        .into_iter()
        .filter(|asset| !asset.is_spam.unwrap_or_default())
        .filter_map(|asset| map_asset_id(asset, chain))
        .collect()
}

pub fn map_collection(metadata: ContractMetadata, collection_id: NFTCollectionId) -> NFTCollection {
    let open_sea_metadata = metadata.open_sea_metadata.as_ref();
    let is_spam = metadata.is_spam.unwrap_or_default();
    let is_verified = !is_spam && open_sea_metadata.and_then(|metadata| metadata.safelist_request_status.as_deref()) == Some("verified");
    let status = if is_spam {
        VerificationStatus::Suspicious
    } else {
        VerificationStatus::from_verified(is_verified)
    };

    NFTCollection {
        chain: collection_id.chain,
        contract_address: ethereum_address_checksum(&collection_id.contract_address).unwrap_or_else(|_| collection_id.contract_address.clone()),
        id: collection_id,
        name: metadata
            .name
            .clone()
            .or_else(|| open_sea_metadata.and_then(|metadata| metadata.collection_name.clone()))
            .unwrap_or_default(),
        symbol: metadata.symbol.clone(),
        description: open_sea_metadata.and_then(|metadata| metadata.description.clone()),
        images: NFTImages {
            preview: NFTResource::from_url(open_sea_metadata.and_then(|metadata| metadata.image_url.as_deref()).unwrap_or_default()),
        },
        status,
        links: Vec::new(),
        is_verified,
    }
}

pub fn map_asset(metadata: NftMetadata, asset_id: NFTAssetId) -> Option<NFTAsset> {
    if metadata.contract.is_spam.unwrap_or_default() {
        return None;
    }
    let raw_metadata = metadata.raw.as_ref().and_then(|raw| raw.metadata.as_ref());
    let token_type = metadata
        .token_type
        .as_deref()
        .or(metadata.contract.token_type.as_deref())?
        .to_ascii_lowercase()
        .parse()
        .ok()?;
    let raw_image = raw_metadata.and_then(|metadata| metadata.image.as_deref());
    let attributes = raw_metadata
        .and_then(|metadata| metadata.attributes.as_ref())
        .into_iter()
        .flatten()
        .filter_map(map_attribute)
        .collect();

    Some(NFTAsset {
        chain: asset_id.chain,
        contract_address: Some(asset_id.contract_address.clone()),
        token_id: asset_id.token_id.clone(),
        collection_id: asset_id.get_collection_id(),
        id: asset_id,
        token_type,
        name: metadata
            .name
            .clone()
            .or_else(|| raw_metadata.and_then(|metadata| metadata.name.clone()))
            .unwrap_or_default(),
        description: metadata.description.clone().or_else(|| raw_metadata.and_then(|metadata| metadata.description.clone())),
        resource: NFTResource::from_url(resource_url(&metadata, raw_image)),
        images: NFTImages {
            preview: NFTResource::from_url(preview_url(&metadata, raw_image)),
        },
        attributes,
    })
}

fn map_asset_id(asset: OwnedNft, chain: Chain) -> Option<NFTAssetId> {
    let contract_address = ethereum_address_checksum(&asset.contract_address).ok()?;
    Some(NFTAssetId::new(chain, &contract_address, &asset.token_id))
}

fn resource_url<'a>(metadata: &'a NftMetadata, raw_image: Option<&'a str>) -> &'a str {
    metadata
        .image
        .as_ref()
        .and_then(|image| {
            image
                .original_url
                .as_deref()
                .or(image.cached_url.as_deref())
                .or(image.png_url.as_deref())
                .or(image.thumbnail_url.as_deref())
        })
        .or(raw_image)
        .unwrap_or_default()
}

fn preview_url<'a>(metadata: &'a NftMetadata, raw_image: Option<&'a str>) -> &'a str {
    metadata
        .image
        .as_ref()
        .and_then(|image| {
            image
                .thumbnail_url
                .as_deref()
                .or(image.cached_url.as_deref())
                .or(image.png_url.as_deref())
                .or(image.original_url.as_deref())
        })
        .or(raw_image)
        .unwrap_or_default()
}

fn map_attribute(attribute: &Attribute) -> Option<NFTAttribute> {
    let value = json_attribute_value(&attribute.value)?;
    if value == "None" {
        return None;
    }
    Some(NFTAttribute::new(attribute.trait_type.clone(), value, NFTAttributeType::String))
}

#[cfg(test)]
mod tests {
    use gem_alchemy::nft::{ContractMetadata, NftMetadata, OwnedNftsResponse};

    use super::*;
    use crate::testkit::TEST_BSC_COLLECTION;
    use primitives::NFTType;

    #[test]
    fn test_map_assets() {
        let response: OwnedNftsResponse = serde_json::from_str(include_str!("../../testdata/alchemy/owner_nfts.json")).unwrap();
        let assets = map_assets(response.owned_nfts, Chain::SmartChain);

        assert_eq!(assets, vec![NFTAssetId::new(Chain::SmartChain, "0x6DFBB01ECB7991366Cd8acc4D18dCc67bbe345ba", "410")]);
    }

    #[test]
    fn test_map_collection() {
        let collection_metadata: ContractMetadata = serde_json::from_str(include_str!("../../testdata/alchemy/contract_metadata.json")).unwrap();
        let collection_id = NFTCollectionId::new(Chain::SmartChain, TEST_BSC_COLLECTION);
        let collection = map_collection(collection_metadata, collection_id);

        assert_eq!(collection.name, "Reefers by CoralApp");
        assert_eq!(collection.symbol.as_deref(), Some("CRAPP"));
        assert_eq!(collection.status, VerificationStatus::Unverified);
        assert_eq!(collection.links.len(), 0);
    }

    #[test]
    fn test_map_asset() {
        let asset_metadata: NftMetadata = serde_json::from_str(include_str!("../../testdata/alchemy/nft_metadata.json")).unwrap();
        let asset_id = NFTAssetId::new(Chain::SmartChain, TEST_BSC_COLLECTION, "410");
        let asset = map_asset(asset_metadata, asset_id).unwrap();

        assert_eq!(asset.name, "Reefers by CoralApp #411");
        assert_eq!(asset.token_type, NFTType::ERC721);
        assert_eq!(asset.attributes.len(), 8);
        assert_eq!(asset.resource.url, "https://ipfs.io/ipfs/QmRcRJFFnV7Vi4eq7F8kB4Br8axKxH4pN8pNBxpws4Ga94/411.png");
    }

    #[test]
    fn test_map_spam_asset() {
        let mut metadata: NftMetadata = serde_json::from_str(include_str!("../../testdata/alchemy/nft_metadata.json")).unwrap();
        metadata.contract.is_spam = Some(true);
        let asset_id = NFTAssetId::new(Chain::SmartChain, TEST_BSC_COLLECTION, "410");

        assert!(map_asset(metadata, asset_id).is_none());
    }

    #[test]
    fn test_map_spam_collection_is_not_verified() {
        let mut metadata: ContractMetadata = serde_json::from_str(include_str!("../../testdata/alchemy/contract_metadata.json")).unwrap();
        metadata.is_spam = Some(true);
        metadata.open_sea_metadata.as_mut().unwrap().safelist_request_status = Some("verified".to_string());

        let collection = map_collection(metadata, NFTCollectionId::new(Chain::SmartChain, TEST_BSC_COLLECTION));

        assert_eq!(collection.status, VerificationStatus::Suspicious);
        assert!(!collection.is_verified);
    }
}

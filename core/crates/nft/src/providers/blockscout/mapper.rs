use blockscout::{NftAttribute, NftInstance, NftItem, Token};
use gem_evm::ethereum_address_checksum;
use primitives::{Chain, NFTAsset, NFTAssetId, NFTAttribute, NFTAttributeType, NFTCollection, NFTCollectionId, NFTImages, NFTResource, NFTType, VerificationStatus};

use crate::providers::attribute::json_attribute_value;
use crate::providers::image::is_inline_image;

pub fn map_assets(items: Vec<NftItem>, chain: Chain) -> Vec<NFTAssetId> {
    items
        .into_iter()
        .filter(|item| !is_spam(&item.token))
        .filter_map(|item| {
            let contract_address = ethereum_address_checksum(&item.token.address_hash).ok()?;
            Some(NFTAssetId::new(chain, &contract_address, &item.id))
        })
        .collect()
}

pub fn map_collection(token: Token, collection_id: NFTCollectionId) -> NFTCollection {
    let status = if is_spam(&token) { VerificationStatus::Suspicious } else { VerificationStatus::Unverified };

    NFTCollection {
        chain: collection_id.chain,
        contract_address: ethereum_address_checksum(&collection_id.contract_address).unwrap_or_else(|_| collection_id.contract_address.clone()),
        id: collection_id,
        name: token.name.unwrap_or_default(),
        symbol: token.symbol,
        description: None,
        images: NFTImages {
            preview: NFTResource::from_url(token.icon_url.as_deref().unwrap_or_default()),
        },
        status,
        links: Vec::new(),
    }
}

pub fn map_asset(instance: NftInstance, asset_id: NFTAssetId) -> Option<NFTAsset> {
    let image = instance.image_url.unwrap_or_default();
    if is_spam(&instance.token) || is_inline_image(&image) {
        return None;
    }
    let token_type = nft_type(&instance.token.token_type)?;
    let (name, description, attributes) = instance.metadata.map(|metadata| (metadata.name, metadata.description, metadata.attributes)).unwrap_or_default();

    Some(NFTAsset {
        chain: asset_id.chain,
        contract_address: Some(asset_id.contract_address.clone()),
        token_id: asset_id.token_id.clone(),
        collection_id: asset_id.get_collection_id(),
        id: asset_id,
        token_type,
        name: name.unwrap_or_default(),
        description,
        resource: NFTResource::from_url(&image),
        images: NFTImages { preview: NFTResource::from_url(&image) },
        attributes: attributes.into_iter().flatten().filter_map(map_attribute).collect(),
    })
}

fn nft_type(token_type: &str) -> Option<NFTType> {
    match token_type {
        "ERC-721" => Some(NFTType::ERC721),
        "ERC-1155" => Some(NFTType::ERC1155),
        _ => None,
    }
}

fn is_spam(token: &Token) -> bool {
    token.reputation.as_deref().is_some_and(|reputation| reputation != "ok")
}

fn map_attribute(attribute: NftAttribute) -> Option<NFTAttribute> {
    Some(NFTAttribute::new(attribute.trait_type?, json_attribute_value(&attribute.value)?, NFTAttributeType::String))
}

#[cfg(test)]
mod tests {
    use blockscout::testkit::{ADDRESS_NFTS, NFT_COLLECTION, NFT_INSTANCE};

    use super::*;

    const COLLECTION: &str = "0xB856127c2371B396f92993814d8F64c3204911dE";

    fn items() -> Vec<NftItem> {
        let page: serde_json::Value = serde_json::from_str(ADDRESS_NFTS).unwrap();
        serde_json::from_value(page["items"].clone()).unwrap()
    }

    #[test]
    fn test_map_assets_skips_spam() {
        let mut items = items();
        items[1].token.reputation = Some("scam".to_string());

        assert_eq!(map_assets(items, Chain::Arc), vec![NFTAssetId::new(Chain::Arc, COLLECTION, "202689")]);
    }

    #[test]
    fn test_map_collection() {
        let collection = map_collection(serde_json::from_str(NFT_COLLECTION).unwrap(), NFTCollectionId::new(Chain::Arc, COLLECTION));

        assert_eq!(collection.name, "The Arc Begins");
        assert_eq!(collection.symbol.as_deref(), Some("ARCxOS"));
        assert_eq!(collection.status, VerificationStatus::Unverified);
    }

    #[test]
    fn test_map_asset() {
        let asset = map_asset(serde_json::from_str(NFT_INSTANCE).unwrap(), NFTAssetId::new(Chain::Arc, COLLECTION, "1")).unwrap();

        assert_eq!(asset.name, "The Arc Begins");
        assert_eq!(asset.token_type, NFTType::ERC721);
        assert_eq!(asset.attributes.len(), 6);
        assert_eq!(asset.images.preview.url, "https://dweb.link/ipfs/bafybeidl35yu2hne4jems3wmdqn7vqbda4xdre63bmjgngmiezzw2cwprm");
    }

    #[test]
    fn test_map_asset_leaves_inline_images_to_the_next_provider() {
        let mut instance: NftInstance = serde_json::from_str(NFT_INSTANCE).unwrap();
        instance.image_url = Some("data:image/png;base64,iVBORw0KGgo".to_string());

        assert!(map_asset(instance, NFTAssetId::new(Chain::Arc, COLLECTION, "1")).is_none());
    }
}

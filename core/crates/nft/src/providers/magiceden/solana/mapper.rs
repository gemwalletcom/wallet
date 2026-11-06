use primitives::{Chain, LinkType, NFTAsset, NFTAssetId, NFTAttribute, NFTAttributeType, NFTCollectionId, NFTImages, NFTResource, NFTType, VerificationStatus};

use crate::providers::attribute::json_attribute_value;

use super::model::{Collection, Nft, Trait};

pub fn map_assets(response: Vec<Nft>, chain: Chain) -> Vec<NFTAssetId> {
    response.into_iter().flat_map(|nft| nft.as_asset_id(chain)).collect()
}

pub fn map_collection(collection: Collection, collection_id: NFTCollectionId) -> primitives::NFTCollection {
    collection.as_primitive(collection_id)
}

pub fn map_asset(nft: Nft, asset_id: NFTAssetId, owner: String) -> Option<NFTAsset> {
    nft.as_primitive(asset_id, owner)
}

impl Nft {
    pub fn as_asset_id(&self, chain: Chain) -> Option<NFTAssetId> {
        Some(NFTAssetId::new(chain, self.collection_address(), &self.mint_address))
    }

    pub fn collection_address(&self) -> &str {
        self.collection.as_deref().filter(|collection| !collection.is_empty()).unwrap_or(&self.mint_address)
    }

    pub fn as_primitive(&self, asset: NFTAssetId, owner: String) -> Option<NFTAsset> {
        let traits = self.attributes.clone();
        let collection_id = asset.get_collection_id();
        Some(NFTAsset {
            chain: asset.chain,
            contract_address: Some(owner),
            token_id: asset.token_id.clone(),
            id: asset,
            collection_id,
            token_type: NFTType::SPL,
            name: self.name.clone(),
            description: None,
            resource: NFTResource::from_url(&self.image),
            images: NFTImages { preview: NFTResource::from_url(&self.image) },
            attributes: traits.iter().flat_map(Trait::as_attribute).collect(),
        })
    }
}

impl Trait {
    pub fn as_attribute(&self) -> Option<NFTAttribute> {
        let value = json_attribute_value(&self.value)?;
        Some(NFTAttribute::new(self.trait_type.clone(), value, NFTAttributeType::String))
    }
}

impl Collection {
    pub fn as_primitive(&self, collection: NFTCollectionId) -> primitives::NFTCollection {
        primitives::NFTCollection {
            chain: collection.chain,
            contract_address: self.on_chain_collection_address.clone(),
            id: collection,
            name: self.name.clone(),
            symbol: self.symbol.clone(),
            description: Some(self.description.clone()),
            images: NFTImages { preview: NFTResource::from_url(&self.image) },
            status: self.status(),
            links: self.as_links(),
        }
    }

    fn status(&self) -> VerificationStatus {
        if self.is_blacklisted {
            VerificationStatus::Suspicious
        } else if self.is_unverified {
            VerificationStatus::Unverified
        } else {
            VerificationStatus::Verified
        }
    }

    pub fn as_links(&self) -> Vec<primitives::AssetLink> {
        let mut links = vec![];
        if let Some(x) = self.twitter.clone() {
            links.push(primitives::AssetLink::new(x.as_str(), LinkType::X));
        }
        if let Some(url) = self.website.clone() {
            links.push(primitives::AssetLink::new(url.as_str(), LinkType::Website));
        }
        if let Some(discord) = self.discord.clone() {
            links.push(primitives::AssetLink::new(discord.as_str(), LinkType::Discord));
        }

        links
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::magiceden::solana::model;
    use crate::testkit::{TEST_SOLANA_ADDRESS, TEST_SOLANA_COLLECTION, TEST_SOLANA_COLLECTION_POOKS, TEST_SOLANA_TOKEN_ID};
    use primitives::{Chain, NFTCollectionId};

    #[test]
    fn test_map_assets() {
        let response: Vec<model::Nft> = serde_json::from_str(include_str!("../../../../testdata/magiceden/assets.json")).unwrap();
        let asset_ids = map_assets(response, Chain::Solana);

        assert!(!asset_ids.is_empty());
        if let Some(first_asset) = asset_ids.first() {
            assert_eq!(first_asset.chain, Chain::Solana);
            assert!(!first_asset.contract_address.is_empty());
            assert!(!first_asset.token_id.is_empty());
        }
    }

    #[test]
    fn test_map_assets_without_collection() {
        let response: Vec<model::Nft> = serde_json::from_str(include_str!("../../../../testdata/magiceden/assets_uncollected.json")).unwrap();
        let mint = "W1iLZMsRanht4urj4Vra2JqgeKoLkP5LaPerzpsu8iw";
        for collection in [None, Some(String::new())] {
            let nft = model::Nft { collection, ..response[0].clone() };
            assert_eq!(map_assets(vec![nft], Chain::Solana), vec![NFTAssetId::new(Chain::Solana, mint, mint)]);
        }
    }

    #[test]
    fn test_map_collection_of_an_nft_without_collection() {
        let collection: model::Collection = serde_json::from_str(include_str!("../../../../testdata/magiceden/collection_uncollected.json")).unwrap();
        let mint = "W1iLZMsRanht4urj4Vra2JqgeKoLkP5LaPerzpsu8iw";
        let nft_collection = map_collection(collection, NFTCollectionId::new(Chain::Solana, mint));

        assert_eq!(nft_collection.name, "NFT Coins");
        assert_eq!(nft_collection.contract_address, mint);
        assert_eq!(nft_collection.status, VerificationStatus::Unverified);

        let mut blacklisted: serde_json::Value = serde_json::from_str(include_str!("../../../../testdata/magiceden/collection_uncollected.json")).unwrap();
        blacklisted["isBlacklisted"] = true.into();
        let blacklisted: model::Collection = serde_json::from_value(blacklisted).unwrap();
        assert_eq!(map_collection(blacklisted, NFTCollectionId::new(Chain::Solana, mint)).status, VerificationStatus::Suspicious);
    }

    #[test]
    fn test_map_collection() {
        let collection: model::Collection = serde_json::from_str(include_str!("../../../../testdata/magiceden/collection.json")).unwrap();
        let collection_id = NFTCollectionId::new(Chain::Solana, TEST_SOLANA_COLLECTION);
        let nft_collection = map_collection(collection, collection_id);

        assert_eq!(nft_collection.chain, Chain::Solana);
        assert_eq!(nft_collection.name, "Okay Bears");
        assert_eq!(nft_collection.status, VerificationStatus::Verified);
        assert!(nft_collection.description.is_some());
        assert!(nft_collection.description.as_ref().unwrap().contains("10,000 diverse bears"));
        assert!(!nft_collection.links.is_empty());
        assert!(nft_collection.links.iter().any(|link| link.url.contains("okaybears.com")));
    }

    #[test]
    fn test_map_asset() {
        let nft: model::Nft = serde_json::from_str(include_str!("../../../../testdata/magiceden/asset.json")).unwrap();
        let asset_id = NFTAssetId::new(Chain::Solana, TEST_SOLANA_COLLECTION_POOKS, TEST_SOLANA_TOKEN_ID);
        let owner = TEST_SOLANA_ADDRESS.to_string();
        let nft_asset = map_asset(nft, asset_id, owner.clone()).expect("Failed to map asset");

        assert_eq!(nft_asset.chain, Chain::Solana);
        assert_eq!(nft_asset.token_id, TEST_SOLANA_TOKEN_ID);
        assert_eq!(nft_asset.name, "pooks #3726");
        assert_eq!(nft_asset.contract_address, Some(owner));
        assert!(!nft_asset.attributes.is_empty());

        let background_trait = nft_asset.attributes.iter().find(|attr| attr.name == "Background");
        assert!(background_trait.is_some());
        assert_eq!(background_trait.unwrap().value, "Dewdrop Delight");
    }

    #[test]
    fn test_asset_id_mapping() {
        let response: Vec<model::Nft> = serde_json::from_str(include_str!("../../../../testdata/magiceden/assets.json")).unwrap();
        let asset_ids: Vec<NFTAssetId> = response.into_iter().flat_map(|nft| nft.as_asset_id(Chain::Solana)).collect();

        assert!(!asset_ids.is_empty());

        if let Some(first_asset) = asset_ids.first() {
            assert_eq!(first_asset.chain, Chain::Solana);
            assert!(!first_asset.contract_address.is_empty());
            assert!(!first_asset.token_id.is_empty());
        }
    }

    #[test]
    fn test_asset_primitive_mapping() {
        let nft: model::Nft = serde_json::from_str(include_str!("../../../../testdata/magiceden/asset.json")).unwrap();
        let asset_id = NFTAssetId::new(Chain::Solana, TEST_SOLANA_COLLECTION_POOKS, TEST_SOLANA_TOKEN_ID);
        let owner = TEST_SOLANA_ADDRESS.to_string();

        let nft_asset = nft.as_primitive(asset_id, owner.clone()).expect("Failed to map asset");

        assert_eq!(nft_asset.chain, Chain::Solana);
        assert_eq!(nft_asset.token_id, TEST_SOLANA_TOKEN_ID);
        assert_eq!(nft_asset.name, "pooks #3726");
        assert_eq!(nft_asset.contract_address, Some(owner));
        assert!(!nft_asset.attributes.is_empty());

        let background_trait = nft_asset.attributes.iter().find(|attr| attr.name == "Background");
        assert!(background_trait.is_some());
        assert_eq!(background_trait.unwrap().value, "Dewdrop Delight");
    }

    #[test]
    fn test_collection_primitive_mapping() {
        let collection: model::Collection = serde_json::from_str(include_str!("../../../../testdata/magiceden/collection.json")).unwrap();
        let collection_id = NFTCollectionId::new(Chain::Solana, TEST_SOLANA_COLLECTION);
        let nft_collection = collection.as_primitive(collection_id);

        assert_eq!(nft_collection.chain, Chain::Solana);
        assert_eq!(nft_collection.name, "Okay Bears");
        assert!(nft_collection.description.is_some());
        assert!(nft_collection.description.as_ref().unwrap().contains("10,000 diverse bears"));

        assert!(!nft_collection.links.is_empty());
        assert!(nft_collection.links.iter().any(|link| link.url.contains("okaybears.com")));
        assert!(nft_collection.links.iter().any(|link| link.url.contains("discord.com")));
    }
}

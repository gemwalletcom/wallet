use primitives::{NFTData, VerificationStatus};

pub fn verified_collections(data: Vec<NFTData>) -> Vec<NFTData> {
    collections(data, true)
}

pub fn unverified_collections(data: Vec<NFTData>) -> Vec<NFTData> {
    collections(data, false)
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

pub fn collection_status(status: Option<VerificationStatus>) -> VerificationStatus {
    status.unwrap_or(VerificationStatus::Unverified)
}

fn collections(data: Vec<NFTData>, verified: bool) -> Vec<NFTData> {
    data.into_iter()
        .filter(|item| !item.assets.is_empty() && (item.collection.status == VerificationStatus::Verified) == verified)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Chain, NFTAsset, NFTAssetId, NFTCollection, NFTCollectionId, NFTImages, NFTResource, NFTType};

    #[test]
    fn test_collections_split_by_verification_and_skip_empty_ones() {
        let verified = data("verified", VerificationStatus::Verified, 1);
        let unverified = data("unverified", VerificationStatus::Unverified, 2);
        let empty = data("empty", VerificationStatus::Verified, 0);
        let items = vec![verified, unverified, empty];

        assert_eq!(names(verified_collections(items.clone())), vec!["verified"]);
        assert_eq!(names(unverified_collections(items)), vec!["unverified"]);
    }

    fn names(data: Vec<NFTData>) -> Vec<String> {
        data.into_iter().map(|item| item.collection.name).collect()
    }

    fn images() -> NFTImages {
        NFTImages {
            preview: NFTResource {
                url: String::new(),
                mime_type: String::new(),
            },
        }
    }

    fn data(name: &str, status: VerificationStatus, assets: usize) -> NFTData {
        NFTData {
            collection: NFTCollection {
                id: NFTCollectionId::new(Chain::Ethereum, "0xcollection"),
                name: name.to_string(),
                symbol: None,
                description: None,
                chain: Chain::Ethereum,
                contract_address: "0xcollection".to_string(),
                images: images(),
                is_verified: status == VerificationStatus::Verified,
                status,
                links: vec![],
            },
            assets: (0..assets)
                .map(|index| NFTAsset {
                    id: NFTAssetId::new(Chain::Ethereum, "0xcollection", &index.to_string()),
                    collection_id: NFTCollectionId::new(Chain::Ethereum, "0xcollection"),
                    contract_address: Some("0xcollection".to_string()),
                    token_id: index.to_string(),
                    token_type: NFTType::ERC721,
                    name: name.to_string(),
                    description: None,
                    chain: Chain::Ethereum,
                    resource: NFTResource {
                        url: String::new(),
                        mime_type: String::new(),
                    },
                    images: images(),
                    attributes: vec![],
                })
                .collect(),
        }
    }

    #[test]
    fn test_an_unknown_collection_status_is_not_verified() {
        assert_eq!(collection_status(None), VerificationStatus::Unverified);
        assert_eq!(collection_status(Some(VerificationStatus::Verified)), VerificationStatus::Verified);
    }

    #[test]
    fn test_collections_sort_by_size_then_name() {
        let big = data("zebra", VerificationStatus::Verified, 3);
        let small_a = data("alpha", VerificationStatus::Verified, 1);
        let small_b = data("beta", VerificationStatus::Verified, 1);

        let sorted = sorted_collections(vec![small_b, big, small_a]);

        assert_eq!(names(sorted), vec!["zebra", "alpha", "beta"]);
    }
}

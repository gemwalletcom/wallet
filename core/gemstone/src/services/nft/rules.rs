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
    use primitives::NFTData;

    #[test]
    fn test_collections_split_by_verification_and_skip_empty_ones() {
        let verified = NFTData::mock_with("verified", VerificationStatus::Verified, 1);
        let unverified = NFTData::mock_with("unverified", VerificationStatus::Unverified, 2);
        let empty = NFTData::mock_with("empty", VerificationStatus::Verified, 0);
        let items = vec![verified, unverified, empty];

        assert_eq!(names(verified_collections(items.clone())), vec!["verified"]);
        assert_eq!(names(unverified_collections(items)), vec!["unverified"]);
    }

    fn names(data: Vec<NFTData>) -> Vec<String> {
        data.into_iter().map(|item| item.collection.name).collect()
    }

    #[test]
    fn test_an_unknown_collection_status_is_not_verified() {
        assert_eq!(collection_status(None), VerificationStatus::Unverified);
        assert_eq!(collection_status(Some(VerificationStatus::Verified)), VerificationStatus::Verified);
    }

    #[test]
    fn test_collections_sort_by_size_then_name() {
        let big = NFTData::mock_with("zebra", VerificationStatus::Verified, 3);
        let small_a = NFTData::mock_with("alpha", VerificationStatus::Verified, 1);
        let small_b = NFTData::mock_with("beta", VerificationStatus::Verified, 1);

        let sorted = sorted_collections(vec![small_b, big, small_a]);

        assert_eq!(names(sorted), vec!["zebra", "alpha", "beta"]);
    }
}

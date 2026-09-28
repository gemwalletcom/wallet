use std::str::FromStr;

use primitives::{Chain, asset_score::AssetRank};

const MIN_LIST_SEARCH_QUERY_LENGTH: usize = 2;
const STRICT_RANK_QUERY_LENGTH: usize = 16;
const STRICT_RANK_THRESHOLD: i32 = 5;

pub struct SearchRequest {
    pub query: String,
    pub chains: Vec<String>,
    pub tags: Vec<String>,
    pub limit: usize,
    pub offset: usize,
}

impl SearchRequest {
    pub fn new(query: &str, chains: Option<&str>, tags: Option<&str>, limit: usize, offset: Option<usize>) -> Self {
        let chains = chains.unwrap_or_default().split(',').flat_map(Chain::from_str).map(|x| x.to_string()).collect::<Vec<String>>();

        let tags = tags.unwrap_or_default().split(',').filter(|x| !x.is_empty()).map(ToString::to_string).collect::<Vec<String>>();

        Self {
            query: query.trim().to_string(),
            chains,
            tags,
            limit,
            offset: offset.unwrap_or(0),
        }
    }

    pub fn rank_threshold(&self) -> i32 {
        if self.query.len() < STRICT_RANK_QUERY_LENGTH { AssetRank::Trivial.threshold() } else { STRICT_RANK_THRESHOLD }
    }

    pub fn should_search_lists(&self) -> bool {
        !self.has_tag_filter() && self.query.chars().count() >= MIN_LIST_SEARCH_QUERY_LENGTH
    }

    pub fn has_tag_filter(&self) -> bool {
        !self.tags.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::MAX_QUERY_LIMIT;

    #[test]
    fn rank_threshold() {
        assert_eq!(SearchRequest::new("BTC", None, None, MAX_QUERY_LIMIT, None).rank_threshold(), 15);
        assert_eq!(SearchRequest::new("USDT", None, None, MAX_QUERY_LIMIT, None).rank_threshold(), 15);
        assert_eq!(SearchRequest::new("USDT TRC20", None, None, MAX_QUERY_LIMIT, None).rank_threshold(), 15);
        assert_eq!(SearchRequest::new("ethereum chain", None, None, MAX_QUERY_LIMIT, None).rank_threshold(), 15);
        assert_eq!(SearchRequest::new("ethereum contract", None, None, MAX_QUERY_LIMIT, None).rank_threshold(), 5);
    }

    #[test]
    fn should_search_lists() {
        assert!(!SearchRequest::new("B", None, None, MAX_QUERY_LIMIT, None).should_search_lists());
        assert!(SearchRequest::new("BT", None, None, MAX_QUERY_LIMIT, None).should_search_lists());
        assert!(!SearchRequest::new("stocks", None, Some("stocks"), MAX_QUERY_LIMIT, None).should_search_lists());
    }

    #[test]
    fn has_tag_filter() {
        assert!(!SearchRequest::new("BTC", None, None, MAX_QUERY_LIMIT, None).has_tag_filter());
        assert!(SearchRequest::new("BTC", None, Some("stocks"), MAX_QUERY_LIMIT, None).has_tag_filter());
    }

    #[test]
    fn new() {
        let request = SearchRequest::new(" test ", Some("ethereum,bitcoin"), Some("defi,nft"), MAX_QUERY_LIMIT, Some(10));
        assert_eq!(request.query, "test");
        assert_eq!(request.chains, vec!["ethereum", "bitcoin"]);
        assert_eq!(request.tags, vec!["defi", "nft"]);
        assert_eq!(request.limit, 100);
        assert_eq!(request.offset, 10);
    }
}

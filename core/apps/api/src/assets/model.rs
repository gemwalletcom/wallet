use primitives::Chain;
use std::str::FromStr;

const MAX_LIMIT: usize = 500;
const MAX_OFFSET: usize = 10_000;
const DEFAULT_LIMIT: usize = 50;
const MIN_LIST_SEARCH_QUERY_LENGTH: usize = 3;

pub struct SearchRequest {
    pub query: String,
    pub chains: Vec<String>,
    pub tags: Vec<String>,
    pub limit: usize,
    pub offset: usize,
}

impl SearchRequest {
    pub fn new(query: &str, chains: Option<&str>, tags: Option<&str>, limit: Option<usize>, offset: Option<usize>) -> Self {
        let chains = chains
            .unwrap_or_default()
            .split(',')
            .flat_map(Chain::from_str)
            .map(|x| x.to_string())
            .collect::<Vec<String>>();

        let tags = tags
            .unwrap_or_default()
            .split(',')
            .filter(|x| !x.is_empty())
            .map(|x| x.to_string())
            .collect::<Vec<String>>();

        Self {
            query: query.trim().to_string(),
            chains,
            tags,
            limit: limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT),
            offset: offset.unwrap_or(0).min(MAX_OFFSET),
        }
    }

    pub fn rank_threshold(&self) -> u32 {
        if self.query.len() < 8 { 15 } else { 5 }
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

    #[test]
    fn rank_threshold() {
        assert_eq!(SearchRequest::new("BTC", None, None, None, None).rank_threshold(), 15);
        assert_eq!(SearchRequest::new("USDT", None, None, None, None).rank_threshold(), 15);
        assert_eq!(SearchRequest::new("ethereum", None, None, None, None).rank_threshold(), 5);
    }

    #[test]
    fn should_search_lists() {
        assert!(!SearchRequest::new("BT", None, None, None, None).should_search_lists());
        assert!(SearchRequest::new("BTC", None, None, None, None).should_search_lists());
        assert!(!SearchRequest::new("stocks", None, Some("stocks"), None, None).should_search_lists());
    }

    #[test]
    fn has_tag_filter() {
        assert!(!SearchRequest::new("BTC", None, None, None, None).has_tag_filter());
        assert!(SearchRequest::new("BTC", None, Some("stocks"), None, None).has_tag_filter());
    }

    #[test]
    fn new() {
        let request = SearchRequest::new(" test ", Some("ethereum,bitcoin"), Some("defi,nft"), Some(100), Some(10));
        assert_eq!(request.query, "test");
        assert_eq!(request.chains, vec!["ethereum", "bitcoin"]);
        assert_eq!(request.tags, vec!["defi", "nft"]);
        assert_eq!(request.limit, 100);
        assert_eq!(request.offset, 10);

        let default_request = SearchRequest::new("query", None, None, None, None);
        assert!(default_request.chains.is_empty());
        assert!(default_request.tags.is_empty());
        assert_eq!(default_request.limit, 50);
        assert_eq!(default_request.offset, 0);
    }
}

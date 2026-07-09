use crate::params::{MAX_QUERY_LIMIT, MAX_QUERY_LIMIT_VALIDATION, SearchQueryParam};
use primitives::Chain;
use rocket::FromForm;
use std::str::FromStr;

const MIN_LIST_SEARCH_QUERY_LENGTH: usize = 2;
const STRICT_RANK_QUERY_LENGTH: usize = 16;

#[derive(FromForm)]
pub struct SearchParams<'r> {
    pub(crate) query: SearchQueryParam,
    pub(crate) chains: Option<&'r str>,
    pub(crate) tags: Option<&'r str>,
    #[field(default = MAX_QUERY_LIMIT, validate = range(..=MAX_QUERY_LIMIT_VALIDATION))]
    pub(crate) limit: usize,
    pub(crate) offset: Option<usize>,
}

pub struct SearchRequest {
    pub query: String,
    pub chains: Vec<String>,
    pub tags: Vec<String>,
    pub limit: usize,
    pub offset: usize,
}

impl SearchRequest {
    pub fn new(query: &str, chains: Option<&str>, tags: Option<&str>, limit: usize, offset: Option<usize>) -> Self {
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
            limit,
            offset: offset.unwrap_or(0),
        }
    }

    pub fn rank_threshold(&self) -> u32 {
        if self.query.len() < STRICT_RANK_QUERY_LENGTH { 15 } else { 5 }
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
    use rocket::form::Form;

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

    #[test]
    fn search_params_defaults_limit() {
        let params = Form::<SearchParams<'_>>::parse("query=btc").unwrap();

        assert_eq!(params.limit, MAX_QUERY_LIMIT);
    }

    #[test]
    fn search_params_accepts_max_limit() {
        let query = format!("query=btc&limit={MAX_QUERY_LIMIT}");
        let params = Form::<SearchParams<'_>>::parse(&query).unwrap();

        assert_eq!(params.limit, MAX_QUERY_LIMIT);
    }

    #[test]
    fn search_params_rejects_limit_above_max() {
        let query = format!("query=btc&limit={}", MAX_QUERY_LIMIT + 1);
        let result = Form::<SearchParams<'_>>::parse(&query);

        assert!(result.is_err());
    }
}

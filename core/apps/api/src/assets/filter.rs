use super::SearchRequest;
use serde_json::Value;

pub fn build_assets_filters(request: &SearchRequest) -> Vec<String> {
    let mut filters = vec![];
    filters.push("properties.isEnabled = true".to_string());
    filters.push(format!("score.rank > {}", request.rank_threshold()));

    if request.has_tag_filter() {
        filters.push(filter_array("tags", request.tags.clone()));
    }

    if !request.chains.is_empty() {
        filters.push(filter_array("asset.chain", request.chains.clone()));
    }

    filters
}

pub fn build_perpetuals_filters(request: &SearchRequest) -> Vec<String> {
    if request.has_tag_filter() {
        vec![filter_array("tags", request.tags.clone())]
    } else {
        vec![]
    }
}

pub fn build_filter(filters: Vec<String>) -> String {
    filters.join(" AND ")
}

fn filter_array(field: &str, values: Vec<String>) -> String {
    let values = values.into_iter().map(filter_string).collect::<Vec<_>>().join(",");
    format!("{field} IN [{values}]")
}

fn filter_string(value: String) -> String {
    Value::String(value).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::MAX_QUERY_LIMIT;

    #[test]
    fn build_assets_filters_short_query() {
        let request = SearchRequest::new("USDT TRC20", None, None, MAX_QUERY_LIMIT, None);
        let filters = build_assets_filters(&request);

        assert_eq!(filters, vec!["properties.isEnabled = true", "score.rank > 15"]);
    }

    #[test]
    fn build_assets_filters_long_query() {
        let request = SearchRequest::new("ethereum contract", None, None, MAX_QUERY_LIMIT, None);
        let filters = build_assets_filters(&request);

        assert_eq!(filters, vec!["properties.isEnabled = true", "score.rank > 5"]);
    }

    #[test]
    fn build_assets_filters_with_tags() {
        let request = SearchRequest::new("ethereum contract", None, Some("defi"), MAX_QUERY_LIMIT, None);
        let filters = build_assets_filters(&request);

        assert_eq!(filters, vec!["properties.isEnabled = true", "score.rank > 5", "tags IN [\"defi\"]"]);
    }

    #[test]
    fn build_assets_filters_with_chains() {
        let request = SearchRequest::new("ethereum contract", Some("ethereum"), None, MAX_QUERY_LIMIT, None);
        let filters = build_assets_filters(&request);

        assert_eq!(filters, vec!["properties.isEnabled = true", "score.rank > 5", "asset.chain IN [\"ethereum\"]"]);
    }

    #[test]
    fn build_perpetuals_filters_with_tags() {
        let request = SearchRequest::new("longquery", None, Some("stocks"), MAX_QUERY_LIMIT, None);
        let filters = build_perpetuals_filters(&request);

        assert_eq!(filters, vec!["tags IN [\"stocks\"]"]);
    }

    #[test]
    fn build_filter_joins_with_and() {
        assert_eq!(build_filter(vec!["a".to_string(), "b".to_string()]), "a AND b");
    }

    #[test]
    fn filter_array_formats_correctly() {
        assert_eq!(filter_array("tags", vec!["defi".to_string(), "nft".to_string()]), "tags IN [\"defi\",\"nft\"]");
    }

    #[test]
    fn filter_array_escapes_filter_literals() {
        let filter = filter_array("tags", vec!["x\"] OR properties.isEnabled = false OR tags IN [\"y".to_string()]);

        assert_eq!(filter, r#"tags IN ["x\"] OR properties.isEnabled = false OR tags IN [\"y"]"#);
    }
}

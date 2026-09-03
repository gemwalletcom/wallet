use serde::Serialize;

pub fn build_request_url(base_url: &str, path: &str) -> String {
    let base_url = base_url.trim_end_matches('/');

    if path.is_empty() {
        base_url.to_string()
    } else if path.starts_with('/') {
        format!("{base_url}{path}")
    } else {
        format!("{base_url}/{path}")
    }
}

pub fn build_path_with_query<T: Serialize + ?Sized>(path: &str, query: &T) -> String {
    let query = serde_urlencoded::to_string(query).unwrap_or_default();
    if query.is_empty() {
        path.to_string()
    } else if path.contains('?') {
        format!("{path}&{query}")
    } else {
        format!("{path}?{query}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[test]
    fn test_build_request_url() {
        assert_eq!(build_request_url("https://example.com/", "/path"), "https://example.com/path");
        assert_eq!(build_request_url("https://example.com", "path"), "https://example.com/path");
        assert_eq!(build_request_url("https://example.com/rpc-key", ""), "https://example.com/rpc-key");
        assert_eq!(build_request_url("https://example.com/rpc-key/", ""), "https://example.com/rpc-key");
    }

    #[derive(Serialize)]
    struct CoinQuery {
        pub market_data: bool,
        pub community_data: bool,
        pub tickers: bool,
        pub localization: bool,
        pub developer_data: bool,
    }

    #[test]
    fn test_build_path_with_query_coingecko_case() {
        let id = "bitcoin";
        let query = CoinQuery {
            market_data: false,
            community_data: true,
            tickers: false,
            localization: true,
            developer_data: true,
        };
        let base_path = format!("/api/v3/coins/{}", id);
        let result = build_path_with_query(&base_path, &query);

        let expected = "/api/v3/coins/bitcoin?market_data=false&community_data=true&tickers=false&localization=true&developer_data=true";
        assert_eq!(result, expected);
        assert_eq!(build_path_with_query(&result, &[("apikey", "key")]), format!("{expected}&apikey=key"));
        assert_eq!(build_path_with_query("/path", &[] as &[(&str, &str)]), "/path");
    }
}

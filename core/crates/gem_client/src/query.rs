use serde::Serialize;

pub fn build_request_url(base_url: &str, path: &str) -> String {
    format!("{}/{}", base_url.trim_end_matches('/'), path.trim_start_matches('/'))
}

/// Build a path with query parameters from a serializable struct
pub fn build_path_with_query<T: Serialize>(path: &str, query: &T) -> Result<String, serde_urlencoded::ser::Error> {
    let query_string = serde_urlencoded::to_string(query)?;
    Ok(format!("{}?{}", path, query_string))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[test]
    fn test_build_request_url() {
        assert_eq!(build_request_url("https://example.com/", "/path"), "https://example.com/path");
        assert_eq!(build_request_url("https://example.com", "path"), "https://example.com/path");
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
        let result = build_path_with_query(&base_path, &query).unwrap();

        let expected = "/api/v3/coins/bitcoin?market_data=false&community_data=true&tickers=false&localization=true&developer_data=true";
        assert_eq!(result, expected);
    }
}

use crate::proxy::proxy_request::ProxyRequest;
use primitives::Chain;
use reqwest::{Method, header::HeaderMap};
use rocket::http::Status;
use url::Url;

pub struct ProxyRequestBuilder;

impl ProxyRequestBuilder {
    pub fn build(method: Method, headers: HeaderMap, body: Vec<u8>, uri: String, chain: Chain) -> Result<ProxyRequest, Status> {
        let host = Self::extract_host(&headers)?;
        let user_agent = Self::extract_user_agent(&headers);
        let (path, path_with_query) = Self::prepare_paths(&uri);

        Ok(ProxyRequest::new(method, headers, body, path, path_with_query, host, user_agent, chain))
    }

    fn extract_host(headers: &HeaderMap) -> Result<String, Status> {
        let host_header = headers.get(reqwest::header::HOST).and_then(|h| h.to_str().ok()).ok_or(Status::BadRequest)?;

        Ok(Self::parse_hostname(host_header))
    }

    fn extract_user_agent(headers: &HeaderMap) -> String {
        headers.get(reqwest::header::USER_AGENT).and_then(|h| h.to_str().ok()).unwrap_or_default().to_string()
    }

    fn extract_path(uri: &str) -> String {
        uri.split('?').next().unwrap_or(uri).to_string()
    }

    fn prepare_paths(uri: &str) -> (String, String) {
        let path_with_query = Self::canonicalize_path(&Self::remove_chain_from_path(uri));
        let path = Self::extract_path(&path_with_query);
        (path, path_with_query)
    }

    fn canonicalize_path(path_with_query: &str) -> String {
        Url::parse("http://0.0.0.0")
            .and_then(|base| base.join(path_with_query))
            .map(|resolved| match resolved.query() {
                Some(query) => format!("{}?{}", resolved.path(), query),
                None => resolved.path().to_string(),
            })
            .unwrap_or_else(|_| path_with_query.to_string())
    }

    fn parse_hostname(host_header: &str) -> String {
        let candidate = format!("http://{}", host_header);
        Url::parse(&candidate)
            .ok()
            .and_then(|url| url.host_str().map(str::to_string))
            .unwrap_or_else(|| host_header.to_string())
    }

    fn remove_chain_from_path(uri: &str) -> String {
        let (path_part, query_part) = uri.split_once('?').unwrap_or((uri, ""));

        let remaining = path_part
            .trim_start_matches('/')
            .split_once('/')
            .map(|(_, rest)| format!("/{}", rest))
            .unwrap_or_else(|| "/".to_string());

        if query_part.is_empty() { remaining } else { format!("{}?{}", remaining, query_part) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_chain_from_path() {
        assert_eq!(ProxyRequestBuilder::remove_chain_from_path("/tron/wallet/getchainparameters"), "/wallet/getchainparameters");
        assert_eq!(ProxyRequestBuilder::remove_chain_from_path("/ethereum/v1/some/path"), "/v1/some/path");
        assert_eq!(ProxyRequestBuilder::remove_chain_from_path("/bitcoin"), "/");
        assert_eq!(ProxyRequestBuilder::remove_chain_from_path("/solana?query=1"), "/?query=1");
        assert_eq!(ProxyRequestBuilder::remove_chain_from_path("/chain/path?foo=bar&baz=qux"), "/path?foo=bar&baz=qux");
    }

    #[test]
    fn test_prepare_paths() {
        assert_eq!(
            ProxyRequestBuilder::prepare_paths("/bitcoin/api/v2/address/../block/900000"),
            ("/api/v2/block/900000".to_string(), "/api/v2/block/900000".to_string())
        );
        assert_eq!(
            ProxyRequestBuilder::prepare_paths("/bitcoin/api/v2/address/%2e%2e/block"),
            ("/api/v2/block".to_string(), "/api/v2/block".to_string())
        );
        assert_eq!(ProxyRequestBuilder::prepare_paths("/ethereum/../secret"), ("/secret".to_string(), "/secret".to_string()));
        assert_eq!(
            ProxyRequestBuilder::prepare_paths("/bitcoin/api/v2/address/bc1qtest?page=1"),
            ("/api/v2/address/bc1qtest".to_string(), "/api/v2/address/bc1qtest?page=1".to_string())
        );
    }

    #[test]
    fn test_parse_hostname() {
        assert_eq!(ProxyRequestBuilder::parse_hostname("example.com"), "example.com");
        assert_eq!(ProxyRequestBuilder::parse_hostname("example.com:8080"), "example.com");
        assert_eq!(ProxyRequestBuilder::parse_hostname("localhost:3000"), "localhost");
    }
}

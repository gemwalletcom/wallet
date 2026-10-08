use std::collections::HashMap;

use bytes::Bytes;
use reqwest::header::{HeaderMap, HeaderName};
use reqwest::{Method, Request, Url as ReqwestUrl};
use url::ParseError;

use crate::config::{ChainConfig, Url};
use crate::jsonrpc_types::{JsonRpcRequest, RequestType};
use crate::proxy::ProxyRequest;

#[derive(Debug)]
pub struct RequestUrl {
    pub url: ReqwestUrl,
    pub headers: HashMap<String, String>,
}

impl RequestUrl {
    pub(crate) fn for_request(request: &ProxyRequest, active_url: &Url, chain_config: &ChainConfig) -> Result<Self, ParseError> {
        let rpc_method = match request.request_type() {
            RequestType::JsonRpc(JsonRpcRequest::Single(call)) => Some(call.method.as_str()),
            RequestType::JsonRpc(JsonRpcRequest::Batch(_)) | RequestType::Regular { .. } => None,
        };
        Self::from_parts(chain_config.url_for_request(active_url, rpc_method, Some(&request.path)), &request.path_with_query)
    }

    pub fn build_request(&self, method: &Method, body: Bytes, mut headers: HeaderMap) -> Request {
        for (name, value) in &self.headers {
            if let (Ok(name), Ok(value)) = (HeaderName::from_bytes(name.as_bytes()), value.parse()) {
                headers.append(name, value);
            }
        }
        let mut request = Request::new(method.clone(), self.url.clone());
        *request.headers_mut() = headers;
        *request.body_mut() = Some(body.into());
        request
    }

    pub fn from_parts(url: Url, original_path_and_query: &str) -> Result<RequestUrl, ParseError> {
        let path = if original_path_and_query == "/" { "" } else { original_path_and_query };
        let resolved = ReqwestUrl::parse(&format!("{}{}", url.url, path))?;

        Ok(RequestUrl {
            url: resolved,
            headers: url.headers.unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use primitives::Chain;
    use reqwest::header::{CONTENT_TYPE, HeaderValue};

    use super::*;
    use crate::config::Override;
    use crate::proxy::constants::JSON_CONTENT_TYPE;

    #[test]
    fn test_for_request_resolves_effective_host_without_credentials() {
        let active = Url::mock("https://user:credential@node.example/secret-key?apikey=secret");
        let config = ChainConfig {
            overrides: Some(vec![
                Override {
                    rpc_method: Some("eth_call".to_string()),
                    path: None,
                    url: "https://relay.example/another-secret-key".to_string(),
                },
                Override {
                    rpc_method: None,
                    path: Some("/rest".to_string()),
                    url: "https://rest.example/rest-secret-key".to_string(),
                },
            ]),
            ..ChainConfig::mock(Chain::Ethereum)
        };
        for (request, expected_host) in [
            (ProxyRequest::mock_jsonrpc(Chain::Ethereum, "eth_call"), "relay.example"),
            (ProxyRequest::mock_jsonrpc(Chain::Ethereum, "eth_blockNumber"), "node.example"),
            (ProxyRequest::mock(Chain::Ethereum, Method::POST, "/rest", &[]), "rest.example"),
        ] {
            let resolved = RequestUrl::for_request(&request, &active, &config).unwrap();
            assert_eq!(resolved.url.host_str(), Some(expected_host));
        }
    }

    #[test]
    fn test_from_uri() {
        let url = Url {
            headers: Some(HashMap::new()),
            ..Url::mock("https://example.com")
        };
        let request_url = RequestUrl::from_parts(url, "/path").unwrap();
        assert_eq!(request_url.url.to_string(), "https://example.com/path");
        assert!(request_url.headers.is_empty());
    }

    #[test]
    fn test_from_uri_with_headers() {
        let headers = HashMap::from([("x-api-key".to_string(), "secret".to_string())]);
        let url = Url {
            headers: Some(headers),
            ..Url::mock("https://example.com")
        };
        let request_url = RequestUrl::from_parts(url, "/path").unwrap();
        assert_eq!(request_url.headers.get("x-api-key"), Some(&"secret".to_string()));
    }

    #[test]
    fn test_from_parts_preserves_configured_path_and_query() {
        for (base, path, expected) in [
            ("https://example.com/rpc/key", "/", "https://example.com/rpc/key"),
            ("https://example.com/rpc?key=credential", "/", "https://example.com/rpc?key=credential"),
            ("https://example.com/rpc", "/blocks/%2F?before=one%2Btwo&before=three", "https://example.com/rpc/blocks/%2F?before=one%2Btwo&before=three"),
        ] {
            assert_eq!(RequestUrl::from_parts(Url::mock(base), path).unwrap().url.as_str(), expected);
        }
    }
    #[test]
    fn test_from_parts_rejects_an_unparsable_url() {
        assert_eq!(RequestUrl::from_parts(Url::mock("rpc"), "/path").unwrap_err(), ParseError::RelativeUrlWithoutBase);
    }

    #[test]
    fn test_build_with_headers() {
        let req_url = RequestUrl::from_parts(
            Url {
                headers: Some(HashMap::from([("x-api-key".to_string(), "secret".to_string())])),
                ..Url::mock("https://example.com")
            },
            "/rpc",
        )
        .unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static(JSON_CONTENT_TYPE));

        let req = req_url.build_request(&Method::POST, Bytes::from_static(b"{}"), headers);

        assert_eq!(req.method(), &Method::POST);
        assert_eq!(req.url().to_string(), "https://example.com/rpc");

        let headers = req.headers();
        assert_eq!(headers.get(CONTENT_TYPE).unwrap(), &HeaderValue::from_static(JSON_CONTENT_TYPE));
        assert_eq!(headers.get("x-api-key").unwrap(), &HeaderValue::from_str("secret").unwrap());
    }

    #[test]
    fn test_build_appends_configured_headers() {
        let url = RequestUrl::from_parts(
            Url {
                headers: Some(HashMap::from([("x-api-key".to_string(), "configured".to_string())])),
                ..Url::mock("https://example.com")
            },
            "/rpc",
        )
        .unwrap();
        let headers = HeaderMap::from_iter([(HeaderName::from_static("x-api-key"), HeaderValue::from_static("inbound"))]);
        let request = url.build_request(&Method::POST, Bytes::new(), headers);

        assert_eq!(request.headers().get_all("x-api-key").iter().map(|value| value.to_str().unwrap()).collect::<Vec<_>>(), vec!["inbound", "configured"]);
    }
    #[test]
    fn test_build_request_preserves_wire_data() {
        let url = ReqwestUrl::parse("https://example.com/rpc/%2F?key=one%2Btwo&key=three").unwrap();
        let headers = HeaderMap::from_iter([(CONTENT_TYPE, HeaderValue::from_static("application/grpc+proto"))]);
        let body = Bytes::from_static(&[0, 0xff, 0x80, b'\n']);
        let request_url = RequestUrl { url: url.clone(), headers: HashMap::new() };
        let request = request_url.build_request(&Method::POST, body.clone(), headers.clone());

        assert_eq!(request.method(), Method::POST);
        assert_eq!(request.url(), &url);
        assert_eq!(request.headers(), &headers);
        assert_eq!(request.body().unwrap().as_bytes(), Some(&body[..]));
    }
}

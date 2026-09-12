use std::collections::HashMap;

use reqwest::header::{HeaderMap, HeaderName};
use reqwest::{Method, Request, Url as ReqwestUrl};

use crate::config::Url;

#[derive(Debug)]
pub struct RequestUrl {
    pub url: ReqwestUrl,
    pub headers: HashMap<String, String>,
}

impl RequestUrl {
    pub fn build_request(&self, method: &Method, body: Vec<u8>, mut headers: HeaderMap) -> Request {
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

    pub fn from_parts(url: Url, original_path_and_query: &str) -> RequestUrl {
        let path = if original_path_and_query == "/" { "" } else { original_path_and_query };
        let combined = format!("{}{}", url.url, path);
        let resolved = ReqwestUrl::parse(&combined).expect("invalid url");

        RequestUrl {
            url: resolved,
            headers: url.headers.unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use reqwest::header::{CONTENT_TYPE, HeaderValue};

    use super::*;
    use crate::proxy::constants::JSON_CONTENT_TYPE;

    #[test]
    fn test_from_uri() {
        let url = Url {
            url: "https://example.com".to_string(),
            headers: Some(HashMap::new()),
        };
        let request_url = RequestUrl::from_parts(url, "/path");
        assert_eq!(request_url.url.to_string(), "https://example.com/path");
        assert!(request_url.headers.is_empty());
    }

    #[test]
    fn test_from_uri_with_headers() {
        let headers = HashMap::from([("x-api-key".to_string(), "secret".to_string())]);
        let url = Url {
            url: "https://example.com".to_string(),
            headers: Some(headers),
        };
        let request_url = RequestUrl::from_parts(url, "/path");
        assert_eq!(request_url.headers.get("x-api-key"), Some(&"secret".to_string()));
    }

    #[test]
    fn test_from_parts_preserves_configured_path_and_query() {
        for (base, path, expected) in [
            ("https://example.com/rpc/key", "/", "https://example.com/rpc/key"),
            ("https://example.com/rpc?key=credential", "/", "https://example.com/rpc?key=credential"),
            (
                "https://example.com/rpc",
                "/blocks/%2F?before=one%2Btwo&before=three",
                "https://example.com/rpc/blocks/%2F?before=one%2Btwo&before=three",
            ),
        ] {
            let url = Url {
                url: base.to_string(),
                headers: None,
            };
            assert_eq!(RequestUrl::from_parts(url, path).url.as_str(), expected);
        }
    }
    fn request_url(base: &str, path: &str, header: Option<(&str, &str)>) -> RequestUrl {
        RequestUrl::from_parts(
            Url {
                url: base.to_string(),
                headers: header.map(|(name, value)| HashMap::from([(name.to_string(), value.to_string())])),
            },
            path,
        )
    }

    #[test]
    fn test_build_with_headers() {
        let req_url = request_url("https://example.com", "/rpc", Some(("x-api-key", "secret")));
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static(JSON_CONTENT_TYPE));

        let req = req_url.build_request(&Method::POST, b"{}".to_vec(), headers);

        assert_eq!(req.method(), &Method::POST);
        assert_eq!(req.url().to_string(), "https://example.com/rpc");

        let headers = req.headers();
        assert_eq!(headers.get(CONTENT_TYPE).unwrap(), &HeaderValue::from_static(JSON_CONTENT_TYPE));
        assert_eq!(headers.get("x-api-key").unwrap(), &HeaderValue::from_str("secret").unwrap());
    }

    #[test]
    fn test_build_appends_configured_headers() {
        let url = request_url("https://example.com", "/rpc", Some(("x-api-key", "configured")));
        let headers = HeaderMap::from_iter([(HeaderName::from_static("x-api-key"), HeaderValue::from_static("inbound"))]);
        let request = url.build_request(&Method::POST, Vec::new(), headers);

        assert_eq!(
            request.headers().get_all("x-api-key").iter().map(|value| value.to_str().unwrap()).collect::<Vec<_>>(),
            vec!["inbound", "configured"]
        );
    }
    #[test]
    fn test_build_request_preserves_wire_data() {
        let url = ReqwestUrl::parse("https://example.com/rpc/%2F?key=one%2Btwo&key=three").unwrap();
        let headers = HeaderMap::from_iter([(CONTENT_TYPE, HeaderValue::from_static("application/grpc+proto"))]);
        let body = vec![0, 0xff, 0x80, b'\n'];
        let request_url = RequestUrl {
            url: url.clone(),
            headers: HashMap::new(),
        };
        let request = request_url.build_request(&Method::POST, body.clone(), headers.clone());

        assert_eq!(request.method(), Method::POST);
        assert_eq!(request.url(), &url);
        assert_eq!(request.headers(), &headers);
        assert_eq!(request.body().unwrap().as_bytes(), Some(body.as_slice()));
    }
}

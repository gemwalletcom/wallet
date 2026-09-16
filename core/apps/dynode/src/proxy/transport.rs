use std::collections::HashSet;

use reqwest::header::{HeaderMap, HeaderName};
use reqwest::{Client, Error as RequestError, Request};

use super::ProxyResponse;

#[derive(Debug)]
pub(crate) enum TransportError {
    Transport(RequestError),
    ResponseBody(RequestError),
}

impl TransportError {
    pub(crate) fn into_inner(self) -> RequestError {
        match self {
            Self::Transport(error) | Self::ResponseBody(error) => error,
        }
    }

    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Self::Transport(_) => "transport",
            Self::ResponseBody(_) => "response_body",
        }
    }
}

pub(crate) fn filter_headers(headers: &HeaderMap, forward_headers: &HashSet<HeaderName>) -> HeaderMap {
    headers
        .iter()
        .filter(|(name, _)| forward_headers.contains(*name))
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect()
}

pub(crate) async fn send(client: &Client, request: Request) -> Result<ProxyResponse, TransportError> {
    let response = client.execute(request).await.map_err(TransportError::Transport)?;
    let status = response.status().as_u16();
    let headers = response.headers().clone();
    let body = response.bytes().await.map_err(TransportError::ResponseBody)?.to_vec();
    Ok(ProxyResponse::new(status, headers, body))
}

#[cfg(test)]
mod tests {
    use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderValue};

    use super::*;

    #[test]
    fn test_filter_headers_preserves_repeated_values() {
        let mut inbound = HeaderMap::new();
        inbound.append(ACCEPT, HeaderValue::from_static("application/json"));
        inbound.append(ACCEPT, HeaderValue::from_static("application/grpc"));
        inbound.insert(AUTHORIZATION, HeaderValue::from_static("Bearer caller"));
        let mut expected = HeaderMap::new();
        expected.append(ACCEPT, HeaderValue::from_static("application/json"));
        expected.append(ACCEPT, HeaderValue::from_static("application/grpc"));

        assert_eq!(filter_headers(&inbound, &HashSet::from([ACCEPT])), expected);
        assert_eq!(filter_headers(&inbound, &HashSet::new()), HeaderMap::new());
    }
}

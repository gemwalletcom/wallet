use std::collections::HashSet;
use std::{error::Error, fmt};

use reqwest::header::{HeaderMap, HeaderName};
use reqwest::{Client, Error as RequestError, Request};

use super::ProxyResponse;

#[derive(Debug)]
pub(crate) enum TransportError {
    Transport(RequestError),
    ResponseBody(RequestError),
}

impl TransportError {
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Self::Transport(_) => "transport",
            Self::ResponseBody(_) => "response_body",
        }
    }

    fn cause(&self) -> Option<&(dyn Error + 'static)> {
        let (Self::Transport(error) | Self::ResponseBody(error)) = self;
        std::iter::successors(error.source(), |error| (*error).source()).last()
    }
}

impl fmt::Display for TransportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.kind())?;
        match self.cause() {
            Some(cause) => write!(formatter, " cause={cause}"),
            None => Ok(()),
        }
    }
}

impl Error for TransportError {}

pub(crate) fn filter_headers(headers: &HeaderMap, forward_headers: &HashSet<HeaderName>) -> HeaderMap {
    headers.iter().filter(|(name, _)| forward_headers.contains(*name)).map(|(name, value)| (name.clone(), value.clone())).collect()
}

pub(crate) async fn send(client: &Client, request: Request) -> Result<ProxyResponse, TransportError> {
    let response = client.execute(request).await.map_err(TransportError::Transport)?;
    let status = response.status().as_u16();
    let headers = response.headers().clone();
    let body = response.bytes().await.map_err(TransportError::ResponseBody)?;
    Ok(ProxyResponse::new(status, headers, body))
}

#[cfg(test)]
mod tests {
    use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderValue};
    use tokio::net::TcpListener;

    use super::*;

    #[tokio::test]
    async fn test_send_reports_closed_connection_without_url() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            socket.readable().await.unwrap();
            socket.try_read(&mut [0; 1024]).unwrap();
        });
        let client = Client::new();
        let request = client.get(format!("http://{address}/secret-key?apikey=secret")).build().unwrap();

        let error = send(&client, request).await.err().unwrap();

        assert_eq!(error.to_string(), "transport cause=connection closed before message completed");
    }

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

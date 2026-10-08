use std::error::Error;

use gem_client::ClientError;
use reqwest::Error as RequestError;
use strum::Display;

use crate::proxy::jsonrpc::error::ResponseError;
use crate::proxy::transport::TransportError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum FailureReason {
    #[strum(to_string = "status={0}")]
    Status(u16),
    Timeout,
    ConnectError,
    Transport,
    ResponseBody,
    ResponseDecode,
    InvalidRpcBatch,
    RequestError,
}

impl FailureReason {
    pub(crate) fn from_error(error: &(dyn Error + Send + Sync + 'static)) -> Self {
        if let Some(error) = error.downcast_ref::<ResponseError>() {
            if !(200..300).contains(&error.status()) {
                return Self::Status(error.status());
            }
            return match error {
                ResponseError::Decode { .. } => Self::ResponseDecode,
                ResponseError::InvalidBatch { .. } => Self::InvalidRpcBatch,
            };
        }
        if let Some(error) = error.downcast_ref::<TransportError>() {
            return match error {
                TransportError::Transport(error) => Self::from_request_error(error),
                TransportError::ResponseBody(error) if error.is_timeout() => Self::Timeout,
                TransportError::ResponseBody(_) => Self::ResponseBody,
            };
        }
        if let Some(error) = error.downcast_ref::<ClientError>() {
            return match error {
                ClientError::Timeout => Self::Timeout,
                ClientError::Http { status, .. } | ClientError::Response { status, .. } => Self::Status(*status),
                ClientError::Network(_) => Self::Transport,
                ClientError::Serialization(_) => Self::ResponseDecode,
            };
        }

        match error.downcast_ref::<RequestError>() {
            Some(error) => Self::from_request_error(error),
            None => Self::RequestError,
        }
    }

    fn from_request_error(error: &RequestError) -> Self {
        if error.is_timeout() {
            Self::Timeout
        } else if error.is_connect() {
            Self::ConnectError
        } else if error.is_body() || error.is_decode() {
            Self::ResponseBody
        } else {
            Self::Transport
        }
    }

    pub(crate) fn error_detail(error: &(dyn Error + Send + Sync + 'static)) -> String {
        if let Some(error) = error.downcast_ref::<ResponseError>() {
            return error.to_string();
        }
        if let Some(error) = error.downcast_ref::<TransportError>() {
            return error.to_string();
        }
        Self::from_error(error).to_string()
    }
}

#[cfg(test)]
mod tests {
    use reqwest::Client;

    use super::*;

    #[test]
    fn client_errors_map_to_reasons() {
        let cases: [(ClientError, FailureReason); 4] = [
            (ClientError::Timeout, FailureReason::Timeout),
            (ClientError::Http { status: 503, body: Vec::new() }, FailureReason::Status(503)),
            (ClientError::Network("request failed".to_string()), FailureReason::Transport),
            (ClientError::Serialization("missing field `result` at line 1 column 2".to_string()), FailureReason::ResponseDecode),
        ];

        for (error, expected) in cases {
            assert_eq!(FailureReason::from_error(&error), expected);
        }
    }

    #[test]
    fn test_transport_errors_preserve_stage_without_exposing_credentials() {
        let client = Client::new();
        let error = client.get("https://node.example/secret-key?apikey=secret").header("invalid header", "value").build().unwrap_err();
        let transport = TransportError::Transport(error);
        assert_eq!(FailureReason::from_error(&transport), FailureReason::Transport);
        assert_eq!(FailureReason::error_detail(&transport), "transport");
        assert_eq!(transport.to_string(), "transport");

        let error = client.get("https://node.example/secret-key?apikey=secret").header("invalid header", "value").build().unwrap_err();
        let body = TransportError::ResponseBody(error);
        assert_eq!(FailureReason::from_error(&body), FailureReason::ResponseBody);
        assert_eq!(FailureReason::error_detail(&body), "response_body");
        assert_eq!(body.to_string(), "response_body");

        let error = ClientError::Network("https://node.example/secret-key?apikey=secret".into());
        assert_eq!(FailureReason::error_detail(&error), "transport");
    }
}

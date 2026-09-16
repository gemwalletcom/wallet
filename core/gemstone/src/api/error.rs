use gem_client::ClientError;
use primitives::ResponseError;

#[derive(Debug, uniffi::Error)]
pub enum GemApiError {
    Network { msg: String },
    Timeout,
    Http { status: u16, msg: String },
    Response { status: u16, msg: String },
    Serialization { msg: String },
}

impl std::fmt::Display for GemApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network { msg } | Self::Serialization { msg } => write!(f, "{msg}"),
            Self::Timeout => write!(f, "request timed out"),
            Self::Http { status, msg } => write!(f, "{status}: {msg}"),
            Self::Response { msg, .. } => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GemApiError {}

impl From<ClientError> for GemApiError {
    fn from(error: ClientError) -> Self {
        match error {
            ClientError::Network(msg) => Self::Network { msg },
            ClientError::Timeout => Self::Timeout,
            ClientError::Http { status, body } => match serde_json::from_slice::<ResponseError>(&body) {
                Ok(response) => Self::Response {
                    status,
                    msg: response.error.message,
                },
                Err(_) => Self::Http {
                    status,
                    msg: String::from_utf8_lossy(&body).to_string(),
                },
            },
            ClientError::Serialization(msg) => Self::Serialization { msg },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::GemApiClient;
    use crate::services::error::GemServiceError;
    use crate::testkit::TestAlienProvider;
    use futures::executor::block_on;
    use primitives::{AssetId, Chain};
    use std::sync::Arc;

    const REWARDS_DISABLED: &str = r#"{"error":{"message":"Rewards are not enabled for this user"}}"#;

    #[test]
    fn test_an_api_error_body_reads_as_its_message() {
        let error = GemApiError::from(ClientError::Http {
            status: 400,
            body: REWARDS_DISABLED.as_bytes().to_vec(),
        });

        assert!(matches!(error, GemApiError::Response { status: 400, .. }));
        assert_eq!(error.to_string(), "Rewards are not enabled for this user");
        assert!(matches!(GemServiceError::from(error), GemServiceError::Api { msg } if msg == "Rewards are not enabled for this user"));
    }

    #[test]
    fn test_an_opaque_error_body_keeps_its_status_and_text() {
        let error = GemApiError::from(ClientError::Http {
            status: 502,
            body: b"Bad Gateway".to_vec(),
        });

        assert!(matches!(error, GemApiError::Http { status: 502, .. }));
        assert_eq!(error.to_string(), "502: Bad Gateway");
    }

    #[test]
    fn test_a_request_answered_with_an_error_body_surfaces_the_message() {
        let client = GemApiClient::new(Arc::new(TestAlienProvider::with_json(404, r#"{"error":{"message":"Asset not found"}}"#)));

        let error = GemApiError::from(block_on(client.client.get_asset(AssetId::from_chain(Chain::Bitcoin))).unwrap_err());

        assert_eq!(error.to_string(), "Asset not found");
    }
}

use std::error::Error;

use gem_client::ClientError;
use strum::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum FailureReason {
    #[strum(to_string = "status={0}")]
    Status(u16),
    Timeout,
    ConnectError,
    RequestError,
}

impl FailureReason {
    pub(crate) fn from_error(error: &(dyn Error + Send + Sync + 'static)) -> Self {
        if let Some(error) = error.downcast_ref::<ClientError>() {
            return match error {
                ClientError::Timeout => Self::Timeout,
                ClientError::Http { status, .. } => Self::Status(*status),
                ClientError::Network(_) | ClientError::Serialization(_) => Self::RequestError,
            };
        }

        match error.downcast_ref::<reqwest::Error>() {
            Some(error) if error.is_timeout() => Self::Timeout,
            Some(error) if error.is_connect() => Self::ConnectError,
            Some(_) | None => Self::RequestError,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_errors_map_to_reasons() {
        let cases = [
            (ClientError::Timeout, FailureReason::Timeout),
            (ClientError::Http { status: 503, body: Vec::new() }, FailureReason::Status(503)),
            (ClientError::Network("request failed".to_string()), FailureReason::RequestError),
            (
                ClientError::Serialization("missing field `result` at line 1 column 2".to_string()),
                FailureReason::RequestError,
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(FailureReason::from_error(&error), expected);
        }
    }
}

use std::fmt::{Display, Formatter};

pub type RpcAlienError = gem_jsonrpc::alien::AlienError;

#[derive(Debug, Clone, uniffi::Error)]
pub enum AlienError {
    RequestError { msg: String },
    ResponseError { msg: String },
    Http { status: u16, len: u32 },
    Offline,
}

impl Display for AlienError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        RpcAlienError::from(self.clone()).fmt(f)
    }
}

impl std::error::Error for AlienError {}

impl From<uniffi::UnexpectedUniFFICallbackError> for AlienError {
    fn from(error: uniffi::UnexpectedUniFFICallbackError) -> Self {
        Self::RequestError { msg: error.reason }
    }
}

impl From<AlienError> for RpcAlienError {
    fn from(error: AlienError) -> Self {
        match error {
            AlienError::RequestError { msg } => Self::RequestError { msg },
            AlienError::ResponseError { msg } => Self::ResponseError { msg },
            AlienError::Http { status, len } => Self::Http { status, len },
            AlienError::Offline => Self::Offline,
        }
    }
}

impl From<RpcAlienError> for AlienError {
    fn from(error: RpcAlienError) -> Self {
        match error {
            RpcAlienError::RequestError { msg } => Self::RequestError { msg },
            RpcAlienError::ResponseError { msg } => Self::ResponseError { msg },
            RpcAlienError::Http { status, len } => Self::Http { status, len },
            RpcAlienError::Offline => Self::Offline,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use uniffi::{LiftReturn, UnexpectedUniFFICallbackError};

    use super::AlienError;
    use crate::alien::AlienResponse;

    #[test]
    fn test_unexpected_platform_error_is_returned_as_request_error() {
        let unexpected = UnexpectedUniFFICallbackError {
            reason: "java.lang.IllegalArgumentException: unexpected url".to_string(),
        };
        let result = <Result<Arc<AlienResponse>, AlienError> as LiftReturn<crate::UniFfiTag>>::handle_callback_unexpected_error(unexpected);

        match result {
            Err(AlienError::RequestError { msg }) => assert_eq!(msg, "java.lang.IllegalArgumentException: unexpected url"),
            other => panic!("Expected RequestError, got {other:?}"),
        }
    }
}

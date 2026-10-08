pub type AlienError = swapper::AlienError;

#[uniffi::remote(Enum)]
pub enum AlienError {
    RequestError { msg: String },
    ResponseError { msg: String },
    Http { status: u16, len: u32 },
    Offline,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use uniffi::{LiftReturn, UnexpectedUniFFICallbackError};

    use super::AlienError;
    use crate::alien::AlienResponse;

    #[test]
    fn test_unexpected_callback_error_returns_request_error() {
        let result = <Result<Arc<AlienResponse>, AlienError> as LiftReturn<crate::UniFfiTag>>::handle_callback_unexpected_error(UnexpectedUniFFICallbackError {
            reason: "unexpected platform exception".into(),
        });

        match result.unwrap_err() {
            AlienError::RequestError { msg } => assert_eq!(msg, "unexpected platform exception"),
            error => panic!("Expected RequestError, got {error:?}"),
        }
    }
}

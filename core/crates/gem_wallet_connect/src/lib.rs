#[cfg(feature = "request")]
pub mod actions;
#[cfg(feature = "request")]
pub mod decode;
#[cfg(feature = "request")]
pub mod request_handler;

#[cfg(feature = "session")]
pub mod accounts;
#[cfg(feature = "session")]
pub mod response_handler;
#[cfg(feature = "session")]
pub mod session;
#[cfg(feature = "session")]
pub mod validator;
#[cfg(feature = "session")]
pub mod verifier;

#[cfg(test)]
mod testkit;

#[cfg(feature = "request")]
pub use actions::*;
#[cfg(feature = "request")]
pub use decode::decode_sign_message;
#[cfg(feature = "request")]
pub use request_handler::WalletConnectRequestHandler;

#[cfg(feature = "session")]
pub use response_handler::WalletConnectResponseHandler;
#[cfg(feature = "session")]
pub use session::config_session_properties;
#[cfg(feature = "session")]
pub use validator::{SignMessageValidation, validate_send_transaction, validate_sign_message};
#[cfg(feature = "session")]
pub use verifier::WalletConnectVerifier;

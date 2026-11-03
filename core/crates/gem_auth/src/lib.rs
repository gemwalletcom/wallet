mod device_signature;
mod signature;

pub const AUTHORIZATION_HEADER: &str = "Authorization";
pub const BEARER_PREFIX: &str = "Bearer ";

pub use device_signature::{DeviceAuthPayload, build_device_auth_header, device_auth_message, device_body_hash, device_public_key, parse_device_auth, verify_device_auth, verify_device_signature};
pub use signature::{AuthMessageData, create_auth_hash, verify_auth_signature};

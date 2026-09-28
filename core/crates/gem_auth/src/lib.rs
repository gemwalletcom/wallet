mod device_signature;
#[cfg(feature = "jwt")]
mod jwt;
mod signature;

pub const AUTHORIZATION_HEADER: &str = "Authorization";
pub const BEARER_PREFIX: &str = "Bearer ";

pub use device_signature::{DeviceAuthPayload, build_device_auth_header, device_auth_message, device_body_hash, device_public_key, parse_device_auth, verify_device_auth, verify_device_signature};
#[cfg(feature = "jwt")]
pub use jsonwebtoken::errors::Error as JwtError;
#[cfg(feature = "jwt")]
pub use jwt::{JwtClaims, create_device_token, verify_device_token};
pub use signature::{AuthMessageData, create_auth_hash, verify_auth_signature};

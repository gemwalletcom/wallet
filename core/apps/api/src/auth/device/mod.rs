mod error;
pub mod guard;
mod middleware;

use std::time::Duration;

pub use error::DeviceError;
pub use guard::{AuthenticatedDevice, AuthenticatedDeviceWallet, DeviceBody, DeviceJson, VerifiedDeviceId};
pub use middleware::{DeviceAuth, SignedPath, VerifiedRequest, device_auth};

pub const DEVICE_ID_LENGTH: usize = 64;

pub struct JwtConfig {
    pub secret: String,
    pub expiry: Duration,
}

pub struct DeviceAuthConfig {
    pub tolerance: Duration,
    pub jwt: JwtConfig,
}

impl DeviceAuthConfig {
    pub fn new(tolerance: Duration, jwt: JwtConfig) -> Self {
        Self { tolerance, jwt }
    }
}

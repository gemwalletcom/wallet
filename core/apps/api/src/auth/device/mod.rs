mod error;
pub mod guard;
mod middleware;

use std::time::Duration;

pub use error::DeviceError;
pub use guard::{AuthenticatedDevice, AuthenticatedDeviceWallet, DeviceBody, DeviceJson, VerifiedDeviceId};
pub use middleware::{DeviceAuth, SignedPath, VerifiedRequest, device_auth};

pub const DEVICE_ID_LENGTH: usize = 64;

pub struct DeviceAuthConfig {
    pub tolerance: Duration,
}

impl DeviceAuthConfig {
    pub fn new(tolerance: Duration) -> Self {
        Self { tolerance }
    }
}

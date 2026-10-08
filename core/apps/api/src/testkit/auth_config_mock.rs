use primitives::MINUTE;

use crate::auth::device::{DeviceAuthConfig, JwtConfig};

impl JwtConfig {
    pub fn mock() -> Self {
        Self {
            secret: "secret".to_string(),
            expiry: MINUTE,
        }
    }
}

impl DeviceAuthConfig {
    pub fn mock() -> Self {
        Self::new(MINUTE, JwtConfig::mock())
    }
}

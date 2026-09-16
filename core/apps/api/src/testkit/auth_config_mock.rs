use crate::devices::auth_config::{AuthConfig, JwtConfig};
use primitives::MINUTE;

impl JwtConfig {
    pub fn mock() -> Self {
        Self {
            secret: "secret".to_string(),
            expiry: MINUTE,
        }
    }
}

impl AuthConfig {
    pub fn mock() -> Self {
        Self::new(MINUTE, JwtConfig::mock())
    }
}

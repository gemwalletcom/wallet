use std::time::Duration;

use crate::devices::auth_config::{AuthConfig, JwtConfig};

impl JwtConfig {
    pub fn mock() -> Self {
        Self {
            secret: "secret".to_string(),
            expiry: Duration::from_secs(60),
        }
    }
}

impl AuthConfig {
    pub fn mock() -> Self {
        Self::new(Duration::from_secs(60), JwtConfig::mock())
    }
}

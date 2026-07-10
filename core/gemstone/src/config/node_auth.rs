#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct GemNodeAuthConfig {
    pub check_interval_seconds: u32,
    pub refresh_threshold_seconds: u32,
}

#[uniffi::export]
pub fn node_auth_config() -> GemNodeAuthConfig {
    GemNodeAuthConfig {
        check_interval_seconds: 60,
        refresh_threshold_seconds: 300,
    }
}

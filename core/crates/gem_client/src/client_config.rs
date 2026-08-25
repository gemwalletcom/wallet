use std::time::Duration;

use crate::DEFAULT_REQUEST_TIMEOUT;

const GEM_WALLET_USER_AGENT: &str = concat!("Gem/Rust/", env!("CARGO_PKG_VERSION"));
const FAILED_TO_BUILD_REQWEST_CLIENT: &str = "Failed to build reqwest client";

pub fn builder() -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        .user_agent(GEM_WALLET_USER_AGENT)
        .timeout(DEFAULT_REQUEST_TIMEOUT)
        .connect_timeout(Duration::from_secs(15))
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(20)
        .tcp_keepalive(Duration::from_secs(60))
        .gzip(true)
        .brotli(true)
        .deflate(true)
}

pub fn reqwest_client() -> reqwest::Client {
    builder().build().expect(FAILED_TO_BUILD_REQWEST_CLIENT)
}

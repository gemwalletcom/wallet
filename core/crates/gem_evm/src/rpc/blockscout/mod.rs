mod client;
mod model;

pub(crate) use client::BlockscoutClient;

#[cfg(feature = "reqwest")]
pub(super) const BLOCKSCOUT_URL: &str = "https://api.blockscout.com";

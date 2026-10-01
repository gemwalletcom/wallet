mod asset;
mod config_store;
mod hubpool;
mod provider;
mod status;
#[cfg(test)]
mod testkit;

pub use provider::Across;

const DEFAULT_FILL_TIMEOUT: u32 = 60 * 60 * 6;
const DEFAULT_DEPOSIT_GAS_LIMIT: u64 = 180_000;
const DEFAULT_FILL_GAS_LIMIT: u64 = 120_000;

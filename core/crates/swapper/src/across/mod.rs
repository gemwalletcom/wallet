mod asset;
mod config_store;
mod deposit;
mod hubpool;
mod provider;
mod status;

pub use provider::Across;

const DEFAULT_FILL_TIMEOUT: u32 = 60 * 60 * 6; // 6 hours
const DEFAULT_DEPOSIT_GAS_LIMIT: u64 = 180_000; // gwei
const DEFAULT_FILL_GAS_LIMIT: u64 = 120_000; // gwei

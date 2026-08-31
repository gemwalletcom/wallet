pub const TREASURY_ADDRESS: &str = "13UVJyLnbVp9RBZYFwFGyDvVd1y27Tt8tkntv6Q7JVPhFsTB";

pub const TRANSACTION_TYPE_TRANSFER_KEEP_ALIVE: &str = "transferKeepAlive";
pub const TRANSACTION_TYPE_TRANSFER_ALLOW_DEATH: &str = "transferAllowDeath";
#[cfg(feature = "rpc")]
pub(crate) const TRANSACTION_FEE_ESTIMATE: u64 = 150_000_000;

pub const TRANSACTION_TYPE_TRANSFER_KEEP_ALIVE: &str = "transferKeepAlive";
pub const TRANSACTION_TYPE_TRANSFER_ALLOW_DEATH: &str = "transferAllowDeath";
#[cfg(feature = "rpc")]
pub(crate) const TRANSACTION_FEE_ESTIMATE: u64 = 150_000_000;

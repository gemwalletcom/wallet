pub const TRANSACTION_STATUS_FINAL: &str = "FINAL";
pub const TRANSACTION_STATUS_EXECUTED: &str = "EXECUTED";
pub const TRANSACTION_STATUS_EXECUTED_OPTIMISTIC: &str = "EXECUTED_OPTIMISTIC";

#[cfg(feature = "rpc")]
pub(crate) const STORAGE_AMOUNT_PER_BYTE: u128 = 10_000_000_000_000_000_000;
#[cfg(feature = "signer")]
pub(crate) const FUNGIBLE_TOKEN_TRANSFER_DEPOSIT: u128 = 1;
#[cfg(any(feature = "rpc", feature = "signer"))]
pub(crate) const FUNGIBLE_TOKEN_FUNCTION_CALL_GAS: u64 = 30_000_000_000_000;

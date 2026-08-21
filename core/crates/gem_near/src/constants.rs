pub const TRANSACTION_STATUS_FINAL: &str = "FINAL";
pub const TRANSACTION_STATUS_EXECUTED: &str = "EXECUTED";
pub const TRANSACTION_STATUS_EXECUTED_OPTIMISTIC: &str = "EXECUTED_OPTIMISTIC";
pub const TRANSACTION_STATUSES_EXECUTED: [&str; 3] = [TRANSACTION_STATUS_FINAL, TRANSACTION_STATUS_EXECUTED, TRANSACTION_STATUS_EXECUTED_OPTIMISTIC];

#[cfg(feature = "rpc")]
pub(crate) const EVENT_JSON_PREFIX: &str = "EVENT_JSON:";
#[cfg(feature = "rpc")]
pub(crate) const EMPTY_TRANSACTION_ROOT: &str = "11111111111111111111111111111111";
#[cfg(feature = "rpc")]
pub(crate) const FUNGIBLE_TOKEN_TRANSFER_EVENT: &str = "ft_transfer";
#[cfg(feature = "rpc")]
pub(crate) const NATIVE_ASSET_ID: &str = "near";
#[cfg(feature = "rpc")]
pub(crate) const NEP_141_STANDARD: &str = "nep141";
#[cfg(feature = "rpc")]
pub(crate) const RPC_CONCURRENCY: usize = 5;
#[cfg(feature = "rpc")]
pub(crate) const STORAGE_AMOUNT_PER_BYTE: u128 = 10_000_000_000_000_000_000;
#[cfg(feature = "signer")]
pub(crate) const FUNGIBLE_TOKEN_TRANSFER_DEPOSIT: u128 = 1;
#[cfg(any(feature = "rpc", feature = "signer"))]
pub(crate) const FUNGIBLE_TOKEN_FUNCTION_CALL_GAS: u64 = 30_000_000_000_000;
#[cfg(feature = "rpc")]
pub(crate) const TRANSFER_GAS_UNITS: u64 = 3_000_000_000_000;

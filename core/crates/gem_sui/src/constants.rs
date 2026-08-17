pub const SUI_SYSTEM_ID: &str = "sui_system";

pub const SUI_FRAMEWORK_PACKAGE_ID: u8 = 0x2;
pub const SUI_SYSTEM_PACKAGE_ID: u8 = 0x3;
pub const SUI_SYSTEM_STATE_OBJECT_ID: u8 = 0x5;
pub const SUI_CLOCK_OBJECT_ID: u8 = 0x6;

pub const SUI_COIN_TYPE: &str = "0x2::sui::SUI";
pub const SUI_COIN_TYPE_FULL: &str = "0x0000000000000000000000000000000000000000000000000000000000000002::sui::SUI";
pub const EMPTY_ADDRESS: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";
pub const ESTIMATION_GAS_BUDGET: u64 = 50_000_000;
pub const SUI_STAKE_EVENT: &str = "0x3::validator::StakingRequestEvent";
pub const SUI_UNSTAKE_EVENT: &str = "0x3::validator::UnstakingRequestEvent";

#[cfg(feature = "rpc")]
pub(crate) const TRANSFER_GAS_UNITS: u64 = 3_000;
#[cfg(feature = "rpc")]
pub(crate) const TOKEN_TRANSFER_GAS_UNITS: u64 = 5_000;
#[cfg(feature = "rpc")]
pub(crate) const SWAP_GAS_UNITS: u64 = 10_000;

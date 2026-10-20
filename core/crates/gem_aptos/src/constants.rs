pub const APTOS_NATIVE_COIN: &str = "0x1::aptos_coin::AptosCoin";
pub const APTOS_NATIVE_METADATA_ADDRESS: &str = "0xa";
pub const APTOS_TRANSFER_FUNCTION: &str = "0x1::aptos_account::transfer";
pub const FUNGIBLE_TRANSFER_FUNCTION: &str = "0x1::primary_fungible_store::transfer";
pub const OBJECT_CORE_TYPE: &str = "0x1::object::ObjectCore";
pub const ENTRY_FUNCTION_PAYLOAD_TYPE: &str = "entry_function_payload";
pub const NO_ACCOUNT_SIGNATURE_TYPE: &str = "no_account_signature";
pub const FEE_PAYER_SIGNATURE_TYPE: &str = "fee_payer_signature";
pub const SIMULATION_FEE_PAYER_ADDRESS: &str = "0x0";
pub const SIMULATION_MAX_GAS_AMOUNT: u64 = 2_000_000;
pub const DEFAULT_MAX_GAS_AMOUNT: u64 = 1500;
pub const DEFAULT_SWAP_MAX_GAS_AMOUNT: u64 = 30000;
pub const GAS_LIMIT_MULTIPLIER: u64 = 120;

pub const STAKE_WITHDRAW_EVENT: &str = "0x1::coin::WithdrawEvent";
pub const STAKE_DEPOSIT_EVENT: &str = "0x1::coin::DepositEvent";

pub const FUNGIBLE_ASSET_WITHDRAW_EVENT: &str = "0x1::fungible_asset::Withdraw";
pub const FUNGIBLE_ASSET_DEPOSIT_EVENT: &str = "0x1::fungible_asset::Deposit";

pub const DELEGATION_POOL_ADD_STAKE_FUNCTION: &str = "0x1::delegation_pool::add_stake";
pub const DELEGATION_POOL_UNLOCK_FUNCTION: &str = "0x1::delegation_pool::unlock";
pub const DELEGATION_POOL_WITHDRAW_FUNCTION: &str = "0x1::delegation_pool::withdraw";
pub const DELEGATION_POOL_GET_STAKE_FUNCTION: &str = "0x1::delegation_pool::get_stake";
pub const DELEGATION_POOL_OPERATOR_COMMISSION_FUNCTION: &str = "0x1::delegation_pool::operator_commission_percentage";
pub const STAKE_GET_LOCKUP_SECS_FUNCTION: &str = "0x1::stake::get_lockup_secs";
pub const STAKING_CONFIG_REWARD_RATE_FUNCTION: &str = "0x1::staking_config::reward_rate";

pub const HYPERION_CONTRACT: &str = "0x8b4a2c4bb53857c718a04c020b98f8c2e1f99a68b0f57389a8bf5434cd22e05c";
pub const VM_STATUS_ABORTED: u64 = 4016;
pub const HYPERION_PARTNER_SWAP_FUNCTION: &str = "0x8b4a2c4bb53857c718a04c020b98f8c2e1f99a68b0f57389a8bf5434cd22e05c::partnership::partner_swap";
pub const HYPERION_THIRD_PARTY_SWAP_EVENT: &str = "0x8b4a2c4bb53857c718a04c020b98f8c2e1f99a68b0f57389a8bf5434cd22e05c::partnership::ThirdPartySwapEvent";
pub const HYPERION_POOL_SWAP_EVENTS: [&str; 3] = [
    "0x8b4a2c4bb53857c718a04c020b98f8c2e1f99a68b0f57389a8bf5434cd22e05c::pool_v3::SwapEvent",
    "0x8b4a2c4bb53857c718a04c020b98f8c2e1f99a68b0f57389a8bf5434cd22e05c::pool_v3::SwapEventV2",
    "0x8b4a2c4bb53857c718a04c020b98f8c2e1f99a68b0f57389a8bf5434cd22e05c::pool_v3::SwapEventV3",
];

pub const DELEGATION_POOL_ADD_STAKE_EVENT: &str = "0x1::delegation_pool::AddStake";
pub const DELEGATION_POOL_UNLOCK_STAKE_EVENT: &str = "0x1::delegation_pool::UnlockStake";

pub const KNOWN_VALIDATOR_POOL: &str = "0xdb5247f859ce63dbe8940cf8773be722a60dcc594a8be9aca4b76abceb251b8e";

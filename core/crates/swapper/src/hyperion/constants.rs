use gem_aptos::AccountAddress;
use hex_lit::hex;

pub(super) const MODULE_POOL: &str = "pool_v3";
pub(super) const POOL_CREATOR: AccountAddress = AccountAddress::new(hex!("4ea3c7d6fd8ee6e752ca70420d4aac1fda379db4475520249faf8e04ad31c5a4"));

pub(super) const FUNCTION_GET_AMOUNT_OUT: &str = "get_amount_out";
pub(super) const FUNCTION_POOL_RESERVE_AMOUNT: &str = "pool_reserve_amount";

pub(super) const HYPERION_FEE_TIERS: [u8; 6] = [0, 1, 2, 3, 4, 5];

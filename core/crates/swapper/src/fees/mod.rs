mod referral;
mod reserve;
mod slippage;

pub(crate) use referral::referral_fees;
pub use referral::{FeeSide, ReferralFee, ReferralFees, ReferralTransfer, SOLANA_FEE_MINTS, default_referral_address, default_referral_fees, on_chain_referral_fees};
pub use reserve::{max_amount_spends_all_but_fee, reserved_transaction_fees};
pub(crate) use slippage::minimum_value;
pub use slippage::{bps_of, bps_to_percent_string, percent_to_bps, subtract_bps};

pub const DEFAULT_SWAP_FEE_BPS: u32 = 50;
const STABLECOIN_SWAP_FEE_BPS: u32 = 25;
pub const BRIDGE_FEE_BPS: u32 = 25;
pub const DEFAULT_AGGREGATOR_FEE_BPS: u32 = 70;
pub const DEFAULT_CHAINFLIP_FEE_BPS: u32 = 45;
pub const DEFAULT_REFERRER: &str = "gemwallet";

pub(crate) fn is_stablecoin_symbol(symbol: &str) -> bool {
    symbol.to_ascii_uppercase().contains("USD")
}

use crate::{
    SwapperError,
    mayan::{cctp_domain::CCTP_TOKEN_DECIMALS, model::MayanMctpQuote, tx_builder::amount::fractional_amount},
};

pub(in crate::mayan) mod evm;
pub(in crate::mayan) mod solana;
pub(in crate::mayan) mod sui;

fn redeem_relayer_fee(route: &MayanMctpQuote) -> Result<u64, SwapperError> {
    fractional_amount(route.redeem_relayer_fee.as_ref().ok_or(SwapperError::InvalidRoute)?, CCTP_TOKEN_DECIMALS)
}

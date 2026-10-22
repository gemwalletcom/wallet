use crate::{
    SwapperError,
    mayan::{model::MayanMctpQuote, tx_builder::amount::value_to_query},
};

pub(super) fn bridge_fee(route: &MayanMctpQuote) -> Result<u64, SwapperError> {
    value_to_query(route.bridge_fee.as_ref().ok_or(SwapperError::InvalidRoute)?)?.parse::<u64>().map_err(SwapperError::from)
}

use alloy_primitives::{Address, U256};

use super::routed_asset::{Funding, RoutedAsset};
use crate::eth_address;

impl RoutedAsset {
    pub fn mock(address: Address, funding: Funding, scale: u64) -> Self {
        Self { address, scale: U256::from(scale), funding }
    }

    pub fn mock_permit2(token_id: &str) -> Self {
        Self::mock(eth_address::parse_str(token_id).unwrap(), Funding::Permit2, 1)
    }

    pub fn mock_value(token_id: &str) -> Self {
        Self::mock(eth_address::parse_str(token_id).unwrap(), Funding::Value, 1)
    }

    pub fn mock_router_balance(token_id: &str, scale: u64) -> Self {
        Self::mock(eth_address::parse_str(token_id).unwrap(), Funding::RouterBalance, scale)
    }
}

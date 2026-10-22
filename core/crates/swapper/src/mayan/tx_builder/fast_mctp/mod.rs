pub(in crate::mayan) mod evm;
pub(in crate::mayan) mod solana;
pub(in crate::mayan) mod sui;

use crate::{
    SwapperError,
    mayan::{
        cctp_domain::CCTP_TOKEN_DECIMALS,
        model::MayanFastMctpQuote,
        tx_builder::{address::native_address_to_bytes32, amount::fractional_amount, swift::token_address_to_bytes32},
        wormhole_chain::{WormholeChain, id_for_name as wormhole_chain_id},
    },
};
use gem_evm::EVM_ZERO_ADDRESS;
use gem_solana::SYSTEM_PROGRAM_ID;

const FAST_MCTP_PAYLOAD_TYPE_DEFAULT: u8 = 1;
const FAST_MCTP_PAYLOAD_TYPE_ORDER: u8 = 3;

fn referrer_bytes(route: &MayanFastMctpQuote) -> Result<[u8; 32], SwapperError> {
    let Some(address) = route.destination_referrer_address()? else {
        return native_address_to_bytes32(SYSTEM_PROGRAM_ID, wormhole_chain_id(WormholeChain::Solana.name())?);
    };
    native_address_to_bytes32(&address, wormhole_chain_id(&route.to_chain)?)
}

fn redeem_relayer_fee(route: &MayanFastMctpQuote) -> Result<u64, SwapperError> {
    fractional_amount(route.redeem_relayer_fee.as_ref().ok_or(SwapperError::InvalidRoute)?, CCTP_TOKEN_DECIMALS)
}

fn fast_mctp_input_contract(route: &MayanFastMctpQuote) -> Result<&str, SwapperError> {
    route.fast_mctp_input_contract.as_deref().ok_or(SwapperError::InvalidRoute)
}

fn fast_mctp_contract(route: &MayanFastMctpQuote) -> Result<&str, SwapperError> {
    route.fast_mctp_mayan_contract.as_deref().ok_or(SwapperError::InvalidRoute)
}

fn fast_mctp_min_finality(route: &MayanFastMctpQuote) -> Result<u32, SwapperError> {
    route.fast_mctp_min_finality.ok_or(SwapperError::InvalidRoute)
}

fn circle_max_fee64(route: &MayanFastMctpQuote) -> Result<&str, SwapperError> {
    route.circle_max_fee64.as_deref().ok_or(SwapperError::InvalidRoute)
}

fn token_out(route: &MayanFastMctpQuote) -> Result<[u8; 32], SwapperError> {
    if route.to_token.contract == EVM_ZERO_ADDRESS {
        return native_address_to_bytes32(SYSTEM_PROGRAM_ID, wormhole_chain_id(WormholeChain::Solana.name())?);
    }
    if route.to_chain == WormholeChain::Sui.name() {
        return token_address_to_bytes32(route.to_token.currency_id.as_deref().ok_or(SwapperError::InvalidRoute)?, WormholeChain::Sui.name());
    }
    native_address_to_bytes32(&route.to_token.contract, route.to_token.w_chain_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{fees::default_referral_address, mayan::tx_builder::address::native_address_to_bytes32};
    use primitives::{Chain, hex};

    #[test]
    fn test_token_out() {
        let route = MayanFastMctpQuote::mock_evm_to_sui();
        assert_eq!(hex::encode_with_0x(&token_out(&route).unwrap()), "0x75cfbbf8c962d542e99a1d15731e6069f60a00db895407785b15d14f606f2b4a");

        let route = MayanFastMctpQuote::mock_evm();
        assert_eq!(token_out(&route).unwrap(), native_address_to_bytes32(&route.to_token.contract, route.to_token.w_chain_id).unwrap());
    }

    #[test]
    fn test_referrer_bytes() {
        let route = MayanFastMctpQuote::mock_evm_to_sui();
        assert_eq!(
            referrer_bytes(&route).unwrap(),
            native_address_to_bytes32(&default_referral_address(Chain::Sui), wormhole_chain_id(WormholeChain::Sui.name()).unwrap()).unwrap()
        );
    }
}

use std::str::FromStr;

use alloy_primitives::U256;
use primitives::Chain;

use super::{broker::RefundParameters, chain::ChainflipChain};
use crate::SwapperError;

const CF_PARAMETERS_VERSION: u8 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultSwapRefund {
    chain: Chain,
    refund: RefundParameters,
}

impl VaultSwapRefund {
    pub(super) fn new(chain: Chain, refund: RefundParameters) -> Self {
        Self { chain, refund }
    }

    pub(super) fn validate(&self, cf_parameters: &[u8]) -> Result<(), SwapperError> {
        if !cf_parameters.starts_with(&self.encode()?) {
            return Err(SwapperError::TransactionError("Chainflip refund mismatch".to_string()));
        }
        Ok(())
    }

    fn encode(&self) -> Result<Vec<u8>, SwapperError> {
        let chain = ChainflipChain::from_chain(self.chain).ok_or(SwapperError::NotSupportedChain)?;
        let address = chain
            .encode_address(&self.refund.refund_address)
            .ok_or_else(|| SwapperError::TransactionError(format!("invalid Chainflip refund address: {}", self.refund.refund_address)))?;
        let min_price = U256::from_str(&self.refund.min_price).map_err(SwapperError::transaction_error)?;
        Ok([&[CF_PARAMETERS_VERSION], &self.refund.retry_duration.to_le_bytes()[..], &address, &min_price.to_le_bytes::<32>()].concat())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chainflip::testkit::mock_refund;
    use primitives::testkit::signer_mock::{TEST_BITCOIN_SENDER, TEST_EVM_SENDER};

    #[test]
    fn test_validate() {
        let refund = mock_refund(Chain::Ethereum, TEST_EVM_SENDER, "0x0102");
        let parameters = [&[1u8, 150, 0, 0, 0][..], &hex::decode(&TEST_EVM_SENDER[2..]).unwrap(), &[2, 1], &[0; 30], &[9]].concat();

        assert_eq!(refund.validate(&parameters), Ok(()));
        assert!(mock_refund(Chain::Ethereum, TEST_EVM_SENDER, "0x0103").validate(&parameters).is_err(), "a different minimum price is refused");
        assert!(mock_refund(Chain::Ethereum, "0x1111111111111111111111111111111111111111", "0x0102").validate(&parameters).is_err());
        assert!(mock_refund(Chain::Ethereum, TEST_BITCOIN_SENDER, "0x0102").validate(&parameters).is_err(), "an address the chain can't read is refused");
        assert!(refund.validate(&parameters[1..]).is_err());
        assert!(refund.validate(&[&[2u8][..], &parameters[1..]].concat()).is_err(), "a layout this app can't read is refused");
    }
}

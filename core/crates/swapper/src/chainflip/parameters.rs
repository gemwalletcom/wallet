use std::str::FromStr;

use alloy_primitives::U256;
use primitives::Chain;

use super::{
    broker::{DcaParameters, RefundParameters},
    chain::ChainflipChain,
    destination::NO_CCM,
};
use crate::{SwapperError, fees::DEFAULT_CHAINFLIP_FEE_BPS};

const CF_PARAMETERS_VERSION: u8 = 1;
const NO_ORACLE_PRICE_SLIPPAGE: u8 = 0;
const BROKER_ACCOUNT_LENGTH: usize = 32;
const BROKER_FEE_BPS: u32 = 5;

#[derive(Debug, Clone, PartialEq)]
pub struct VaultSwapParameters {
    chain: Chain,
    refund: RefundParameters,
    dca: Option<DcaParameters>,
    boost_fee: u8,
}

impl VaultSwapParameters {
    pub(super) fn new(chain: Chain, refund: RefundParameters, dca: Option<DcaParameters>, boost_fee: Option<u32>) -> Result<Self, SwapperError> {
        Ok(Self {
            chain,
            refund,
            dca,
            boost_fee: u8::try_from(boost_fee.unwrap_or_default()).map_err(|_| SwapperError::InvalidRoute)?,
        })
    }

    pub(super) fn validate(&self, cf_parameters: &[u8]) -> Result<(), SwapperError> {
        let commission = cf_parameters
            .strip_prefix(self.encode()?.as_slice())
            .and_then(|fees| fees.split_at_checked(BROKER_ACCOUNT_LENGTH))
            .and_then(|(_, fees)| commission_bps(fees))
            .ok_or(SwapperError::InvalidRoute)?;
        if commission > DEFAULT_CHAINFLIP_FEE_BPS + BROKER_FEE_BPS {
            return Err(SwapperError::InvalidRoute);
        }
        Ok(())
    }

    fn encode(&self) -> Result<Vec<u8>, SwapperError> {
        let chain = ChainflipChain::from_chain(self.chain).ok_or(SwapperError::NotSupportedChain)?;
        let address = chain
            .encode_address(&self.refund.refund_address)
            .ok_or_else(|| SwapperError::TransactionError(format!("invalid Chainflip refund address: {}", self.refund.refund_address)))?;
        let min_price = U256::from_str(&self.refund.min_price).map_err(SwapperError::transaction_error)?;
        let dca = match &self.dca {
            Some(dca) => [&[1u8][..], &dca.number_of_chunks.to_le_bytes(), &dca.chunk_interval.to_le_bytes()].concat(),
            None => vec![0],
        };
        Ok([
            &[CF_PARAMETERS_VERSION],
            &self.refund.retry_duration.to_le_bytes()[..],
            &address,
            &min_price.to_le_bytes::<32>(),
            &[NO_CCM, NO_ORACLE_PRICE_SLIPPAGE],
            &dca,
            &[self.boost_fee],
        ]
        .concat())
    }
}

fn commission_bps(fees: &[u8]) -> Option<u32> {
    let (&[low, high, affiliates], affiliate_fees) = fees.split_first_chunk::<3>()?;
    let (affiliate_fees, rest) = affiliate_fees.as_chunks::<2>();
    if affiliates & 0b11 != 0 || !rest.is_empty() || affiliate_fees.len() != usize::from(affiliates >> 2) {
        return None;
    }
    Some(affiliate_fees.iter().fold(u32::from(u16::from_le_bytes([low, high])), |total, [_, fee]| total + u32::from(*fee)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chainflip::testkit::mock_parameters;
    use primitives::hex;
    use primitives::testkit::signer_mock::{TEST_BITCOIN_SENDER, TEST_EVM_SENDER};

    #[test]
    fn test_validate() {
        let parameters = mock_parameters(Chain::Ethereum, TEST_EVM_SENDER, "0x0102");
        let refund = [&[1u8, 150, 0, 0, 0][..], &hex::decode(&TEST_EVM_SENDER[2..]).unwrap(), &[2, 1], &[0; 30]].concat();
        let encoded = |swap: &[u8], fees: &[u8]| [&refund[..], swap, &[7; 32], fees].concat();
        let shipped = encoded(&[0, 0, 0, 0], &[5, 0, 4, 0, 45]);

        assert_eq!(parameters.validate(&shipped), Ok(()));
        assert_eq!(parameters.validate(&encoded(&[0, 0, 0, 0], &[45, 0, 0])), Ok(()), "the whole commission taken by the broker");
        let split = VaultSwapParameters {
            dca: Some(DcaParameters { number_of_chunks: 3, chunk_interval: 2 }),
            boost_fee: 7,
            ..parameters.clone()
        };
        assert_eq!(split.validate(&encoded(&[0, 0, 1, 3, 0, 0, 0, 2, 0, 0, 0, 7], &[5, 0, 4, 0, 45])), Ok(()));

        for (cf_parameters, reason) in [
            (encoded(&[1, 0, 0, 0], &[5, 0, 4, 0, 45]), "a refund that also makes a call"),
            (encoded(&[0, 1, 77, 0, 0, 0], &[5, 0, 4, 0, 45]), "an oracle price limit the quote did not set"),
            (encoded(&[0, 0, 1, 3, 0, 0, 0, 2, 0, 0, 0, 0], &[5, 0, 4, 0, 45]), "a swap split into chunks the quote did not ask for"),
            (encoded(&[0, 0, 0, 7], &[5, 0, 4, 0, 45]), "a boost fee the quote did not ask for"),
            (encoded(&[0, 0, 0, 0], &[6, 0, 4, 0, 45]), "a broker fee above the one quoted"),
            (encoded(&[0, 0, 0, 0], &[5, 0, 8, 0, 45, 1, 1]), "a second affiliate fee"),
            (encoded(&[0, 0, 0, 0], &[5, 0, 4, 0]), "an affiliate list cut short"),
            (shipped[1..].to_vec(), "a layout this app cannot read"),
            ([&[2u8][..], &shipped[1..]].concat(), "another version of the layout"),
            ([&shipped[..], &[0]].concat(), "bytes after the fees"),
        ] {
            assert_eq!(parameters.validate(&cf_parameters), Err(SwapperError::InvalidRoute), "{reason}");
        }
        assert!(mock_parameters(Chain::Ethereum, TEST_EVM_SENDER, "0x0103").validate(&shipped).is_err(), "a different minimum price is refused");
        assert!(mock_parameters(Chain::Ethereum, "0x1111111111111111111111111111111111111111", "0x0102").validate(&shipped).is_err());
        assert!(mock_parameters(Chain::Ethereum, TEST_BITCOIN_SENDER, "0x0102").validate(&shipped).is_err(), "an address the chain can't read is refused");
    }
}

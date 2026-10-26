use gem_bitcoin::signer::{SpendDestination, SpendPlan, SpendRequest, UtxoPlanner};
use orchard::Address as OrchardAddress;
#[cfg(feature = "rpc")]
use primitives::TransactionInputType;
use primitives::{BitcoinChain, SignerError, SignerInput};

use crate::{
    address::{Destination, ParseAddress},
    fee::Zip317Fee,
};

const MEMO_BYTES: usize = 512;
const NO_MEMO_TAG: u8 = 0xf6;

#[derive(Debug, Clone)]
pub(crate) struct ShieldedOutput {
    pub(crate) recipient: OrchardAddress,
    pub(crate) value: u64,
    pub(crate) memo: Option<Vec<u8>>,
}

impl ShieldedOutput {
    pub(crate) fn memo_bytes(&self) -> [u8; MEMO_BYTES] {
        let mut bytes = [0u8; MEMO_BYTES];
        match &self.memo {
            Some(memo) => bytes[..memo.len()].copy_from_slice(memo),
            None => bytes[0] = NO_MEMO_TAG,
        }
        bytes
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Plan {
    pub(crate) spend: SpendPlan,
    pub(crate) shielded_output: Option<ShieldedOutput>,
}

impl Plan {
    pub(crate) fn transfer(input: &SignerInput) -> Result<Self, SignerError> {
        let mut recipient = None;
        let request = SpendRequest::transfer(BitcoinChain::Zcash, input, |address| Self::destination(address, &mut recipient))?;
        if recipient.is_none() && request.memo.is_some() {
            return SignerError::invalid_input_err("Zcash memo requires a shielded address");
        }
        Self::new(request, recipient)
    }

    pub(crate) fn swap(input: &SignerInput) -> Result<Self, SignerError> {
        let mut recipient = None;
        let request = SpendRequest::swap(BitcoinChain::Zcash, input, |address| Self::destination(address, &mut recipient))?;
        Self::new(request, recipient)
    }

    #[cfg(feature = "rpc")]
    pub(crate) fn for_input(input: &SignerInput) -> Result<Self, SignerError> {
        match &input.input_type {
            TransactionInputType::Transfer { .. } | TransactionInputType::Withdrawal { .. } => Self::transfer(input),
            TransactionInputType::Swap { .. } => Self::swap(input),
            _ => SignerError::invalid_input_err("unsupported Zcash transaction type"),
        }
    }

    fn destination(address: &str, recipient: &mut Option<OrchardAddress>) -> Result<SpendDestination, SignerError> {
        let (destination, shielded) = Destination::parse(address)?.into_spend();
        *recipient = shielded;
        Ok(destination)
    }

    fn new(request: SpendRequest, recipient: Option<OrchardAddress>) -> Result<Self, SignerError> {
        let memo = request.memo.clone();
        if recipient.is_some() && memo.as_ref().is_some_and(|memo| memo.len() > MEMO_BYTES) {
            return SignerError::invalid_input_err("Zcash memo is too large");
        }
        let spend = UtxoPlanner::plan(request, &Zip317Fee)?;
        let shielded_output = recipient.zip(spend.external_value).map(|(recipient, value)| ShieldedOutput { recipient, value, memo });
        Ok(Self { spend, shielded_output })
    }
}

#[cfg(test)]
mod tests {
    use gem_bitcoin::testkit::address_mock::mock_sender_address;
    use primitives::testkit::zcash_mock;

    use super::*;
    use crate::testkit::{TEST_ZCASH_UNIFIED_ADDRESS, TEST_ZCASH_UNIFIED_SAPLING_TRANSPARENT_ADDRESS, mock_orchard_recipient, mock_transparent_script};

    #[test]
    fn test_transfer() {
        let sender_address = mock_sender_address(BitcoinChain::Zcash);
        let change_script = mock_transparent_script(&sender_address);
        let mut input = zcash_mock::mock_signer_input(sender_address.clone(), TEST_ZCASH_UNIFIED_ADDRESS.to_string());
        input.input.memo = Some("thanks".to_string());

        let plan = Plan::transfer(&input).unwrap();
        let shielded_output = plan.shielded_output.unwrap();
        assert_eq!(plan.spend.fee, 15_000);
        assert_eq!(plan.spend.outputs.iter().map(|output| (output.value.to_sat(), output.script_pubkey.clone())).collect::<Vec<_>>(), vec![(15_000, change_script)]);
        assert_eq!((shielded_output.recipient, shielded_output.value, shielded_output.memo), (mock_orchard_recipient(), 20_000, Some(b"thanks".to_vec())));

        input.input.is_max_value = true;
        let plan = Plan::transfer(&input).unwrap();
        assert_eq!(plan.spend.fee, 15_000);
        assert!(plan.spend.outputs.is_empty());
        assert_eq!(plan.shielded_output.unwrap().value, 35_000);

        input.input.memo = Some("a".repeat(513));
        assert_eq!(Plan::transfer(&input).unwrap_err(), SignerError::invalid_input("Zcash memo is too large"));

        let mut input = zcash_mock::mock_signer_input(sender_address.clone(), TEST_ZCASH_UNIFIED_SAPLING_TRANSPARENT_ADDRESS.to_string());
        input.input.memo = Some("secret".to_string());
        assert_eq!(Plan::transfer(&input).unwrap_err(), SignerError::invalid_input("Zcash memo requires a shielded address"));

        input.input.destination_address = sender_address;
        assert_eq!(Plan::transfer(&input).unwrap_err(), SignerError::invalid_input("Zcash memo requires a shielded address"));

        input.input.memo = None;
        assert!(Plan::transfer(&input).unwrap().shielded_output.is_none());
    }

    #[test]
    fn test_memo_bytes() {
        let output = ShieldedOutput {
            memo: Some(b"hello".to_vec()),
            ..ShieldedOutput::mock()
        };
        let mut expected = [0u8; MEMO_BYTES];
        expected[..5].copy_from_slice(b"hello");
        assert_eq!(output.memo_bytes(), expected);

        let mut empty = [0u8; MEMO_BYTES];
        empty[0] = NO_MEMO_TAG;
        assert_eq!(ShieldedOutput { memo: None, ..output }.memo_bytes(), empty);
    }
}

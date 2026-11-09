use gem_bitcoin::signer::{SpendPlan, SpendRequest, UtxoPlanner};
#[cfg(feature = "rpc")]
use primitives::TransactionInputType;
use primitives::{BitcoinChain, SignerError, SignerInput};

use crate::fee::Zip317Fee;

pub(crate) struct Plan;

impl Plan {
    pub(crate) fn transfer(input: &SignerInput) -> Result<SpendPlan, SignerError> {
        UtxoPlanner::plan(SpendRequest::transfer(BitcoinChain::Zcash, input)?, &Zip317Fee)
    }

    #[cfg(feature = "rpc")]
    pub(crate) fn fee(input: &SignerInput) -> Result<u64, SignerError> {
        let request = match &input.input_type {
            TransactionInputType::Transfer { .. } | TransactionInputType::Withdrawal { .. } => SpendRequest::transfer(BitcoinChain::Zcash, input)?,
            TransactionInputType::Swap { .. } => SpendRequest::swap(BitcoinChain::Zcash, input)?,
            _ => return SignerError::invalid_input_err("unsupported Zcash transaction type"),
        };
        UtxoPlanner::fee(request, &Zip317Fee)
    }
}

#[cfg(test)]
mod tests {
    use gem_bitcoin::testkit::signer_mock::mock_funded_transfer_input;

    use super::*;

    #[test]
    fn test_fee_without_utxos_prices_a_single_input_send() {
        let input = mock_funded_transfer_input(BitcoinChain::Zcash);
        let funded = SpendRequest::transfer(BitcoinChain::Zcash, &input).unwrap();
        for request in [funded.clone(), SpendRequest { is_max: true, ..funded }] {
            let without_utxos = SpendRequest { utxos: vec![], ..request.clone() };

            assert_eq!(UtxoPlanner::plan(without_utxos.clone(), &Zip317Fee).unwrap_err(), SignerError::InsufficientFunds);
            assert_eq!(UtxoPlanner::fee(without_utxos, &Zip317Fee).unwrap(), UtxoPlanner::plan(request, &Zip317Fee).unwrap().fee);
        }
    }
}

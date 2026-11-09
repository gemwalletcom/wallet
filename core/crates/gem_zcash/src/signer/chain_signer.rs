use gem_bitcoin::signer::{PlanSigningKey, SpendPlan};
use primitives::{BitcoinChain, ChainSigner, SignerError, SignerInput};

use super::{plan::Plan, signing::Signer};

pub struct ZcashChainSigner;

impl ChainSigner for ZcashChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        self.sign_plan(input, &Plan::transfer(input)?, private_key)
    }

    fn sign_swap(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        Ok(vec![self.sign_plan(input, &Plan::swap(input)?, private_key)?])
    }
}

impl ZcashChainSigner {
    fn sign_plan(&self, input: &SignerInput, plan: &SpendPlan, private_key: &[u8]) -> Result<String, SignerError> {
        let branch_id = input.metadata.get_zcash_branch_id().ok_or_else(|| SignerError::invalid_input("missing Zcash branch id"))?;
        let key = PlanSigningKey::new(BitcoinChain::Zcash, plan, private_key)?;
        Signer::new(branch_id, &key).sign_transparent(plan)
    }
}

#[cfg(test)]
mod tests {
    use gem_bitcoin::testkit::signer_mock::{TEST_PRIVATE_KEY, mock_transfer_input, mock_transfer_swap_input};
    use primitives::{
        TransactionInputType,
        testkit::zcash_mock::{TEST_ZCASH_TEX_ADDRESS, TEST_ZCASH_TRANSPARENT_ADDRESS},
    };

    use super::*;

    #[test]
    fn test_sign_transfer() {
        // Test vector is checked against BitGoJS.
        assert_eq!(
            ZcashChainSigner.sign_transfer(&mock_transfer_input(BitcoinChain::Zcash), &TEST_PRIVATE_KEY).unwrap(),
            "050000800a27a726f04dec4d0000000000000000010100000000000000000000000000000000000000000000000000000000000000000000006a473044022004354fd389558909b1ccfb05dd2f3c324423efcad18eb50a42cb65ebdd17513d02207dac81ff91f4108b4017329e55971c2bd6a213cf925886b95513519bd806fd9a0121031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078fffffffff0210270000000000001976a914030303030303030303030303030303030303030388ac30750000000000001976a91479b000887626b294a914501a4cd226b58b23598388ac000000"
        );
    }

    #[test]
    fn test_sign_swap() {
        let sign_swap_to = |to: &str| {
            let mut input = mock_transfer_swap_input(BitcoinChain::Zcash, "=:b:bc1qdestination:0/1/0:g1:50");
            if let TransactionInputType::Swap { swap_data, .. } = &mut input.input.input_type {
                swap_data.data.to = to.to_string();
            }
            ZcashChainSigner.sign_swap(&input, &TEST_PRIVATE_KEY).unwrap()
        };
        assert_eq!(sign_swap_to(TEST_ZCASH_TEX_ADDRESS), sign_swap_to(TEST_ZCASH_TRANSPARENT_ADDRESS));
    }

    #[test]
    fn test_sign_requires_branch_id() {
        let mut input = mock_transfer_input(BitcoinChain::Zcash);
        input.input.metadata = primitives::TransactionLoadMetadata::Bitcoin { utxos: input.metadata.get_utxos().unwrap() };
        assert_eq!(ZcashChainSigner.sign_transfer(&input, &TEST_PRIVATE_KEY).unwrap_err(), SignerError::invalid_input("missing Zcash branch id"));
    }
}

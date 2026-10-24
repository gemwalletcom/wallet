use gem_bitcoin::signer::PlanSigningKey;
use primitives::{BitcoinChain, ChainSigner, SignerError, SignerInput};

use super::{plan::ZcashPlan, signing::ZcashSigner};

pub struct ZcashChainSigner;

impl ChainSigner for ZcashChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        self.sign_plan(input, &ZcashPlan::transfer(input)?, private_key)
    }

    fn sign_swap(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        Ok(vec![self.sign_plan(input, &ZcashPlan::swap(input)?, private_key)?])
    }
}

impl ZcashChainSigner {
    fn sign_plan(&self, input: &SignerInput, plan: &ZcashPlan, private_key: &[u8]) -> Result<String, SignerError> {
        let branch_id = input.metadata.get_zcash_branch_id().ok_or_else(|| SignerError::invalid_input("missing Zcash branch id"))?;
        let key = PlanSigningKey::new(BitcoinChain::Zcash, &plan.spend, private_key)?;
        let signer = ZcashSigner::new(branch_id, &key);
        match &plan.shielded_output {
            Some(output) => signer.sign_shielded_on_proving_thread(&plan.spend, output),
            None => signer.sign_transparent(&plan.spend),
        }
    }
}

#[cfg(test)]
mod tests {
    use gem_bitcoin::testkit::{
        address_mock::mock_sender_address,
        signer_mock::{TEST_PRIVATE_KEY, mock_transfer_input, mock_transfer_swap_input},
    };
    use primitives::{
        TransactionInputType,
        testkit::zcash_mock::{TEST_ZCASH_NU6_3_BRANCH_ID, TEST_ZCASH_TEX_ADDRESS, TEST_ZCASH_TRANSPARENT_ADDRESS, mock_signer_input_with_branch_id},
    };

    use super::*;
    use crate::testkit::TEST_ZCASH_UNIFIED_ADDRESS;

    const SWIFT_COOPERATIVE_STACK_SIZE: usize = 512 * 1024;

    #[test]
    fn test_sign_transfer() {
        // Test vector is checked against BitGoJS.
        assert_eq!(
            ZcashChainSigner.sign_transfer(&mock_transfer_input(BitcoinChain::Zcash), &TEST_PRIVATE_KEY).unwrap(),
            "050000800a27a726f04dec4d0000000000000000010100000000000000000000000000000000000000000000000000000000000000000000006a473044022004354fd389558909b1ccfb05dd2f3c324423efcad18eb50a42cb65ebdd17513d02207dac81ff91f4108b4017329e55971c2bd6a213cf925886b95513519bd806fd9a0121031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078fffffffff0210270000000000001976a914030303030303030303030303030303030303030388ac30750000000000001976a91479b000887626b294a914501a4cd226b58b23598388ac000000"
        );
    }

    #[test]
    fn test_sign_swap_tex() {
        let mut input = mock_transfer_swap_input(BitcoinChain::Zcash, "=:b:bc1qdestination:0/1/0:g1:50");
        if let TransactionInputType::Swap { swap_data, .. } = &mut input.input.input_type {
            swap_data.data.to = TEST_ZCASH_TRANSPARENT_ADDRESS.to_string();
        }
        let expected = ZcashChainSigner.sign_swap(&input, &TEST_PRIVATE_KEY).unwrap();
        if let TransactionInputType::Swap { swap_data, .. } = &mut input.input.input_type {
            swap_data.data.to = TEST_ZCASH_TEX_ADDRESS.to_string();
        }
        assert_eq!(ZcashChainSigner.sign_swap(&input, &TEST_PRIVATE_KEY).unwrap(), expected);
    }

    #[test]
    fn test_sign_shielded_transfer_on_small_stack() {
        let input = mock_signer_input_with_branch_id(mock_sender_address(BitcoinChain::Zcash), TEST_ZCASH_UNIFIED_ADDRESS.to_string(), TEST_ZCASH_NU6_3_BRANCH_ID);
        let handle = std::thread::Builder::new()
            .stack_size(SWIFT_COOPERATIVE_STACK_SIZE)
            .spawn(move || ZcashChainSigner.sign_transfer(&input, &TEST_PRIVATE_KEY))
            .unwrap();

        assert!(handle.join().unwrap().unwrap().starts_with("06000080"));
    }
}

use crate::signer::models::NearTransaction;
use crate::signer::signing;
use primitives::{ChainSigner, SignerError, SignerInput};

#[derive(Default)]
pub struct NearChainSigner;

impl ChainSigner for NearChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let transaction = NearTransaction::from_transfer_input(input)?;
        signing::sign(&transaction, private_key)
    }

    fn sign_token_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let transaction = NearTransaction::from_token_transfer_input(input)?;
        signing::sign(&transaction, private_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetId, Chain, FeeOption, TransactionFee, TransactionLoadInput};

    const PRIVATE_KEY: &str = "3hoMW1HvnRLSFCLZnvPzWeoGwtdHzke34B2cTHM8rhcbG3TbuLKtShTv3DvyejnXKXKBiV7YPkLeqUHN1ghnqpFv";

    // Tests taken from https://github.com/trustwallet/wallet-core/blob/master/tests/chains/NEAR/SignerTests.cpp
    #[test]
    fn test_sign_near_transfer() {
        let private_key = bs58::decode(PRIVATE_KEY).into_vec().unwrap();

        let input = SignerInput::new(
            TransactionLoadInput::mock_near("test.near", "whatever.near", "1", 1, "244ZQ9cgj3CQ6bWBdytfrJMuMQ1jdXLFGnr4HhvtCTnM"),
            TransactionFee::new_from_fee(0.into(), AssetId::from_chain(Chain::Near)),
        );

        let signed = NearChainSigner.sign_transfer(&input, &private_key[..32]).unwrap();

        assert_eq!(
            signed,
            "CQAAAHRlc3QubmVhcgCRez0mjUtY9/7BsVC9aNab4+5dTMOYVeNBU4Rlu3eGDQEAAAAAAAAADQAAAHdoYXRldmVyLm5lYXIPpHP9JpAd8pa+atxMxN800EDvokNSJLaYaRDmMML+9gEAAAADAQAAAAAAAAAAAAAAAAAAAACWmoMzIYbul1Xkg5MlUlgG4Ymj0tK7S0dg6URD6X4cTyLe7vAFmo6XExAO2m4ZFE2n6KDvflObIHCLodjQIb0B"
        );
    }

    #[test]
    fn test_sign_near_token_transfer() {
        let private_key = bs58::decode(PRIVATE_KEY).into_vec().unwrap();
        let input = SignerInput::mock_near_token_transfer(Some("invoice-1"), TransactionFee::new_from_fee(0.into(), AssetId::from_chain(Chain::Near)));

        let signed = NearChainSigner.sign_token_transfer(&input, &private_key[..32]).unwrap();
        assert_eq!(
            signed,
            "CQAAAHRlc3QubmVhcgCRez0mjUtY9/7BsVC9aNab4+5dTMOYVeNBU4Rlu3eGDQEAAAAAAAAAFgAAAHVzZHQudGV0aGVyLXRva2VuLm5lYXIPpHP9JpAd8pa+atxMxN800EDvokNSJLaYaRDmMML+9gEAAAACCwAAAGZ0X3RyYW5zZmVyRQAAAHsicmVjZWl2ZXJfaWQiOiJyZWNlaXZlci5uZWFyIiwiYW1vdW50IjoiMTAwMDAwMCIsIm1lbW8iOiJpbnZvaWNlLTEifQDgV+tIGwAAAQAAAAAAAAAAAAAAAAAAAAB6kapBZJavaSGAEX6IpNv5yB3d6ANBCaEe5nAfYcvUbfJaiBiOjnaSGRbfqNoZ/g9RVbt89nL4yiI8S2owwPYM"
        );
    }

    #[test]
    fn test_sign_near_token_transfer_with_registration() {
        let private_key = bs58::decode(PRIVATE_KEY).into_vec().unwrap();
        let fee = TransactionFee::new_from_fee_with_option(0.into(), FeeOption::TokenAccountCreation, 1_250_000_000_000_000_000_000u128.into(), AssetId::from_chain(Chain::Near));
        let input = SignerInput::mock_near_token_transfer(None, fee);

        let signed = NearChainSigner.sign_token_transfer(&input, &private_key[..32]).unwrap();
        assert_eq!(
            signed,
            "CQAAAHRlc3QubmVhcgCRez0mjUtY9/7BsVC9aNab4+5dTMOYVeNBU4Rlu3eGDQEAAAAAAAAAFgAAAHVzZHQudGV0aGVyLXRva2VuLm5lYXIPpHP9JpAd8pa+atxMxN800EDvokNSJLaYaRDmMML+9gIAAAACDwAAAHN0b3JhZ2VfZGVwb3NpdDcAAAB7ImFjY291bnRfaWQiOiJyZWNlaXZlci5uZWFyIiwicmVnaXN0cmF0aW9uX29ubHkiOnRydWV9AOBX60gbAAAAAEhWNxk8w0MAAAAAAAAAAgsAAABmdF90cmFuc2ZlcjIAAAB7InJlY2VpdmVyX2lkIjoicmVjZWl2ZXIubmVhciIsImFtb3VudCI6IjEwMDAwMDAifQDgV+tIGwAAAQAAAAAAAAAAAAAAAAAAAAAF1xK8j272al+nh0udGu4ZaHIQcYt+UCFh7epCChVcDrJwWynGfpwPq2be54erTHM42SQI+l+wy/uBPMEXVUoK"
        );
    }
}

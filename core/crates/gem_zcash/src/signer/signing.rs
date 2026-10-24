use bitcoin::secp256k1::{Secp256k1, SignOnly};
use gem_bitcoin::signer::{PlanSigningKey, SpendPlan};
use primitives::SignerError;

use super::transaction::{ZcashSignatureDigests, ZcashTransaction, ZcashTransactionVersion};

pub(super) struct ZcashSigner<'a> {
    pub(super) branch_id: u32,
    pub(super) key: &'a PlanSigningKey,
    pub(super) secp: Secp256k1<SignOnly>,
}

impl<'a> ZcashSigner<'a> {
    pub(super) fn new(branch_id: u32, key: &'a PlanSigningKey) -> Self {
        Self {
            branch_id,
            key,
            secp: Secp256k1::signing_only(),
        }
    }

    pub(super) fn sign_transparent(&self, plan: &SpendPlan) -> Result<String, SignerError> {
        let mut transaction = ZcashTransaction::unsigned(ZcashTransactionVersion::V5, plan, self.branch_id);
        let digests = ZcashSignatureDigests::new(&transaction, plan, None)?;
        transaction.sign_inputs(&digests, plan, self.key, &self.secp)?;
        Ok(hex::encode(transaction.encode()))
    }
}

#[cfg(test)]
mod tests {
    use gem_bitcoin::testkit::{
        address_mock::{mock_sender_address, mock_zec_address},
        signer_mock::TEST_PRIVATE_KEY,
    };
    use primitives::{BitcoinChain, testkit::zcash_mock};

    use super::*;
    use crate::signer::{plan::ZcashPlan, transaction::SIGHASH_ALL};

    #[test]
    fn test_sign_transparent() {
        let input = zcash_mock::mock_signer_input(mock_sender_address(BitcoinChain::Zcash), mock_zec_address([2u8; 20]));
        let plan = ZcashPlan::transfer(&input).unwrap().spend;
        let branch_id = input.metadata.get_zcash_branch_id().unwrap();

        let transaction = ZcashTransaction::unsigned(ZcashTransactionVersion::V5, &plan, branch_id);
        let digests = ZcashSignatureDigests::new(&transaction, &plan, None).unwrap();
        assert_eq!(hex::encode(digests.signature(&plan, Some(0)).unwrap()), "0e9508ded3c1bbbf0a153622e1b5dee4303c33d45bcaa7fa1218cab57feeb065");

        let key = PlanSigningKey::new(BitcoinChain::Zcash, &plan, &TEST_PRIVATE_KEY).unwrap();
        let bytes = hex::decode(ZcashSigner::new(branch_id, &key).sign_transparent(&plan).unwrap()).unwrap();
        assert_eq!(&bytes[..20], hex::decode("050000800a27a726f04dec4d0000000000000000").unwrap().as_slice());
        assert_eq!(plan.fee, 10_000);
        assert_eq!(bytes[20], 1);

        let script_len_index = 20 + 1 + 32 + 4;
        let script_len = bytes[script_len_index] as usize;
        let signature_len = bytes[script_len_index + 1] as usize;
        assert!(script_len > 0);
        assert_eq!(bytes[script_len_index + 2 + signature_len - 1], SIGHASH_ALL);
        assert_eq!(*bytes.last().unwrap(), 0);
    }
}

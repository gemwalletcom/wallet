use std::thread;

use gem_bitcoin::signer::SpendPlan;
use orchard::bundle::TxVersion;
use primitives::{SignerError, hex};
use rand::{CryptoRng, Rng};

use super::bundle;
use crate::signer::{
    plan::ShieldedOutput,
    signing::Signer,
    transaction::{SignatureDigests, Transaction, TransactionVersion},
};

const NU6_3_BRANCH_ID: u32 = 0x37a5_165b;
const NU7_BRANCH_ID: u32 = 0x7719_0ad9;
const SHIELDED_BRANCH_IDS: [u32; 2] = [NU6_3_BRANCH_ID, NU7_BRANCH_ID];
const PROVING_STACK_SIZE: usize = 16 * 1024 * 1024;

impl Signer<'_> {
    pub(in crate::signer) fn sign_shielded_on_proving_thread(&self, plan: &SpendPlan, output: &ShieldedOutput) -> Result<String, SignerError> {
        thread::scope(|scope| {
            let handle = thread::Builder::new()
                .stack_size(PROVING_STACK_SIZE)
                .spawn_scoped(scope, || self.sign_shielded(plan, output, &mut rand::rng()))
                .map_err(SignerError::from_display)?;
            handle.join().unwrap_or_else(|_| Err(SignerError::signing_error("Zcash proving thread panicked")))
        })
    }

    fn sign_shielded<R: Rng + CryptoRng>(&self, plan: &SpendPlan, output: &ShieldedOutput, rng: &mut R) -> Result<String, SignerError> {
        if !SHIELDED_BRANCH_IDS.contains(&self.branch_id) {
            return SignerError::invalid_input_err("unsupported Zcash network upgrade for shielded transfers");
        }
        let unauthorized = output.ironwood_bundle(rng)?;
        let commitment = unauthorized.commitment(TxVersion::V6).map_err(SignerError::from_display)?;
        let mut transaction = Transaction::unsigned(TransactionVersion::V6, plan, self.branch_id);
        let digests = SignatureDigests::new(&transaction, plan, Some(commitment.0.as_bytes()))?;
        let shielded_sighash = digests.signature(plan, None)?;
        let authorized = unauthorized
            .create_proof(bundle::proving_key(), &mut *rng)
            .and_then(|bundle| bundle.apply_signatures(&mut *rng, shielded_sighash, &[]))
            .map_err(SignerError::from_display)?;
        transaction.sign_inputs(&digests, plan, self.key, &self.secp)?;

        let mut bytes = transaction.encode();
        bytes.extend(bundle::encode(&authorized));
        Ok(hex::encode(bytes))
    }
}

#[cfg(test)]
mod tests {
    use gem_bitcoin::{
        signer::PlanSigningKey,
        testkit::{address_mock::mock_sender_address, signer_mock::TEST_PRIVATE_KEY},
    };
    use primitives::{
        BitcoinChain, ChainSigner,
        testkit::zcash_mock::{TEST_ZCASH_NU6_3_BRANCH_ID, mock_signer_input_with_branch_id},
    };
    use rand::{SeedableRng, rngs::StdRng};

    use super::*;
    use crate::{
        signer::{ZcashChainSigner, plan::Plan},
        testkit::TEST_ZCASH_UNIFIED_ADDRESS,
    };

    #[test]
    fn test_sign_shielded() {
        let input = mock_signer_input_with_branch_id(mock_sender_address(BitcoinChain::Zcash), TEST_ZCASH_UNIFIED_ADDRESS.to_string(), TEST_ZCASH_NU6_3_BRANCH_ID);
        let plan = Plan::transfer(&input).unwrap();
        let key = PlanSigningKey::new(BitcoinChain::Zcash, &plan.spend, &TEST_PRIVATE_KEY).unwrap();
        let raw = Signer::new(NU6_3_BRANCH_ID, &key).sign_shielded(&plan.spend, plan.shielded_output.as_ref().unwrap(), &mut StdRng::seed_from_u64(1)).unwrap();

        // Test vector is checked against zcash_primitives 0.31.0-pre.2: parse round trip, proof, signatures and note decryption.
        assert_eq!(raw, include_str!("../../../testdata/zcash_shielded_transfer.hex"));

        let nu7 = Signer::new(NU7_BRANCH_ID, &key).sign_shielded(&plan.spend, plan.shielded_output.as_ref().unwrap(), &mut StdRng::seed_from_u64(1)).unwrap();
        assert_eq!(&nu7[16..24], "d90a1977");

        let other_branch = mock_signer_input_with_branch_id(mock_sender_address(BitcoinChain::Zcash), TEST_ZCASH_UNIFIED_ADDRESS.to_string(), "5437f330");
        assert_eq!(
            ZcashChainSigner.sign_transfer(&other_branch, &TEST_PRIVATE_KEY).unwrap_err(),
            SignerError::invalid_input("unsupported Zcash network upgrade for shielded transfers")
        );
    }
}

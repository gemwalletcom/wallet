use bitcoin::secp256k1::{Secp256k1, SignOnly};
use gem_bitcoin::signer::{PlanSigningKey, SpendPlan};
use primitives::{SignerError, hex};

use super::transaction::{SignatureDigests, Transaction, TransactionVersion};

pub(super) struct Signer<'a> {
    pub(super) branch_id: u32,
    pub(super) key: &'a PlanSigningKey,
    pub(super) secp: Secp256k1<SignOnly>,
}

impl<'a> Signer<'a> {
    pub(super) fn new(branch_id: u32, key: &'a PlanSigningKey) -> Self {
        Self {
            branch_id,
            key,
            secp: Secp256k1::signing_only(),
        }
    }

    pub(super) fn sign_transparent(&self, plan: &SpendPlan) -> Result<String, SignerError> {
        let mut transaction = Transaction::unsigned(TransactionVersion::V5, plan, self.branch_id);
        let digests = SignatureDigests::new(&transaction, plan, None)?;
        transaction.sign_inputs(&digests, plan, self.key, &self.secp)?;
        Ok(hex::encode(transaction.encode()))
    }
}

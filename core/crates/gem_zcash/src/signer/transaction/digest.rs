use bitcoin::consensus::encode::serialize;
use gem_bitcoin::signer::{PlanInput, SpendPlan};
use gem_hash::blake2::blake2b_256_personal;
use primitives::SignerError;

use super::Transaction;
use crate::signer::personalization::*;

pub(crate) const SIGHASH_ALL: u8 = 0x01;

pub(crate) struct SignatureDigests {
    branch_id: u32,
    header: [u8; 32],
    prevouts: [u8; 32],
    amounts: [u8; 32],
    script_pubkeys: [u8; 32],
    sequences: [u8; 32],
    outputs: [u8; 32],
    sapling: [u8; 32],
    orchard: [u8; 32],
}

impl SignatureDigests {
    pub(crate) fn new(transaction: &Transaction, plan: &SpendPlan) -> Result<Self, SignerError> {
        let prevouts = plan.inputs.iter().flat_map(|input| serialize(&input.previous_output)).collect::<Vec<_>>();
        let amounts = plan.inputs.iter().map(Self::signed_value_bytes).collect::<Result<Vec<_>, _>>()?.concat();
        let script_pubkeys = plan.inputs.iter().flat_map(|input| serialize(input.script_pubkey.as_script())).collect::<Vec<_>>();
        let sequences = plan.inputs.iter().flat_map(|input| input.sequence.to_le_bytes()).collect::<Vec<_>>();
        let outputs = transaction.outputs.iter().flat_map(serialize).collect::<Vec<_>>();

        Ok(Self {
            branch_id: transaction.branch_id,
            header: blake2b_256_personal(&transaction.header_bytes(), TXID_HEADERS_HASH_PERSONALIZATION),
            prevouts: blake2b_256_personal(&prevouts, TXID_PREVOUTS_HASH_PERSONALIZATION),
            amounts: blake2b_256_personal(&amounts, TRANSPARENT_AMOUNTS_HASH_PERSONALIZATION),
            script_pubkeys: blake2b_256_personal(&script_pubkeys, TRANSPARENT_SCRIPTS_HASH_PERSONALIZATION),
            sequences: blake2b_256_personal(&sequences, TXID_SEQUENCES_HASH_PERSONALIZATION),
            outputs: blake2b_256_personal(&outputs, TXID_OUTPUTS_HASH_PERSONALIZATION),
            sapling: blake2b_256_personal(&[], TXID_SAPLING_HASH_PERSONALIZATION),
            orchard: blake2b_256_personal(&[], TXID_ORCHARD_HASH_PERSONALIZATION),
        })
    }

    pub(crate) fn signature(&self, plan: &SpendPlan, input_index: usize) -> Result<[u8; 32], SignerError> {
        let mut bytes = Vec::with_capacity(32 * 4);
        bytes.extend_from_slice(&self.header);
        bytes.extend_from_slice(&self.transparent(plan, input_index)?);
        bytes.extend_from_slice(&self.sapling);
        bytes.extend_from_slice(&self.orchard);
        Ok(blake2b_256_personal(&bytes, &self.personalization()))
    }

    fn transparent(&self, plan: &SpendPlan, input_index: usize) -> Result<[u8; 32], SignerError> {
        let mut bytes = Vec::with_capacity(1 + 32 * 6);
        bytes.push(SIGHASH_ALL);
        bytes.extend_from_slice(&self.prevouts);
        bytes.extend_from_slice(&self.amounts);
        bytes.extend_from_slice(&self.script_pubkeys);
        bytes.extend_from_slice(&self.sequences);
        bytes.extend_from_slice(&self.outputs);
        bytes.extend_from_slice(&Self::txin(plan, input_index)?);
        Ok(blake2b_256_personal(&bytes, TXID_TRANSPARENT_HASH_PERSONALIZATION))
    }

    fn txin(plan: &SpendPlan, input_index: usize) -> Result<[u8; 32], SignerError> {
        let input = plan.inputs.get(input_index).ok_or_else(|| SignerError::signing_error("Zcash input index out of bounds"))?;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&serialize(&input.previous_output));
        bytes.extend_from_slice(&Self::signed_value_bytes(input)?);
        bytes.extend_from_slice(&serialize(input.script_pubkey.as_script()));
        bytes.extend_from_slice(&input.sequence.to_le_bytes());
        Ok(blake2b_256_personal(&bytes, TXIN_HASH_PERSONALIZATION))
    }

    fn signed_value_bytes(input: &PlanInput) -> Result<[u8; 8], SignerError> {
        let value = i64::try_from(input.value.to_sat()).map_err(|_| SignerError::invalid_input("invalid Zcash UTXO amount"))?;
        Ok(value.to_le_bytes())
    }

    fn personalization(&self) -> [u8; 16] {
        let mut personal = [0u8; 16];
        personal[..12].copy_from_slice(TX_HASH_PERSONALIZATION_PREFIX);
        personal[12..].copy_from_slice(&self.branch_id.to_le_bytes());
        personal
    }
}

#[cfg(test)]
mod tests {
    use gem_bitcoin::testkit::address_mock::{mock_addr_by_hash, mock_sender_address};
    use primitives::{BitcoinChain, hex, testkit::zcash_mock};

    use super::*;
    use crate::signer::plan::Plan;

    #[test]
    fn test_signature() {
        let input = zcash_mock::mock_signer_input(mock_sender_address(BitcoinChain::Zcash), mock_addr_by_hash(BitcoinChain::Zcash, [2u8; 20]));
        let plan = Plan::transfer(&input).unwrap();
        let transaction = Transaction::unsigned(&plan, input.metadata.get_zcash_branch_id().unwrap());
        let digests = SignatureDigests::new(&transaction, &plan).unwrap();

        assert_eq!(plan.fee, 10_000);
        assert_eq!(hex::encode(digests.signature(&plan, 0).unwrap()), "0e9508ded3c1bbbf0a153622e1b5dee4303c33d45bcaa7fa1218cab57feeb065");
    }
}

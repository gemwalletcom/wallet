use bitcoin::consensus::encode::serialize;
use gem_bitcoin::signer::{PlanInput, SpendPlan};
use gem_hash::blake2::blake2b_256_personal;
use orchard::{ValuePool, bundle::commitments::hash_bundle_txid_empty};
use primitives::SignerError;

use super::{SIGHASH_ALL, ZcashTransaction};
use crate::signer::personalization::*;

pub(in crate::signer) struct ZcashSignatureDigests {
    branch_id: u32,
    header: [u8; 32],
    prevouts: [u8; 32],
    amounts: [u8; 32],
    script_pubkeys: [u8; 32],
    sequences: [u8; 32],
    outputs: [u8; 32],
    shielded: Vec<[u8; 32]>,
}

impl ZcashSignatureDigests {
    pub(in crate::signer) fn new(transaction: &ZcashTransaction, plan: &SpendPlan, ironwood: Option<&[u8]>) -> Result<Self, SignerError> {
        let prevouts = plan.inputs.iter().flat_map(|input| serialize(&input.previous_output)).collect::<Vec<_>>();
        let amounts = plan.inputs.iter().map(Self::signed_value_bytes).collect::<Result<Vec<_>, _>>()?.concat();
        let script_pubkeys = plan.inputs.iter().flat_map(|input| serialize(input.script_pubkey.as_script())).collect::<Vec<_>>();
        let sequences = plan.inputs.iter().flat_map(|input| input.sequence.to_le_bytes()).collect::<Vec<_>>();
        let outputs = transaction.outputs.iter().flat_map(serialize).collect::<Vec<_>>();
        let empty_orchard = hash_bundle_txid_empty(ValuePool::Orchard, transaction.version.tx_version()).map_err(SignerError::from_display)?;
        let shielded = [blake2b_256_personal(&[], ZCASH_TXID_SAPLING_HASH_PERSONALIZATION).as_slice(), empty_orchard.as_bytes()]
            .into_iter()
            .chain(ironwood)
            .map(|digest| digest.try_into().map_err(SignerError::from_display))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            branch_id: transaction.branch_id,
            header: blake2b_256_personal(&transaction.header_bytes(), ZCASH_TXID_HEADERS_HASH_PERSONALIZATION),
            prevouts: blake2b_256_personal(&prevouts, ZCASH_TXID_PREVOUTS_HASH_PERSONALIZATION),
            amounts: blake2b_256_personal(&amounts, ZCASH_TRANSPARENT_AMOUNTS_HASH_PERSONALIZATION),
            script_pubkeys: blake2b_256_personal(&script_pubkeys, ZCASH_TRANSPARENT_SCRIPTS_HASH_PERSONALIZATION),
            sequences: blake2b_256_personal(&sequences, ZCASH_TXID_SEQUENCES_HASH_PERSONALIZATION),
            outputs: blake2b_256_personal(&outputs, ZCASH_TXID_OUTPUTS_HASH_PERSONALIZATION),
            shielded,
        })
    }

    pub(in crate::signer) fn signature(&self, plan: &SpendPlan, input_index: Option<usize>) -> Result<[u8; 32], SignerError> {
        let mut bytes = Vec::with_capacity(32 * (2 + self.shielded.len()));
        bytes.extend_from_slice(&self.header);
        bytes.extend_from_slice(&self.transparent(plan, input_index)?);
        for digest in &self.shielded {
            bytes.extend_from_slice(digest);
        }
        Ok(blake2b_256_personal(&bytes, &self.personalization()))
    }

    fn transparent(&self, plan: &SpendPlan, input_index: Option<usize>) -> Result<[u8; 32], SignerError> {
        let mut bytes = Vec::with_capacity(1 + 32 * 6);
        bytes.push(SIGHASH_ALL);
        bytes.extend_from_slice(&self.prevouts);
        bytes.extend_from_slice(&self.amounts);
        bytes.extend_from_slice(&self.script_pubkeys);
        bytes.extend_from_slice(&self.sequences);
        bytes.extend_from_slice(&self.outputs);
        bytes.extend_from_slice(&Self::txin(plan, input_index)?);
        Ok(blake2b_256_personal(&bytes, ZCASH_TXID_TRANSPARENT_HASH_PERSONALIZATION))
    }

    fn txin(plan: &SpendPlan, input_index: Option<usize>) -> Result<[u8; 32], SignerError> {
        let Some(input_index) = input_index else {
            return Ok(blake2b_256_personal(&[], ZCASH_TXIN_HASH_PERSONALIZATION));
        };
        let input = plan.inputs.get(input_index).ok_or_else(|| SignerError::signing_error("Zcash input index out of bounds"))?;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&serialize(&input.previous_output));
        bytes.extend_from_slice(&Self::signed_value_bytes(input)?);
        bytes.extend_from_slice(&serialize(input.script_pubkey.as_script()));
        bytes.extend_from_slice(&input.sequence.to_le_bytes());
        Ok(blake2b_256_personal(&bytes, ZCASH_TXIN_HASH_PERSONALIZATION))
    }

    fn signed_value_bytes(input: &PlanInput) -> Result<[u8; 8], SignerError> {
        let value = i64::try_from(input.value.to_sat()).map_err(|_| SignerError::invalid_input("invalid Zcash UTXO amount"))?;
        Ok(value.to_le_bytes())
    }

    fn personalization(&self) -> [u8; 16] {
        let mut personal = [0u8; 16];
        personal[..12].copy_from_slice(ZCASH_TX_HASH_PERSONALIZATION_PREFIX);
        personal[12..].copy_from_slice(&self.branch_id.to_le_bytes());
        personal
    }
}

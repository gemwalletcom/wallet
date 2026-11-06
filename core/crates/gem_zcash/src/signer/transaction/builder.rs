use bitcoin::{
    ScriptBuf, Sequence, TxIn, TxOut, Witness,
    blockdata::script::Builder,
    consensus::encode::serialize,
    secp256k1::{Message, Secp256k1, Signing},
};
use gem_bitcoin::signer::{PlanSigningKey, SpendPlan, der_signature, encoding::encode_varint, signature_push_bytes};
use primitives::SignerError;

use super::{SIGHASH_ALL, SignatureDigests, TransactionVersion};

const HEADER_LEN: usize = 20;
const LOCK_TIME: u32 = 0;
const EXPIRY_HEIGHT_DISABLED: u32 = 0;

#[derive(Debug, Clone)]
pub(in crate::signer) struct Transaction {
    pub(super) version: TransactionVersion,
    pub(super) branch_id: u32,
    inputs: Vec<TxIn>,
    pub(super) outputs: Vec<TxOut>,
}

impl Transaction {
    pub(in crate::signer) fn unsigned(version: TransactionVersion, plan: &SpendPlan, branch_id: u32) -> Self {
        let inputs = plan
            .inputs
            .iter()
            .map(|input| TxIn {
                previous_output: input.previous_output,
                script_sig: ScriptBuf::new(),
                sequence: Sequence(input.sequence),
                witness: Witness::default(),
            })
            .collect();

        let outputs = plan
            .outputs
            .iter()
            .map(|output| TxOut {
                value: output.value,
                script_pubkey: output.script_pubkey.clone(),
            })
            .collect();

        Self { version, branch_id, inputs, outputs }
    }

    pub(in crate::signer) fn sign_inputs<C: Signing>(&mut self, digests: &SignatureDigests, plan: &SpendPlan, key: &PlanSigningKey, secp: &Secp256k1<C>) -> Result<(), SignerError> {
        for index in 0..plan.inputs.len() {
            let sighash = digests.signature(plan, Some(index))?;
            let signature = der_signature(secp, key.secret_key(), Message::from_digest(sighash), SIGHASH_ALL);
            self.inputs[index].script_sig = Builder::new().push_slice(signature_push_bytes(signature)?).push_key(key.public_key()).into_script();
        }
        Ok(())
    }

    pub(in crate::signer) fn encode(&self) -> Vec<u8> {
        let mut bytes = self.header_bytes();

        bytes.extend_from_slice(&encode_varint(self.inputs.len()));
        for input in &self.inputs {
            bytes.extend_from_slice(&serialize(input));
        }

        bytes.extend_from_slice(&encode_varint(self.outputs.len()));
        for output in &self.outputs {
            bytes.extend_from_slice(&serialize(output));
        }

        bytes.extend_from_slice(&encode_varint(0));
        bytes.extend_from_slice(&encode_varint(0));
        bytes.extend_from_slice(&encode_varint(0));
        bytes
    }

    pub(super) fn header_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(HEADER_LEN);
        bytes.extend_from_slice(&self.version.header().to_le_bytes());
        bytes.extend_from_slice(&self.version.version_group_id().to_le_bytes());
        bytes.extend_from_slice(&self.branch_id.to_le_bytes());
        bytes.extend_from_slice(&LOCK_TIME.to_le_bytes());
        bytes.extend_from_slice(&EXPIRY_HEIGHT_DISABLED.to_le_bytes());
        bytes
    }
}

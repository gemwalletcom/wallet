mod digest;

use bitcoin::{
    ScriptBuf, Sequence, TxIn as TransactionInput, TxOut as TransactionOutput, Witness,
    blockdata::script::Builder,
    consensus::encode::serialize,
    secp256k1::{Message, Secp256k1, Signing},
};
use gem_bitcoin::signer::{PlanSigningKey, SpendPlan, der_signature, encoding::encode_varint, signature_push_bytes};
use orchard::bundle::TxVersion;
use primitives::SignerError;

pub(super) use digest::ZcashSignatureDigests;

const OVERWINTERED_VERSION_5: u32 = 0x8000_0005;
const OVERWINTERED_VERSION_6: u32 = 0x8000_0006;
const VERSION_GROUP_ID_V5: u32 = 0x26a7_270a;
const VERSION_GROUP_ID_V6: u32 = 0xd884_b698;
const LOCK_TIME: u32 = 0;
const EXPIRY_HEIGHT_DISABLED: u32 = 0;
pub(super) const SIGHASH_ALL: u8 = 0x01;

#[derive(Debug, Clone, Copy)]
pub(super) enum ZcashTransactionVersion {
    V5,
    V6,
}

impl ZcashTransactionVersion {
    fn header(self) -> u32 {
        match self {
            Self::V5 => OVERWINTERED_VERSION_5,
            Self::V6 => OVERWINTERED_VERSION_6,
        }
    }

    fn version_group_id(self) -> u32 {
        match self {
            Self::V5 => VERSION_GROUP_ID_V5,
            Self::V6 => VERSION_GROUP_ID_V6,
        }
    }

    fn tx_version(self) -> TxVersion {
        match self {
            Self::V5 => TxVersion::V5,
            Self::V6 => TxVersion::V6,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct ZcashTransaction {
    version: ZcashTransactionVersion,
    branch_id: u32,
    inputs: Vec<TransactionInput>,
    outputs: Vec<TransactionOutput>,
}

impl ZcashTransaction {
    pub(super) fn unsigned(version: ZcashTransactionVersion, plan: &SpendPlan, branch_id: u32) -> Self {
        let inputs = plan
            .inputs
            .iter()
            .map(|input| TransactionInput {
                previous_output: input.previous_output,
                script_sig: ScriptBuf::new(),
                sequence: Sequence(input.sequence),
                witness: Witness::default(),
            })
            .collect();

        let outputs = plan
            .outputs
            .iter()
            .map(|output| TransactionOutput {
                value: output.value,
                script_pubkey: output.script_pubkey.clone(),
            })
            .collect();

        Self { version, branch_id, inputs, outputs }
    }

    pub(super) fn sign_inputs<C: Signing>(&mut self, digests: &ZcashSignatureDigests, plan: &SpendPlan, key: &PlanSigningKey, secp: &Secp256k1<C>) -> Result<(), SignerError> {
        for index in 0..plan.inputs.len() {
            let sighash = digests.signature(plan, Some(index))?;
            let signature = der_signature(secp, key.secret_key(), Message::from_digest(sighash), SIGHASH_ALL);
            self.inputs[index].script_sig = Builder::new().push_slice(signature_push_bytes(signature)?).push_key(key.public_key()).into_script();
        }
        Ok(())
    }

    pub(super) fn encode(&self) -> Vec<u8> {
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

    fn header_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(20);
        bytes.extend_from_slice(&self.version.header().to_le_bytes());
        bytes.extend_from_slice(&self.version.version_group_id().to_le_bytes());
        bytes.extend_from_slice(&self.branch_id.to_le_bytes());
        bytes.extend_from_slice(&LOCK_TIME.to_le_bytes());
        bytes.extend_from_slice(&EXPIRY_HEIGHT_DISABLED.to_le_bytes());
        bytes
    }
}

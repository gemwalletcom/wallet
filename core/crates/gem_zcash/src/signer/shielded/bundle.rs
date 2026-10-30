use std::sync::OnceLock;

use gem_bitcoin::signer::encoding::encode_varint;
use orchard::{
    Anchor, Bundle,
    builder::{Builder, BundleType, UnauthorizedBundle},
    bundle::{Authorized, BundleVersion, Flags},
    circuit::ProvingKey,
    value::NoteValue,
};
use primitives::SignerError;
use rand::Rng;

use crate::signer::plan::ShieldedOutput;

const IRONWOOD_BUNDLE_VERSION: BundleVersion = BundleVersion::ironwood_v3();

impl ShieldedOutput {
    pub(super) fn ironwood_bundle<R: Rng>(&self, rng: &mut R) -> Result<UnauthorizedBundle<i64>, SignerError> {
        let mut builder = Builder::new(BundleType::DEFAULT, IRONWOOD_BUNDLE_VERSION, Flags::SPENDS_DISABLED, Anchor::empty_tree()).map_err(SignerError::from_display)?;
        builder.add_output(None, self.recipient, NoteValue::from_raw(self.value), self.memo_bytes()).map_err(SignerError::from_display)?;
        let (bundle, _) = builder.build::<i64>(rng).map_err(SignerError::from_display)?.ok_or_else(|| SignerError::signing_error("missing Zcash shielded bundle"))?;
        Ok(bundle)
    }
}

pub(super) fn proving_key() -> &'static ProvingKey {
    static PROVING_KEY: OnceLock<ProvingKey> = OnceLock::new();
    PROVING_KEY.get_or_init(|| ProvingKey::build(IRONWOOD_BUNDLE_VERSION.circuit_version()))
}

pub(super) fn encode(bundle: &Bundle<Authorized, i64>) -> Vec<u8> {
    let mut bytes = encode_varint(bundle.actions().len());
    for action in bundle.actions() {
        let note = action.encrypted_note();
        bytes.extend_from_slice(&action.cv_net().to_bytes());
        bytes.extend_from_slice(&action.nullifier().to_bytes());
        bytes.extend_from_slice(&<[u8; 32]>::from(action.rk()));
        bytes.extend_from_slice(&action.cmx().to_bytes());
        bytes.extend_from_slice(&note.epk_bytes);
        bytes.extend_from_slice(note.enc_ciphertext.as_ref());
        bytes.extend_from_slice(&note.out_ciphertext);
    }
    bytes.push(bundle.flag_byte());
    bytes.extend_from_slice(&bundle.value_balance().to_le_bytes());
    bytes.extend_from_slice(&bundle.anchor().to_bytes());
    let proof = bundle.authorization().proof().as_ref();
    bytes.extend_from_slice(&encode_varint(proof.len()));
    bytes.extend_from_slice(proof);
    for action in bundle.actions() {
        bytes.extend_from_slice(&<[u8; 64]>::from(action.authorization()));
    }
    bytes.extend_from_slice(&<[u8; 64]>::from(bundle.authorization().binding_signature()));
    bytes
}

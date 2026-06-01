use k256::ecdsa::SigningKey as SecpSigningKey;
use primitives::SignerError;

pub const SIGNATURE_LENGTH: usize = 65;
pub const RECOVERY_ID_INDEX: usize = SIGNATURE_LENGTH - 1;
const ETHEREUM_RECOVERY_ID_OFFSET: u8 = 27;

/// Returns (signature_bytes, recovery_id) where recovery_id ∈ {0, 1}.
pub(crate) fn sign_digest(digest: &[u8], private_key: &[u8]) -> Result<(Vec<u8>, u8), SignerError> {
    let signing_key = SecpSigningKey::from_slice(private_key).map_err(|_| SignerError::signing_error("Invalid Secp256k1 private key"))?;
    let (signature, recovery_id) = signing_key
        .sign_prehash_recoverable(digest)
        .map_err(|_| SignerError::signing_error("Failed to sign Secp256k1 digest"))?;
    Ok((signature.to_bytes().to_vec(), u8::from(recovery_id)))
}

/// Returns [r(32), s(32), v(1)] where v ∈ {0, 1}.
pub(crate) fn sign_digest_append_recovery(digest: &[u8], private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
    let (rs, v) = sign_digest(digest, private_key)?;
    Ok([rs, vec![v]].concat())
}

/// Returns [r(32), s(32), v(1)] where v ∈ {27, 28} (Ethereum/Tron).
pub(crate) fn sign_ethereum_digest(digest: &[u8], private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
    let (rs, v) = sign_digest(digest, private_key)?;
    Ok([rs, vec![v + ETHEREUM_RECOVERY_ID_OFFSET]].concat())
}

pub fn public_key_from_private(private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
    let signing_key = SecpSigningKey::from_slice(private_key).map_err(|_| SignerError::invalid_input("Invalid Secp256k1 private key"))?;
    Ok(signing_key.verifying_key().to_sec1_bytes().to_vec())
}

pub fn uncompressed_public_key_from_private(private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
    let signing_key = SecpSigningKey::from_slice(private_key).map_err(|_| SignerError::invalid_input("Invalid Secp256k1 private key"))?;
    Ok(signing_key.verifying_key().to_encoded_point(false).as_bytes().to_vec())
}

/// Ensure a 65-byte signature uses Ethereum's 27/28 recovery id convention.
pub fn ensure_ethereum_signature_recovery_id_offset(signature: &mut [u8]) {
    if signature.len() != 65 {
        return;
    }
    let v = &mut signature[64];
    if *v < ETHEREUM_RECOVERY_ID_OFFSET {
        *v += ETHEREUM_RECOVERY_ID_OFFSET;
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ETHEREUM_RECOVERY_ID_OFFSET, SecpSigningKey, ensure_ethereum_signature_recovery_id_offset, sign_digest, sign_ethereum_digest, uncompressed_public_key_from_private,
    };
    use crate::testkit::TEST_PRIVATE_KEY;
    use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
    const DIGEST: [u8; 32] = [7u8; 32];

    #[test]
    fn sign_digest_returns_raw_recovery_id() {
        let private_key = hex::decode(TEST_PRIVATE_KEY).unwrap();
        let (rs, v) = sign_digest(&DIGEST, &private_key).unwrap();
        let signing_key = SecpSigningKey::from_slice(&private_key).unwrap();

        assert_eq!(rs.len(), 64);
        assert!(matches!(v, 0 | 1), "raw recovery id must be 0 or 1, got {v}");

        let recovery_id = RecoveryId::from_byte(v).unwrap();
        let signature = Signature::try_from(rs.as_slice()).unwrap();
        let recovered = VerifyingKey::recover_from_prehash(&DIGEST, &signature, recovery_id).unwrap();
        assert_eq!(recovered.to_sec1_bytes().to_vec(), signing_key.verifying_key().to_sec1_bytes().to_vec());
    }

    #[test]
    fn sign_ethereum_digest_applies_offset() {
        let private_key = hex::decode(TEST_PRIVATE_KEY).unwrap();
        let (rs, v) = sign_digest(&DIGEST, &private_key).unwrap();
        let signature = sign_ethereum_digest(&DIGEST, &private_key).unwrap();

        assert_eq!(rs, &signature[..64]);
        assert_eq!(v + ETHEREUM_RECOVERY_ID_OFFSET, signature[64]);
    }

    #[test]
    fn uncompressed_public_key_from_private_derives_sec1_key() {
        let private_key = hex::decode(TEST_PRIVATE_KEY).unwrap();
        let public_key = uncompressed_public_key_from_private(&private_key).unwrap();

        assert_eq!(public_key.len(), 65);
        assert_eq!(public_key[0], 0x04);
        assert_eq!(
            hex::encode(public_key),
            "04a73ac47eb0f40940f30eb5444a6471de077a1a1c60ab7a533b82ffdf2d86a4f9a0aad8509e3a1fdda6514b1125cc4ab532a7a6ab58c529fed6a3854e1827f426",
        );
        assert!(uncompressed_public_key_from_private(&[0u8; 16]).is_err());
    }

    #[test]
    fn ensure_ethereum_signature_recovery_id_offset_is_idempotent() {
        let mut sig = vec![0u8; 65];

        sig[64] = 0;
        ensure_ethereum_signature_recovery_id_offset(&mut sig);
        assert_eq!(sig[64], ETHEREUM_RECOVERY_ID_OFFSET);
        ensure_ethereum_signature_recovery_id_offset(&mut sig);
        assert_eq!(sig[64], ETHEREUM_RECOVERY_ID_OFFSET);

        sig[64] = 1;
        ensure_ethereum_signature_recovery_id_offset(&mut sig);
        assert_eq!(sig[64], 1 + ETHEREUM_RECOVERY_ID_OFFSET);
        ensure_ethereum_signature_recovery_id_offset(&mut sig);
        assert_eq!(sig[64], 1 + ETHEREUM_RECOVERY_ID_OFFSET);
    }
}

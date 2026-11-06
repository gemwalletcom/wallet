use alloy_primitives::{Address, Signature as EthereumSignature};
use primitives::SignerError;
use secp256k1::ecdsa::{RecoverableSignature, RecoveryId, Signature};
use secp256k1::{Message, PublicKey, SECP256K1, SecretKey};
use zeroize::Zeroizing;

const ETHEREUM_RECOVERY_ID_OFFSET: u8 = 27;
const SECRET_KEY_LENGTH: usize = 32;
const MIN_SECRET_KEY_LENGTH: usize = 24;

/// Returns (signature_bytes, recovery_id) where recovery_id ∈ {0, 1}.
pub(crate) fn sign_digest(digest: &[u8], private_key: &[u8]) -> Result<(Vec<u8>, u8), SignerError> {
    let message = Message::from_digest_slice(digest).map_err(|_| SignerError::signing_error("Secp256k1 digest must be 32 bytes"))?;
    with_secret_key(
        private_key,
        || SignerError::signing_error("Invalid Secp256k1 private key"),
        |secret_key| {
            let (recovery_id, signature) = SECP256K1.sign_ecdsa_recoverable(&message, secret_key).serialize_compact();
            (signature.to_vec(), recovery_id.to_i32() as u8)
        },
    )
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

pub(crate) fn recover_ethereum_address(digest: &[u8; 32], signature: &[u8]) -> Result<String, SignerError> {
    let invalid = || SignerError::invalid_input("Invalid Ethereum signature");
    let signature = EthereumSignature::try_from(signature).map_err(SignerError::from_display)?;
    let compact = [signature.r().to_be_bytes::<32>(), signature.s().to_be_bytes::<32>()].concat();
    let standard = Signature::from_compact(&compact).map_err(|_| invalid())?;
    let mut normalized = standard;
    normalized.normalize_s();
    if normalized != standard {
        return Err(SignerError::invalid_input("Ethereum signature is not low-S"));
    }
    let recovery_id = RecoveryId::from_i32(i32::from(signature.v())).map_err(|_| invalid())?;
    let recoverable = RecoverableSignature::from_compact(&compact, recovery_id).map_err(|_| invalid())?;
    let public_key = SECP256K1.recover_ecdsa(&Message::from_digest(*digest), &recoverable).map_err(|_| invalid())?;
    Ok(Address::from_raw_public_key(&public_key.serialize_uncompressed()[1..]).to_checksum(None))
}

pub fn public_key_from_private(private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
    with_secret_key(private_key, invalid_private_key, |secret_key| PublicKey::from_secret_key(SECP256K1, secret_key).serialize().to_vec())
}

pub fn uncompressed_public_key_from_private(private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
    with_secret_key(private_key, invalid_private_key, |secret_key| PublicKey::from_secret_key(SECP256K1, secret_key).serialize_uncompressed().to_vec())
}

pub fn uncompressed_public_key(public_key: &[u8]) -> Result<Vec<u8>, SignerError> {
    let invalid = || SignerError::invalid_input("invalid secp256k1 public key");
    let public_key = match public_key.first() {
        Some(0x02..=0x04) => PublicKey::from_slice(public_key).map_err(|_| invalid())?,
        _ => return Err(invalid()),
    };
    Ok(public_key.serialize_uncompressed().to_vec())
}

pub fn generate_private_key() -> Result<Zeroizing<[u8; SECRET_KEY_LENGTH]>, SignerError> {
    let mut bytes = Zeroizing::new([0u8; SECRET_KEY_LENGTH]);
    getrandom::fill(bytes.as_mut_slice()).map_err(SignerError::from_display)?;
    match validate_private_key(bytes.as_slice()) {
        Ok(()) => Ok(bytes),
        Err(_) => generate_private_key(),
    }
}

pub(crate) fn validate_private_key(private_key: &[u8]) -> Result<(), SignerError> {
    with_secret_key(private_key, || SignerError::invalid_input("Invalid secp256k1 private key"), |_| ())
}

fn with_secret_key<T>(private_key: &[u8], error: impl FnOnce() -> SignerError, run: impl FnOnce(&SecretKey) -> T) -> Result<T, SignerError> {
    let mut secret_key = parse_secret_key(private_key).ok_or_else(error)?;
    let result = run(&secret_key);
    secret_key.non_secure_erase();
    Ok(result)
}

fn parse_secret_key(private_key: &[u8]) -> Option<SecretKey> {
    if !(MIN_SECRET_KEY_LENGTH..=SECRET_KEY_LENGTH).contains(&private_key.len()) {
        return None;
    }
    let mut padded = Zeroizing::new([0u8; SECRET_KEY_LENGTH]);
    padded[SECRET_KEY_LENGTH - private_key.len()..].copy_from_slice(private_key);
    SecretKey::from_slice(padded.as_slice()).ok()
}

fn invalid_private_key() -> SignerError {
    SignerError::invalid_input("Invalid Secp256k1 private key")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{TEST_PRIVATE_KEY, TEST_PRIVATE_KEY_ETHEREUM_ADDRESS};
    use alloy_primitives::U256;

    const DIGEST: [u8; 32] = [7u8; 32];
    const RECOVERY_ID_INDEX: usize = 64;
    const CURVE_ORDER: &str = "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141";
    const TEST_UNCOMPRESSED_PUBLIC_KEY: &str = "04a73ac47eb0f40940f30eb5444a6471de077a1a1c60ab7a533b82ffdf2d86a4f9a0aad8509e3a1fdda6514b1125cc4ab532a7a6ab58c529fed6a3854e1827f426";

    #[test]
    fn test_sign_digest() {
        let private_key = hex::decode(TEST_PRIVATE_KEY).unwrap();
        let (rs, v) = sign_digest(&DIGEST, &private_key).unwrap();
        assert_eq!(
            hex::encode([rs.as_slice(), &[v]].concat()),
            "39b9017b040c7c8a2adeb56e09f0540878db3a5981c52b7c3beb9853ce5e6aff1ed8a577616609cdfa6dc9c671bc0d62f46304526546be28d40d1b7c6ee864b801"
        );
        assert_eq!(sign_digest(&DIGEST[..31], &private_key), Err(SignerError::signing_error("Secp256k1 digest must be 32 bytes")));
        assert_eq!(sign_digest(&[DIGEST.as_slice(), &[0]].concat(), &private_key), Err(SignerError::signing_error("Secp256k1 digest must be 32 bytes")));
        for invalid in [vec![0u8; 32], hex::decode(CURVE_ORDER).unwrap(), vec![1u8; 23], vec![1u8; 33], vec![]] {
            assert_eq!(sign_digest(&DIGEST, &invalid), Err(SignerError::signing_error("Invalid Secp256k1 private key")));
        }
    }

    #[test]
    fn test_sign_digest_left_pads_short_private_keys() {
        let private_key = [[0u8; 8].as_slice(), &hex::decode(TEST_PRIVATE_KEY).unwrap()[8..]].concat();
        for length in 24..32 {
            let short = &private_key[32 - length..];
            assert_eq!(sign_digest(&DIGEST, short), sign_digest(&DIGEST, &private_key), "{length}");
            assert_eq!(public_key_from_private(short), public_key_from_private(&private_key), "{length}");
        }
    }

    #[test]
    fn test_sign_ethereum_digest() {
        let private_key = hex::decode(TEST_PRIVATE_KEY).unwrap();
        let (rs, v) = sign_digest(&DIGEST, &private_key).unwrap();
        assert_eq!(sign_ethereum_digest(&DIGEST, &private_key).unwrap(), [rs.as_slice(), &[v + ETHEREUM_RECOVERY_ID_OFFSET]].concat());
        assert_eq!(sign_digest_append_recovery(&DIGEST, &private_key).unwrap(), [rs.as_slice(), &[v]].concat());
    }

    #[test]
    fn test_recover_ethereum_address() {
        let private_key = hex::decode(TEST_PRIVATE_KEY).unwrap();
        let mut signature = sign_digest_append_recovery(&DIGEST, &private_key).unwrap();
        assert_eq!(recover_ethereum_address(&DIGEST, &signature).unwrap(), TEST_PRIVATE_KEY_ETHEREUM_ADDRESS);

        signature[RECOVERY_ID_INDEX] += ETHEREUM_RECOVERY_ID_OFFSET;
        assert_eq!(recover_ethereum_address(&DIGEST, &signature).unwrap(), TEST_PRIVATE_KEY_ETHEREUM_ADDRESS);

        signature[RECOVERY_ID_INDEX] = 2;
        assert!(recover_ethereum_address(&DIGEST, &signature).is_err());
        assert!(recover_ethereum_address(&DIGEST, &signature[..RECOVERY_ID_INDEX]).is_err());
        assert_eq!(recover_ethereum_address(&DIGEST, &[0u8; 65]), Err(SignerError::invalid_input("Invalid Ethereum signature")));
    }

    #[test]
    fn test_recover_ethereum_address_rejects_high_s() {
        let private_key = hex::decode(TEST_PRIVATE_KEY).unwrap();
        let (rs, recovery_id) = sign_digest(&DIGEST, &private_key).unwrap();
        let order = U256::from_str_radix(CURVE_ORDER, 16).unwrap();
        let high_s = order - U256::from_be_slice(&rs[32..]);
        let forged = [&rs[..32], high_s.to_be_bytes::<32>().as_slice(), &[recovery_id ^ 1]].concat();
        assert_eq!(recover_ethereum_address(&DIGEST, &forged), Err(SignerError::invalid_input("Ethereum signature is not low-S")));

        let digest = hex::decode("5eb4f5a33c621f32a8622d5f943b6b102994dfe4e5aebbefe69bb1b2aa0fc93e").unwrap().try_into().unwrap();
        let alloy_high_s = hex::decode("48b55bfa915ac795c431978d8a6a992b628d557da5ff759b307d495a36649353efffd310ac743f371de3b9f7f9cb56c0b28ad43601b4ab949f53faa07bd2c8041b").unwrap();
        assert!(recover_ethereum_address(&digest, &alloy_high_s).is_err());
    }

    #[test]
    fn test_public_keys_from_private() {
        let private_key = hex::decode(TEST_PRIVATE_KEY).unwrap();
        let uncompressed = uncompressed_public_key_from_private(&private_key).unwrap();
        let compressed = public_key_from_private(&private_key).unwrap();
        assert_eq!(hex::encode(&uncompressed), TEST_UNCOMPRESSED_PUBLIC_KEY);
        assert_eq!(hex::encode(&compressed), "02a73ac47eb0f40940f30eb5444a6471de077a1a1c60ab7a533b82ffdf2d86a4f9");
        assert_eq!(uncompressed_public_key_from_private(&[0u8; 16]), Err(SignerError::invalid_input("Invalid Secp256k1 private key")));
    }

    #[test]
    fn test_uncompressed_public_key() {
        let uncompressed = hex::decode(TEST_UNCOMPRESSED_PUBLIC_KEY).unwrap();
        assert_eq!(uncompressed_public_key(&hex::decode("02a73ac47eb0f40940f30eb5444a6471de077a1a1c60ab7a533b82ffdf2d86a4f9").unwrap()).unwrap(), uncompressed);
        assert_eq!(uncompressed_public_key(&uncompressed).unwrap(), uncompressed);
        let hybrid = [[0x06 | (uncompressed[64] & 1)].as_slice(), &uncompressed[1..]].concat();
        for invalid in [vec![], [[0x02].as_slice(), &[0xff; 32]].concat(), vec![0x05; 33], uncompressed[..64].to_vec(), hybrid] {
            assert_eq!(uncompressed_public_key(&invalid), Err(SignerError::invalid_input("invalid secp256k1 public key")));
        }
    }

    #[test]
    fn test_generate_private_key() {
        let first = generate_private_key().unwrap();
        let second = generate_private_key().unwrap();
        assert_eq!(validate_private_key(first.as_slice()), Ok(()));
        assert_ne!(first.as_slice(), second.as_slice());
    }
}

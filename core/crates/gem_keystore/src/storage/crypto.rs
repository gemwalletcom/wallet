use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{AeadInOut, KeyInit},
};
use argon2::{Algorithm, Argon2, Params, Version};
use zeroize::Zeroizing;

use super::{
    constants::{
        AES_GCM_NONCE_LEN, AES_GCM_TAG_LEN, DEFAULT_ARGON2_ITERATIONS, DEFAULT_ARGON2_MEMORY_KIB, DEFAULT_ARGON2_OUTPUT_LEN, DEFAULT_ARGON2_PARALLELISM, MAX_ARGON2_ITERATIONS,
        MAX_ARGON2_MEMORY_KIB, MAX_ARGON2_PARALLELISM,
    },
    format::validate_v4_password,
    types::{CipherParams, KdfParams},
};
use crate::KeystoreError;

impl KdfParams {
    pub(super) fn default_argon2id() -> Result<Self, KeystoreError> {
        Ok(Self::Argon2id {
            memory_kib: DEFAULT_ARGON2_MEMORY_KIB,
            iterations: DEFAULT_ARGON2_ITERATIONS,
            parallelism: DEFAULT_ARGON2_PARALLELISM,
            salt: gem_crypto::random::bytes()?,
            output_len: DEFAULT_ARGON2_OUTPUT_LEN,
        })
    }

    pub(super) fn with_random_salt(&self) -> Result<Self, KeystoreError> {
        match self {
            Self::Argon2id {
                memory_kib,
                iterations,
                parallelism,
                output_len,
                ..
            } => Ok(Self::Argon2id {
                memory_kib: *memory_kib,
                iterations: *iterations,
                parallelism: *parallelism,
                salt: gem_crypto::random::bytes()?,
                output_len: *output_len,
            }),
        }
    }

    pub(super) fn validate(&self) -> Result<(), KeystoreError> {
        match self {
            Self::Argon2id {
                memory_kib,
                iterations,
                parallelism,
                output_len,
                ..
            } => {
                if *memory_kib == 0 || *memory_kib > MAX_ARGON2_MEMORY_KIB {
                    return Err(KeystoreError::corrupt_file("invalid Argon2 memory"));
                }
                if *iterations == 0 || *iterations > MAX_ARGON2_ITERATIONS {
                    return Err(KeystoreError::corrupt_file("invalid Argon2 iterations"));
                }
                if *parallelism == 0 || *parallelism > MAX_ARGON2_PARALLELISM {
                    return Err(KeystoreError::corrupt_file("invalid Argon2 parallelism"));
                }
                if *output_len != DEFAULT_ARGON2_OUTPUT_LEN {
                    return Err(KeystoreError::corrupt_file("invalid Argon2 output length"));
                }
                Ok(())
            }
        }
    }
}

impl CipherParams {
    pub(super) fn random_aes256_gcm() -> Result<Self, KeystoreError> {
        Ok(Self::Aes256Gcm {
            nonce: gem_crypto::random::bytes::<AES_GCM_NONCE_LEN>()?,
            tag_len: AES_GCM_TAG_LEN,
        })
    }

    pub(super) fn validate(&self) -> Result<(), KeystoreError> {
        match self {
            Self::Aes256Gcm { tag_len, .. } => {
                if *tag_len != AES_GCM_TAG_LEN {
                    return Err(KeystoreError::corrupt_file("invalid AES-GCM tag length"));
                }
                Ok(())
            }
        }
    }

    pub(super) fn nonce(&self) -> &[u8; AES_GCM_NONCE_LEN] {
        match self {
            Self::Aes256Gcm { nonce, .. } => nonce,
        }
    }

    pub(super) fn tag_len(&self) -> u8 {
        match self {
            Self::Aes256Gcm { tag_len, .. } => *tag_len,
        }
    }
}

pub(super) fn derive_key(password: &[u8], kdf: &KdfParams) -> Result<Zeroizing<[u8; 32]>, KeystoreError> {
    validate_v4_password(password)?;
    kdf.validate()?;
    match kdf {
        KdfParams::Argon2id {
            memory_kib,
            iterations,
            parallelism,
            salt,
            output_len,
        } => {
            let params = Params::new(
                *memory_kib,
                *iterations,
                *parallelism,
                Some(usize::try_from(*output_len).map_err(|_| KeystoreError::corrupt_file("invalid key length"))?),
            )
            .map_err(|error| KeystoreError::corrupt_file(error.to_string()))?;
            let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
            let mut key = Zeroizing::new([0u8; 32]);
            argon2
                .hash_password_into(password, salt, key.as_mut())
                .map_err(|error| KeystoreError::corrupt_file(error.to_string()))?;
            Ok(key)
        }
    }
}

// AES-256-GCM, header authenticated as AAD, tag appended to the ciphertext (wire layout: ciphertext || 16-byte tag).
pub(super) fn encrypt_aes256_gcm(key: &[u8], nonce: &[u8; AES_GCM_NONCE_LEN], aad: &[u8], buffer: &mut Vec<u8>) -> Result<(), KeystoreError> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| KeystoreError::corrupt_file("invalid AES key"))?;
    cipher.encrypt_in_place(&Nonce::from(*nonce), aad, buffer).map_err(|_| KeystoreError::AuthenticationFailed)
}

// Verifies the tag, then strips it in place so `buffer` holds only the plaintext on success.
pub(super) fn decrypt_aes256_gcm(key: &[u8], nonce: &[u8; AES_GCM_NONCE_LEN], aad: &[u8], buffer: &mut Vec<u8>) -> Result<(), KeystoreError> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| KeystoreError::corrupt_file("invalid AES key"))?;
    cipher.decrypt_in_place(&Nonce::from(*nonce), aad, buffer).map_err(|_| KeystoreError::AuthenticationFailed)
}

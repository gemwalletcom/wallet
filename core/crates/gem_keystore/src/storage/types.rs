use std::fmt;
use std::path::PathBuf;

use borsh::{BorshDeserialize, BorshSerialize};
use zeroize::{Zeroize, Zeroizing};

use super::constants::{AES_GCM_NONCE_LEN, ARGON2_SALT_LEN};
use crate::KeystoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum SecretKind {
    Mnemonic,
    PrivateKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredSecretMeta {
    pub keystore_id: String,
    pub kind: SecretKind,
    pub version: u8,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeystoreInspection {
    pub meta: Option<StoredSecretMeta>,
    pub authenticated: bool,
    pub file_len: u64,
    pub header_len: u32,
    pub ciphertext_len: u64,
    pub tag_len: u8,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeystoreFileError {
    pub path: PathBuf,
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileKeystore {
    pub(super) base_dir: PathBuf,
    pub(super) default_kdf: KdfParams,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub(super) struct Header {
    pub(super) keystore_id: String,
    pub(super) kind: SecretKind,
    pub(super) created_at: i64,
    pub(super) kdf: KdfParams,
    pub(super) cipher: CipherParams,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub(super) enum KdfParams {
    Argon2id {
        memory_kib: u32,
        iterations: u32,
        parallelism: u32,
        salt: [u8; ARGON2_SALT_LEN],
        output_len: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub(super) enum CipherParams {
    Aes256Gcm { nonce: [u8; AES_GCM_NONCE_LEN], tag_len: u8 },
}

#[derive(Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub(super) enum SecretPayload {
    Mnemonic { phrase: String },
    PrivateKey { bytes: Vec<u8> },
}

impl fmt::Debug for SecretPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecretPayload::Mnemonic { .. } => f.debug_struct("Mnemonic").field("phrase", &"<redacted>").finish(),
            SecretPayload::PrivateKey { .. } => f.debug_struct("PrivateKey").field("bytes", &"<redacted>").finish(),
        }
    }
}

impl Drop for SecretPayload {
    fn drop(&mut self) {
        match self {
            SecretPayload::Mnemonic { phrase } => phrase.zeroize(),
            SecretPayload::PrivateKey { bytes } => bytes.zeroize(),
        }
    }
}

impl SecretPayload {
    pub(super) fn into_mnemonic(mut self) -> Result<Zeroizing<String>, KeystoreError> {
        match &mut self {
            SecretPayload::Mnemonic { phrase } => Ok(Zeroizing::new(std::mem::take(phrase))),
            SecretPayload::PrivateKey { .. } => Err(KeystoreError::corrupt_file("stored secret is not a mnemonic")),
        }
    }

    pub(super) fn into_private_key(mut self) -> Result<Zeroizing<Vec<u8>>, KeystoreError> {
        match &mut self {
            SecretPayload::Mnemonic { .. } => Err(KeystoreError::corrupt_file("stored secret is not a private key")),
            SecretPayload::PrivateKey { bytes } => Ok(Zeroizing::new(std::mem::take(bytes))),
        }
    }
}

#[derive(Debug)]
pub(super) struct ParsedFile<'a> {
    pub(super) header: Header,
    pub(super) header_len: u32,
    pub(super) header_end: usize,
    pub(super) body: &'a [u8],
}

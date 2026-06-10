use std::fmt;
use std::path::PathBuf;

use strum::{AsRefStr, EnumString};
use zeroize::{Zeroize, Zeroizing};

use super::constants::{AES_GCM_NONCE_LEN, ARGON2_SALT_LEN};
use crate::KeystoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum SecretKind {
    Mnemonic,
    PrivateKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredSecretMeta {
    pub keystore_id: String,
    pub kind: SecretKind,
    pub version: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeystoreInspection {
    pub meta: Option<StoredSecretMeta>,
    pub authenticated: bool,
    pub file_len: u64,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Header {
    pub(super) keystore_id: String,
    pub(super) kind: SecretKind,
    pub(super) kdf: KdfParams,
    pub(super) cipher: CipherParams,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum KdfParams {
    Argon2id {
        memory_kib: u32,
        iterations: u32,
        parallelism: u32,
        salt: [u8; ARGON2_SALT_LEN],
        output_len: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CipherParams {
    Aes256Gcm { nonce: [u8; AES_GCM_NONCE_LEN], tag_len: u8 },
}

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
    pub(super) fn from_bytes(kind: SecretKind, bytes: Vec<u8>) -> Result<Self, KeystoreError> {
        match kind {
            SecretKind::Mnemonic => match String::from_utf8(bytes) {
                Ok(phrase) => Ok(SecretPayload::Mnemonic { phrase }),
                Err(error) => {
                    let mut bytes = error.into_bytes();
                    bytes.zeroize();
                    Err(KeystoreError::corrupt_file("mnemonic payload is not valid UTF-8"))
                }
            },
            SecretKind::PrivateKey => Ok(SecretPayload::PrivateKey { bytes }),
        }
    }

    pub(super) fn into_bytes(mut self) -> Zeroizing<Vec<u8>> {
        match &mut self {
            SecretPayload::Mnemonic { phrase } => Zeroizing::new(std::mem::take(phrase).into_bytes()),
            SecretPayload::PrivateKey { bytes } => Zeroizing::new(std::mem::take(bytes)),
        }
    }

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
pub(super) struct ParsedFile {
    pub(super) header: Header,
    pub(super) ciphertext: Vec<u8>,
}

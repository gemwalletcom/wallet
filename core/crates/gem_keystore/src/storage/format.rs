use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::{
    constants::{AES_GCM_NONCE_LEN, ARGON2_SALT_LEN, ENCRYPTED_BODY_CAP, PASSWORD_CAP, VERSION_V4},
    types::{CipherParams, Header, KdfParams, ParsedFile, SecretKind, StoredSecretMeta},
};
use crate::{KeystoreError, KeystoreId};

const KDF_ARGON2ID: &str = "argon2id";
const CIPHER_AES_256_GCM: &str = "aes-256-gcm";

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FileV4 {
    pub(super) version: u8,
    pub(super) id: String,
    pub(super) kind: String,
    pub(super) crypto: CryptoV4,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CryptoV4 {
    pub(super) kdf: KdfV4,
    pub(super) cipher: CipherV4,
    pub(super) ciphertext: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct KdfV4 {
    pub(super) algorithm: String,
    pub(super) memory_kib: u32,
    pub(super) iterations: u32,
    pub(super) parallelism: u32,
    pub(super) salt: String,
    pub(super) output_len: u32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CipherV4 {
    pub(super) algorithm: String,
    pub(super) nonce: String,
    pub(super) tag_len: u8,
}

/// Everything except the ciphertext, rebuilt from parsed values so the bytes are canonical
/// regardless of how the JSON on disk is formatted. Used as AES-GCM AAD.
#[derive(Serialize)]
struct AuthenticatedV4 {
    version: u8,
    id: String,
    kind: String,
    kdf: KdfV4,
    cipher: CipherV4,
}

pub(super) fn parse_v4(bytes: &[u8]) -> Result<ParsedFile, KeystoreError> {
    let file: FileV4 = serde_json::from_slice(bytes).map_err(|error| KeystoreError::corrupt_file(error.to_string()))?;
    if file.version != VERSION_V4 {
        return Err(KeystoreError::unsupported("version"));
    }
    let kind = SecretKind::from_str(&file.kind).map_err(|_| KeystoreError::corrupt_file("unknown secret kind"))?;
    if file.crypto.kdf.algorithm != KDF_ARGON2ID {
        return Err(KeystoreError::corrupt_file("unknown kdf algorithm"));
    }
    if file.crypto.cipher.algorithm != CIPHER_AES_256_GCM {
        return Err(KeystoreError::corrupt_file("unknown cipher algorithm"));
    }
    let salt: [u8; ARGON2_SALT_LEN] = decode_hex(&file.crypto.kdf.salt)?
        .try_into()
        .map_err(|_| KeystoreError::corrupt_file("invalid Argon2 salt length"))?;
    let nonce: [u8; AES_GCM_NONCE_LEN] = decode_hex(&file.crypto.cipher.nonce)?
        .try_into()
        .map_err(|_| KeystoreError::corrupt_file("invalid AES-GCM nonce length"))?;
    let ciphertext = decode_hex(&file.crypto.ciphertext)?;
    let header = Header {
        keystore_id: file.id,
        kind,
        kdf: KdfParams::Argon2id {
            memory_kib: file.crypto.kdf.memory_kib,
            iterations: file.crypto.kdf.iterations,
            parallelism: file.crypto.kdf.parallelism,
            salt,
            output_len: file.crypto.kdf.output_len,
        },
        cipher: CipherParams::Aes256Gcm {
            nonce,
            tag_len: file.crypto.cipher.tag_len,
        },
    };
    validate_header(&header)?;
    if ciphertext.len() < usize::from(header.cipher.tag_len()) {
        return Err(KeystoreError::corrupt_file("ciphertext shorter than tag"));
    }
    if ciphertext.len() > ENCRYPTED_BODY_CAP {
        return Err(KeystoreError::corrupt_file("ciphertext too large"));
    }
    Ok(ParsedFile { header, ciphertext })
}

pub(super) fn encode_v4(header: &Header, ciphertext: &[u8]) -> Result<Vec<u8>, KeystoreError> {
    let file = FileV4 {
        version: VERSION_V4,
        id: header.keystore_id.clone(),
        kind: header.kind.as_ref().to_string(),
        crypto: CryptoV4 {
            kdf: kdf_to_wire(&header.kdf),
            cipher: cipher_to_wire(&header.cipher),
            ciphertext: hex::encode(ciphertext),
        },
    };
    serde_json::to_vec_pretty(&file).map_err(|error| KeystoreError::corrupt_file(error.to_string()))
}

pub(super) fn authenticated_bytes(header: &Header) -> Result<Vec<u8>, KeystoreError> {
    let authenticated = AuthenticatedV4 {
        version: VERSION_V4,
        id: header.keystore_id.clone(),
        kind: header.kind.as_ref().to_string(),
        kdf: kdf_to_wire(&header.kdf),
        cipher: cipher_to_wire(&header.cipher),
    };
    serde_json::to_vec(&authenticated).map_err(|error| KeystoreError::corrupt_file(error.to_string()))
}

fn kdf_to_wire(kdf: &KdfParams) -> KdfV4 {
    match kdf {
        KdfParams::Argon2id {
            memory_kib,
            iterations,
            parallelism,
            salt,
            output_len,
        } => KdfV4 {
            algorithm: KDF_ARGON2ID.to_string(),
            memory_kib: *memory_kib,
            iterations: *iterations,
            parallelism: *parallelism,
            salt: hex::encode(salt),
            output_len: *output_len,
        },
    }
}

fn cipher_to_wire(cipher: &CipherParams) -> CipherV4 {
    match cipher {
        CipherParams::Aes256Gcm { nonce, tag_len } => CipherV4 {
            algorithm: CIPHER_AES_256_GCM.to_string(),
            nonce: hex::encode(nonce),
            tag_len: *tag_len,
        },
    }
}

fn decode_hex(value: &str) -> Result<Vec<u8>, KeystoreError> {
    hex::decode(value).map_err(|_| KeystoreError::corrupt_file("invalid hex"))
}

pub(super) fn meta_from_header(header: &Header) -> StoredSecretMeta {
    StoredSecretMeta {
        keystore_id: header.keystore_id.clone(),
        kind: header.kind,
        version: VERSION_V4,
    }
}

pub(crate) fn validate_v4_password(password: &[u8]) -> Result<(), KeystoreError> {
    if password.is_empty() || password.len() > PASSWORD_CAP {
        return Err(KeystoreError::invalid_input("password input"));
    }
    Ok(())
}

fn validate_header(header: &Header) -> Result<(), KeystoreError> {
    KeystoreId::parse(&header.keystore_id).map_err(|_| KeystoreError::corrupt_file("invalid keystore id"))?;
    header.kdf.validate()?;
    header.cipher.validate()?;
    Ok(())
}

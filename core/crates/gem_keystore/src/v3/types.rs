use std::fmt;

use serde::Deserialize;
use zeroize::{Zeroize, Zeroizing};

use super::constants::{DERIVED_KEY_LEN, MAX_SALT_LEN, MAX_SCRYPT_N, MAX_SCRYPT_P, MAX_SCRYPT_R, V3_CIPHER, V3_KDF, WHOLE_FILE_CAP};
use super::deserialize::{deserialize_ciphertext, deserialize_iv, deserialize_mac, deserialize_salt, map_json_error};
use crate::KeystoreError;

pub(crate) struct ReaderV3;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct KeystoreV3 {
    pub(super) crypto: CryptoV3,
    #[serde(rename = "type")]
    pub(super) kind: KindV3,
    pub(super) version: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct CryptoV3 {
    pub(super) cipher: String,
    pub(super) cipherparams: CipherParamsV3,
    #[serde(deserialize_with = "deserialize_ciphertext")]
    pub(super) ciphertext: Vec<u8>,
    pub(super) kdf: String,
    pub(super) kdfparams: KdfParamsV3,
    #[serde(deserialize_with = "deserialize_mac")]
    pub(super) mac: Vec<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CipherParamsV3 {
    #[serde(deserialize_with = "deserialize_iv")]
    pub(super) iv: Vec<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct KdfParamsV3 {
    pub(super) dklen: u32,
    pub(super) n: u64,
    pub(super) p: u32,
    pub(super) r: u32,
    #[serde(deserialize_with = "deserialize_salt")]
    pub(super) salt: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub(super) enum KindV3 {
    #[serde(rename = "mnemonic")]
    Mnemonic,
    #[serde(rename = "private-key")]
    PrivateKey,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum SecretV3 {
    Mnemonic(String),
    PrivateKey(Zeroizing<Vec<u8>>),
}

impl fmt::Debug for SecretV3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecretV3::Mnemonic(_) => f.debug_tuple("Mnemonic").field(&"<redacted>").finish(),
            SecretV3::PrivateKey(_) => f.debug_tuple("PrivateKey").field(&"<redacted>").finish(),
        }
    }
}

impl Drop for SecretV3 {
    fn drop(&mut self) {
        if let SecretV3::Mnemonic(phrase) = self {
            phrase.zeroize();
        }
    }
}

impl KeystoreV3 {
    pub(super) fn parse(input: &[u8]) -> Result<Self, KeystoreError> {
        if input.len() > WHOLE_FILE_CAP {
            return Err(KeystoreError::corrupt_file("v3 file too large"));
        }
        let keystore = serde_json::from_slice::<Self>(input).map_err(map_json_error)?;
        keystore.validate()?;
        Ok(keystore)
    }

    fn validate(&self) -> Result<(), KeystoreError> {
        if self.version != 3 {
            return Err(KeystoreError::unsupported("version"));
        }
        if self.crypto.cipher != V3_CIPHER {
            return Err(KeystoreError::corrupt_file("unsupported v3 cipher"));
        }
        if self.crypto.kdf != V3_KDF {
            return Err(KeystoreError::unsupported("v3 KDF"));
        }
        self.crypto.kdfparams.validate()?;
        Ok(())
    }
}

impl KdfParamsV3 {
    pub(super) fn validate(&self) -> Result<(), KeystoreError> {
        if self.dklen != DERIVED_KEY_LEN as u32 {
            return Err(KeystoreError::corrupt_file("invalid v3 scrypt dklen"));
        }
        if self.n == 0 || self.n > MAX_SCRYPT_N || !self.n.is_power_of_two() {
            return Err(KeystoreError::corrupt_file("invalid v3 scrypt n"));
        }
        if self.r == 0 || self.r > MAX_SCRYPT_R {
            return Err(KeystoreError::corrupt_file("invalid v3 scrypt r"));
        }
        if self.p == 0 || self.p > MAX_SCRYPT_P {
            return Err(KeystoreError::corrupt_file("invalid v3 scrypt p"));
        }
        // The MAC still authenticates the secret, so short/empty salts are safe; legacy WalletCore emitted empty scrypt salts. Only cap the max length.
        if self.salt.len() > MAX_SALT_LEN {
            return Err(KeystoreError::corrupt_file("invalid v3 hex length"));
        }
        Ok(())
    }
}

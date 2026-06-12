use std::path::Path;

use zeroize::Zeroize;

use crate::{KeystoreError, Mnemonic};

use super::{
    constants::{MNEMONIC_PLAINTEXT_CAP, V3_PASSWORD_CAP, WHOLE_FILE_CAP},
    crypto::decrypt_v3_json,
    types::{KeystoreV3, KindV3, ReaderV3, SecretV3},
};

impl ReaderV3 {
    pub(crate) fn decrypt_path(path: &Path, password: &[u8]) -> Result<SecretV3, KeystoreError> {
        let bytes = crate::storage::read_capped(path, WHOLE_FILE_CAP)?;
        Self::decrypt_json_bytes(&bytes, password)
    }

    #[cfg(test)]
    pub(super) fn decrypt_json(input: &str, password: &[u8]) -> Result<SecretV3, KeystoreError> {
        Self::decrypt_json_bytes(input.as_bytes(), password)
    }

    fn decrypt_json_bytes(input: &[u8], password: &[u8]) -> Result<SecretV3, KeystoreError> {
        if input.len() > WHOLE_FILE_CAP {
            return Err(KeystoreError::corrupt_file("v3 file too large"));
        }
        if password.len() > V3_PASSWORD_CAP {
            return Err(KeystoreError::invalid_input("password input"));
        }
        let keystore = KeystoreV3::parse(input)?;
        let mut plaintext = decrypt_v3_json(&keystore, password)?;
        Ok(match keystore.kind {
            KindV3::Mnemonic => {
                if plaintext.len() > MNEMONIC_PLAINTEXT_CAP {
                    plaintext.zeroize();
                    return Err(KeystoreError::corrupt_file("v3 mnemonic plaintext too large"));
                }
                let phrase = std::str::from_utf8(&plaintext).map_err(|_| KeystoreError::corrupt_file("invalid v3 mnemonic"))?;
                let sanitized = Mnemonic::sanitize(phrase).map_err(|_| KeystoreError::corrupt_file("invalid v3 mnemonic"))?;
                plaintext.zeroize();
                SecretV3::Mnemonic(sanitized)
            }
            KindV3::PrivateKey => {
                if plaintext.len() != 32 {
                    plaintext.zeroize();
                    return Err(KeystoreError::corrupt_file("invalid v3 private key"));
                }
                SecretV3::PrivateKey(plaintext)
            }
        })
    }
}

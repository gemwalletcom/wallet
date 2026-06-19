use serde::Deserialize;

use super::constants::{AES_128_CTR_IV_LEN, CIPHERTEXT_CAP, MAC_LEN, MAX_SALT_LEN};
use crate::KeystoreError;

const ERROR_HEX: &str = "invalid v3 hex";
const ERROR_HEX_LENGTH: &str = "invalid v3 hex length";
const ERROR_CIPHERTEXT_TOO_LARGE: &str = "v3 ciphertext too large";

pub(super) fn deserialize_iv<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_hex_in_range(deserializer, AES_128_CTR_IV_LEN, AES_128_CTR_IV_LEN, ERROR_HEX_LENGTH)
}

pub(super) fn deserialize_mac<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_hex_in_range(deserializer, MAC_LEN, MAC_LEN, ERROR_HEX_LENGTH)
}

pub(super) fn deserialize_salt<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_hex_in_range(deserializer, 0, MAX_SALT_LEN, ERROR_HEX_LENGTH)
}

pub(super) fn deserialize_ciphertext<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_hex_in_range(deserializer, 0, CIPHERTEXT_CAP, ERROR_CIPHERTEXT_TOO_LARGE)
}

fn deserialize_hex_in_range<'de, D>(deserializer: D, min: usize, max: usize, length_error: &'static str) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    let decoded = hex::decode(&value).map_err(|_| serde::de::Error::custom(ERROR_HEX))?;
    if decoded.len() < min || decoded.len() > max {
        return Err(serde::de::Error::custom(length_error));
    }
    Ok(decoded)
}

pub(super) fn map_json_error(error: serde_json::Error) -> KeystoreError {
    let message = error.to_string();
    for known_message in [ERROR_HEX_LENGTH, ERROR_HEX, ERROR_CIPHERTEXT_TOO_LARGE] {
        if message.contains(known_message) {
            return KeystoreError::corrupt_file(known_message);
        }
    }
    KeystoreError::corrupt_file(message)
}

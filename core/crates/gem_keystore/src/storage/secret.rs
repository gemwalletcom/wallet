use zeroize::Zeroize;

use crate::{KeystoreError, KeystoreId};

use super::{
    constants::{AES_GCM_TAG_LEN, ENCRYPTED_BODY_CAP, WHOLE_FILE_CAP},
    crypto::{decrypt_aes256_gcm, derive_key, encrypt_aes256_gcm},
    format::{authenticated_bytes, encode_v4, validate_v4_password},
    types::{CipherParams, Header, KdfParams, ParsedFile, SecretPayload},
};

pub(super) fn encrypt_secret(default_kdf: &KdfParams, payload: SecretPayload, password: &[u8], id: &KeystoreId) -> Result<Vec<u8>, KeystoreError> {
    validate_v4_password(password)?;
    let header = Header {
        keystore_id: id.as_str().to_owned(),
        kind: payload.kind(),
        kdf: default_kdf.with_random_salt()?,
        cipher: CipherParams::random_aes256_gcm()?,
    };
    let aad = authenticated_bytes(&header)?;
    let mut body = payload.into_bytes();
    if body.len() + AES_GCM_TAG_LEN as usize > ENCRYPTED_BODY_CAP {
        return Err(KeystoreError::corrupt_file("payload too large"));
    }
    let key = derive_key(password, &header.kdf)?;
    encrypt_aes256_gcm(key.as_ref(), header.cipher.nonce(), &aad, &mut body)?;
    let bytes = encode_v4(&header, &body)?;
    // Hex doubles the ciphertext; reject anything the read cap would refuse later.
    if bytes.len() > WHOLE_FILE_CAP {
        return Err(KeystoreError::corrupt_file("payload too large"));
    }
    Ok(bytes)
}

pub(super) fn decrypt_secret(parsed: ParsedFile, expected_id: Option<&KeystoreId>, password: &[u8]) -> Result<SecretPayload, KeystoreError> {
    validate_v4_password(password)?;
    if let Some(expected_id) = expected_id
        && parsed.header.keystore_id != expected_id.as_str()
    {
        return Err(KeystoreError::corrupt_file("authenticated keystore id does not match the requested id"));
    }
    let ParsedFile { header, ciphertext: mut body } = parsed;
    let aad = authenticated_bytes(&header)?;
    let key = derive_key(password, &header.kdf)?;
    // On tag failure aes-gcm leaves unauthenticated plaintext in the buffer, so zeroize before bailing.
    if let Err(error) = decrypt_aes256_gcm(key.as_ref(), header.cipher.nonce(), &aad, &mut body) {
        body.zeroize();
        return Err(error);
    }
    SecretPayload::from_bytes(header.kind, body)
}

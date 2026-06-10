use aes::Aes128;
use ctr::cipher::{KeyIvInit, StreamCipher};
use gem_crypto::compare::constant_time_eq;
use scrypt::{Params as ScryptParams, scrypt};
use sha3::{Digest, Keccak256};
use zeroize::Zeroizing;

use super::{
    constants::{AES_128_KEY_LEN, DERIVED_KEY_LEN},
    types::{KdfParamsV3, KeystoreV3},
};
use crate::KeystoreError;

pub(super) type Aes128Ctr = ctr::Ctr128BE<Aes128>;

pub(super) fn decrypt_v3_json(json: &KeystoreV3, password: &[u8]) -> Result<Zeroizing<Vec<u8>>, KeystoreError> {
    let derived_key = derive_scrypt_key(password, &json.crypto.kdfparams)?;
    verify_v3_mac(&derived_key, &json.crypto.ciphertext, &json.crypto.mac)?;
    let mut plaintext = Zeroizing::new(json.crypto.ciphertext.clone());
    let mut cipher =
        Aes128Ctr::new_from_slices(&derived_key[..AES_128_KEY_LEN], &json.crypto.cipherparams.iv).map_err(|_| KeystoreError::corrupt_file("invalid AES-128-CTR parameters"))?;
    cipher.apply_keystream(&mut plaintext);
    Ok(plaintext)
}

pub(super) fn derive_scrypt_key(password: &[u8], params: &KdfParamsV3) -> Result<Zeroizing<Vec<u8>>, KeystoreError> {
    params.validate()?;
    let log_n = u8::try_from(params.n.trailing_zeros()).map_err(|_| KeystoreError::corrupt_file("invalid v3 scrypt n"))?;
    let scrypt_params = ScryptParams::new(log_n, params.r, params.p).map_err(|error| KeystoreError::corrupt_file(error.to_string()))?;
    let mut derived_key = Zeroizing::new(vec![0u8; DERIVED_KEY_LEN]);
    scrypt(password, &params.salt, &scrypt_params, &mut derived_key).map_err(|error| KeystoreError::corrupt_file(error.to_string()))?;
    Ok(derived_key)
}

fn verify_v3_mac(derived_key: &[u8], ciphertext: &[u8], mac: &[u8]) -> Result<(), KeystoreError> {
    let mut hasher = Keccak256::new();
    hasher.update(&derived_key[AES_128_KEY_LEN..DERIVED_KEY_LEN]);
    hasher.update(ciphertext);
    let expected = hasher.finalize();
    if constant_time_eq(expected.as_slice(), mac) {
        Ok(())
    } else {
        Err(KeystoreError::AuthenticationFailed)
    }
}

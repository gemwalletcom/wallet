use ctr::cipher::{KeyIvInit, StreamCipher};
use sha3::{Digest, Keccak256};

use super::{
    constants::{AES_128_KEY_LEN, DERIVED_KEY_LEN},
    crypto::{Aes128Ctr, derive_scrypt_key},
    types::{KdfParamsV3, KeystoreV3},
};
use crate::v3_testkit::V3_ETHEREUM_ADDRESS;

const V3_MINIMAL_TEMPLATE: &str = include_str!("../../testdata/v3_minimal_template.json");

impl KeystoreV3 {
    pub(super) fn mock_json(kind: &str, plaintext: &[u8], password: &[u8]) -> String {
        Self::mock_json_with_salt(kind, plaintext, password, &[1u8; 16])
    }

    pub(super) fn mock_json_with_salt(kind: &str, plaintext: &[u8], password: &[u8], salt: &[u8]) -> String {
        let iv = [2u8; 16];
        let kdfparams = KdfParamsV3 {
            dklen: DERIVED_KEY_LEN as u32,
            n: 16,
            p: 1,
            r: 1,
            salt: salt.to_vec(),
        };
        let derived_key = derive_scrypt_key(password, &kdfparams).unwrap();
        let mut ciphertext = plaintext.to_vec();
        let mut cipher = Aes128Ctr::new_from_slices(&derived_key[..AES_128_KEY_LEN], &iv).unwrap();
        cipher.apply_keystream(&mut ciphertext);
        let mut hasher = Keccak256::new();
        hasher.update(&derived_key[AES_128_KEY_LEN..DERIVED_KEY_LEN]);
        hasher.update(&ciphertext);
        let mac = hasher.finalize();
        V3_MINIMAL_TEMPLATE
            .replace("__ETHEREUM_ADDRESS__", V3_ETHEREUM_ADDRESS)
            .replace("__PUBLIC_KEY_SUFFIX__", &"11".repeat(64))
            .replace("__IV__", &hex::encode(iv))
            .replace("__CIPHERTEXT__", &hex::encode(ciphertext))
            .replace("__SALT__", &hex::encode(salt))
            .replace("__MAC__", &hex::encode(mac))
            .replace("__KIND__", kind)
    }
}

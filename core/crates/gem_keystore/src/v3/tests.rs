use ctr::cipher::{KeyIvInit, StreamCipher};
use serde_json::Value;
use sha3::{Digest, Keccak256};
use zeroize::Zeroizing;

use super::{
    constants::{AES_128_KEY_LEN, DERIVED_KEY_LEN},
    crypto::{Aes128Ctr, derive_scrypt_key},
    types::{KdfParamsV3, KeystoreV3, KindV3, ReaderV3, SecretV3},
};
use crate::{
    KeystoreError,
    testkit::ABANDON_PHRASE,
    v3_testkit::{V3_ETHEREUM_ADDRESS, V3_MNEMONIC_FIXTURE, V3_MNEMONIC_PHRASE, V3_PASSWORD, V3_PRIVATE_KEY, V3_PRIVATE_KEY_FIXTURE},
};

const V3_MINIMAL_TEMPLATE: &str = include_str!("../../testdata/v3_minimal_template.json");

fn mock_v3_json(kind: &str, plaintext: &[u8], password: &[u8]) -> String {
    let salt = [1u8; 16];
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

#[test]
fn test_v3_decrypt_mnemonic_and_private_key() {
    let password = b"v3-password";
    let mnemonic_json = mock_v3_json("mnemonic", ABANDON_PHRASE.as_bytes(), password);
    let mnemonic = KeystoreV3::parse(mnemonic_json.as_bytes()).unwrap();
    assert_eq!(mnemonic.kind, KindV3::Mnemonic);
    assert_eq!(mnemonic.crypto.kdfparams.n, 16);
    assert_eq!(ReaderV3::decrypt_json(&mnemonic_json, password).unwrap(), SecretV3::Mnemonic(ABANDON_PHRASE.to_string()));

    let private_key = [9u8; 32];
    let private_key_json = mock_v3_json("private-key", &private_key, password);
    let private_key_secret = ReaderV3::decrypt_json(&private_key_json, password).unwrap();
    assert_eq!(KeystoreV3::parse(private_key_json.as_bytes()).unwrap().kind, KindV3::PrivateKey);
    assert_eq!(private_key_secret, SecretV3::PrivateKey(Zeroizing::new(private_key.to_vec())));

    let empty_password_json = mock_v3_json("private-key", &private_key, b"");
    let empty_password_secret = ReaderV3::decrypt_json(&empty_password_json, b"").unwrap();
    assert_eq!(empty_password_secret, SecretV3::PrivateKey(Zeroizing::new(private_key.to_vec())));
}

#[test]
fn test_v3_accepts_extra_legacy_metadata_fields() {
    let password = b"v3-password";
    let json = mock_v3_json("mnemonic", ABANDON_PHRASE.as_bytes(), password);
    let mut value: Value = serde_json::from_str(&json).unwrap();
    value.as_object_mut().unwrap().insert("legacyMetadata".to_string(), Value::Bool(true));

    let json = serde_json::to_string(&value).unwrap();

    assert_eq!(ReaderV3::decrypt_json(&json, password).unwrap(), SecretV3::Mnemonic(ABANDON_PHRASE.to_string()));
}

#[test]
fn test_v3_rejects_wrong_password_and_malformed_inputs() {
    let password = b"v3-password";
    let json = mock_v3_json("mnemonic", ABANDON_PHRASE.as_bytes(), password);
    assert_eq!(ReaderV3::decrypt_json(&json, b"wrong").unwrap_err(), KeystoreError::AuthenticationFailed);

    let bad_json = json.replace(r#""n": 16"#, r#""n": 32768"#);
    assert_eq!(KeystoreV3::parse(bad_json.as_bytes()).unwrap_err(), KeystoreError::corrupt_file("invalid v3 scrypt n"));

    let bad_hex = json.replace(r#""iv": "02020202020202020202020202020202""#, r#""iv": "zz""#);
    assert_eq!(KeystoreV3::parse(bad_hex.as_bytes()).unwrap_err(), KeystoreError::corrupt_file("invalid v3 hex"));

    let bad_mnemonic = mock_v3_json("mnemonic", b"not a recovery phrase", password);
    assert_eq!(
        ReaderV3::decrypt_json(&bad_mnemonic, password).unwrap_err(),
        KeystoreError::corrupt_file("invalid v3 mnemonic")
    );

    let large_password = vec![b'a'; 1024 * 1024 + 1];
    assert_eq!(ReaderV3::decrypt_json(&json, &large_password).unwrap_err(), KeystoreError::invalid_input("password input"));
}

#[test]
fn test_v3_private_key_length() {
    let json = mock_v3_json("private-key", &[1u8; 31], b"password");
    assert_eq!(
        ReaderV3::decrypt_json(&json, b"password").unwrap_err(),
        KeystoreError::corrupt_file("invalid v3 private key")
    );
}

#[test]
fn test_v3_ios_mnemonic_fixture() {
    let parsed = KeystoreV3::parse(V3_MNEMONIC_FIXTURE.as_bytes()).unwrap();
    let secret = ReaderV3::decrypt_json(V3_MNEMONIC_FIXTURE, V3_PASSWORD).unwrap();
    assert_eq!(parsed.kind, KindV3::Mnemonic);
    assert_eq!(parsed.crypto.kdf, "scrypt");
    assert_eq!(parsed.crypto.kdfparams.dklen, 32);
    assert_eq!(parsed.crypto.kdfparams.n, 16_384);
    assert_eq!(parsed.crypto.kdfparams.r, 8);
    assert_eq!(parsed.crypto.kdfparams.p, 4);
    assert_eq!(secret, SecretV3::Mnemonic(V3_MNEMONIC_PHRASE.to_string()));
}

#[test]
fn test_v3_ios_private_key_fixture() {
    let parsed = KeystoreV3::parse(V3_PRIVATE_KEY_FIXTURE.as_bytes()).unwrap();
    let secret = ReaderV3::decrypt_json(V3_PRIVATE_KEY_FIXTURE, V3_PASSWORD).unwrap();
    assert_eq!(parsed.kind, KindV3::PrivateKey);
    assert_eq!(parsed.crypto.kdf, "scrypt");
    assert_eq!(parsed.crypto.kdfparams.n, 16_384);
    assert_eq!(secret, SecretV3::PrivateKey(Zeroizing::new(hex::decode(V3_PRIVATE_KEY).unwrap())));

    let fixture: serde_json::Value = serde_json::from_str(V3_PRIVATE_KEY_FIXTURE).unwrap();
    let ciphertext = fixture.get("crypto").unwrap().get("ciphertext").unwrap().as_str().unwrap();
    assert_eq!(hex::decode(ciphertext).unwrap().len(), 32);
}

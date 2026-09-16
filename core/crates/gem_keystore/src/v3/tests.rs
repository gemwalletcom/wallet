use serde_json::Value;
use zeroize::Zeroizing;

use super::{
    constants::MAX_SALT_LEN,
    types::{KeystoreV3, KindV3, ReaderV3, SecretV3},
};
use crate::{
    KeystoreError,
    testkit::ABANDON_PHRASE,
    v3_testkit::{V3_MNEMONIC_FIXTURE, V3_MNEMONIC_PHRASE, V3_PASSWORD, V3_PRIVATE_KEY, V3_PRIVATE_KEY_FIXTURE},
};

#[test]
fn test_v3_decrypt_mnemonic_and_private_key() {
    let password = b"v3-password";
    let mnemonic_json = KeystoreV3::mock_json("mnemonic", ABANDON_PHRASE.as_bytes(), password);
    let mnemonic = KeystoreV3::parse(mnemonic_json.as_bytes()).unwrap();
    assert_eq!(mnemonic.kind, KindV3::Mnemonic);
    assert_eq!(mnemonic.crypto.kdfparams.n, 16);
    assert_eq!(ReaderV3::decrypt_json(&mnemonic_json, password).unwrap(), SecretV3::Mnemonic(ABANDON_PHRASE.to_string()));

    let private_key = [9u8; 32];
    let private_key_json = KeystoreV3::mock_json("private-key", &private_key, password);
    let private_key_secret = ReaderV3::decrypt_json(&private_key_json, password).unwrap();
    assert_eq!(KeystoreV3::parse(private_key_json.as_bytes()).unwrap().kind, KindV3::PrivateKey);
    assert_eq!(private_key_secret, SecretV3::PrivateKey(Zeroizing::new(private_key.to_vec())));

    let empty_password_json = KeystoreV3::mock_json("private-key", &private_key, b"");
    let empty_password_secret = ReaderV3::decrypt_json(&empty_password_json, b"").unwrap();
    assert_eq!(empty_password_secret, SecretV3::PrivateKey(Zeroizing::new(private_key.to_vec())));
}

#[test]
fn test_v3_accepts_extra_legacy_metadata_fields() {
    let password = b"v3-password";
    let json = KeystoreV3::mock_json("mnemonic", ABANDON_PHRASE.as_bytes(), password);
    let mut value: Value = serde_json::from_str(&json).unwrap();
    value.as_object_mut().unwrap().insert("legacyMetadata".to_string(), Value::Bool(true));

    let json = serde_json::to_string(&value).unwrap();

    assert_eq!(ReaderV3::decrypt_json(&json, password).unwrap(), SecretV3::Mnemonic(ABANDON_PHRASE.to_string()));
}

#[test]
fn test_v3_accepts_empty_and_short_scrypt_salt() {
    let password = b"v3-password";

    for salt in [Vec::new(), vec![7u8; 8]] {
        let json = KeystoreV3::mock_json_with_salt("mnemonic", ABANDON_PHRASE.as_bytes(), password, &salt);
        let parsed = KeystoreV3::parse(json.as_bytes()).unwrap();
        assert_eq!(parsed.crypto.kdfparams.salt.len(), salt.len());
        assert_eq!(ReaderV3::decrypt_json(&json, password).unwrap(), SecretV3::Mnemonic(ABANDON_PHRASE.to_string()));
        assert_eq!(ReaderV3::decrypt_json(&json, b"wrong").unwrap_err(), KeystoreError::AuthenticationFailed);
    }

    let valid = KeystoreV3::mock_json("mnemonic", ABANDON_PHRASE.as_bytes(), password);
    let mut value: Value = serde_json::from_str(&valid).unwrap();
    value["crypto"]["kdfparams"]["salt"] = Value::String(hex::encode(vec![1u8; MAX_SALT_LEN + 1]));
    let oversized = serde_json::to_string(&value).unwrap();
    assert_eq!(KeystoreV3::parse(oversized.as_bytes()).unwrap_err(), KeystoreError::corrupt_file("invalid v3 hex length"));
}

#[test]
fn test_v3_rejects_wrong_password_and_malformed_inputs() {
    let password = b"v3-password";
    let json = KeystoreV3::mock_json("mnemonic", ABANDON_PHRASE.as_bytes(), password);
    assert_eq!(ReaderV3::decrypt_json(&json, b"wrong").unwrap_err(), KeystoreError::AuthenticationFailed);

    let bad_json = json.replace(r#""n": 16"#, r#""n": 32768"#);
    assert_eq!(KeystoreV3::parse(bad_json.as_bytes()).unwrap_err(), KeystoreError::corrupt_file("invalid v3 scrypt n"));

    let bad_hex = json.replace(r#""iv": "02020202020202020202020202020202""#, r#""iv": "zz""#);
    assert_eq!(KeystoreV3::parse(bad_hex.as_bytes()).unwrap_err(), KeystoreError::corrupt_file("invalid v3 hex"));

    let bad_mnemonic = KeystoreV3::mock_json("mnemonic", b"not a recovery phrase", password);
    assert_eq!(
        ReaderV3::decrypt_json(&bad_mnemonic, password).unwrap_err(),
        KeystoreError::corrupt_file("invalid v3 mnemonic")
    );

    let invalid_utf8_mnemonic = KeystoreV3::mock_json("mnemonic", &[0xff, 0xfe], password);
    assert_eq!(
        ReaderV3::decrypt_json(&invalid_utf8_mnemonic, password).unwrap_err(),
        KeystoreError::corrupt_file("invalid v3 mnemonic")
    );

    let large_password = vec![b'a'; 1024 * 1024 + 1];
    assert_eq!(ReaderV3::decrypt_json(&json, &large_password).unwrap_err(), KeystoreError::invalid_input("password input"));
}

#[test]
fn test_v3_private_key_length() {
    let json = KeystoreV3::mock_json("private-key", &[1u8; 31], b"password");
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
